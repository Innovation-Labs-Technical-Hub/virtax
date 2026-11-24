//! Build a VM from a `VmConfig` and run its boot vCPU until the guest powers off or resets.

use std::io::Write;
use std::path::PathBuf;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use oxvmm_hv::kvm::{Exit, HvError, Vcpu, Vm};
use vmm_sys_util::eventfd::EventFd;

use crate::devices::{COM1_IRQ, PioAction, PioBus};
use crate::layout::{MIN_MEMORY_MIB, guest_memory};
use crate::x86::{acpi, boot, cpu};

/// Kernel parameters oxvmm needs: serial console, keyboard-controller reboot (which oxvmm turns into
/// a VM exit), no PCI scan, and fast exit on panic. ACPI tells the kernel there is no PS/2
/// controller, VGA or CMOS clock to probe.
pub const DEFAULT_CMDLINE: &str = "console=ttyS0 reboot=k panic=1 pci=off";

#[derive(Debug, Clone)]
pub struct VmConfig {
    pub memory_mib: u64,
    pub vcpus: u8,
    pub kernel: PathBuf,
    pub initrd: Option<PathBuf>,
    /// Appended to `DEFAULT_CMDLINE`.
    pub extra_cmdline: String,
}

impl VmConfig {
    pub fn cmdline(&self) -> String {
        let extra = self.extra_cmdline.trim();
        if extra.is_empty() {
            DEFAULT_CMDLINE.to_string()
        } else {
            format!("{DEFAULT_CMDLINE} {extra}")
        }
    }
}

/// Why the VM stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmExit {
    /// The guest powered off through ACPI (Linux `poweroff`).
    PowerOff,
    /// The guest asked for a reboot (Linux `reboot`, or `panic=N`).
    Reset,
    /// A vCPU triple-faulted or KVM reported a shutdown.
    TripleFault,
}

#[derive(Debug, thiserror::Error)]
pub enum VmmError {
    #[error("unsupported configuration: {0}")]
    Config(String),
    #[error("boot: {0}")]
    Boot(String),
    #[error(transparent)]
    Hypervisor(#[from] HvError),
    #[error("guest error: {0}")]
    Guest(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Run one vCPU until the guest stops the VM or the vCPU fails.
fn vcpu_loop(mut vcpu: Vcpu, bus: &Mutex<PioBus>) -> Result<VmExit, VmmError> {
    loop {
        match vcpu.run()? {
            Exit::IoIn { port, data } => bus.lock().unwrap().read(port, data),
            Exit::IoOut { port, data } => match bus.lock().unwrap().write(port, data) {
                PioAction::Continue => {}
                PioAction::Reset => return Ok(VmExit::Reset),
                PioAction::PowerOff => return Ok(VmExit::PowerOff),
            },
            // No MMIO devices yet: reads float high, writes are dropped.
            Exit::MmioRead { data, .. } => data.fill(0xff),
            Exit::MmioWrite { .. } | Exit::Interrupted => {}
            Exit::Shutdown => return Ok(VmExit::TripleFault),
            Exit::Hlt => {
                return Err(VmmError::Guest(
                    "vCPU halted with no interrupt controller to wake it".into(),
                ));
            }
            Exit::Other(reason) => {
                return Err(VmmError::Guest(format!("unexpected vCPU exit: {reason}")));
            }
        }
    }
}

/// Boot the guest and run it until it powers off, reboots or crashes. `ready` runs once the VM is
/// fully set up, just before the first instruction executes.
///
/// Every vCPU runs on its own thread. `run` returns as soon as any vCPU stops the VM; the other
/// vCPU threads stay parked in the hypervisor until the process exits, which is how `oxvmm` (one
/// process per VM) ends a VM.
pub fn run(
    config: &VmConfig,
    serial_out: Box<dyn Write + Send>,
    ready: impl FnOnce(),
) -> Result<VmExit, VmmError> {
    if config.vcpus == 0 || config.vcpus > acpi::MAX_VCPUS {
        return Err(VmmError::Config(format!(
            "between 1 and {} vCPUs are supported, not {}",
            acpi::MAX_VCPUS,
            config.vcpus
        )));
    }
    if config.memory_mib < MIN_MEMORY_MIB {
        return Err(VmmError::Config(format!(
            "at least {MIN_MEMORY_MIB} MiB of memory is required"
        )));
    }

    // vCPU threads hold the memory alive for as long as they may run guest code.
    let memory = Arc::new(guest_memory(config.memory_mib).map_err(VmmError::Config)?);
    let cmdline = config.cmdline();
    let rsdp = acpi::write_tables(&memory, config.vcpus).map_err(VmmError::Boot)?;
    let images = boot::BootImages {
        kernel: &config.kernel,
        initrd: config.initrd.as_deref(),
        cmdline: &cmdline,
        memory_mib: config.memory_mib,
        acpi_rsdp: Some(rsdp),
    };
    let entry = boot::load(&memory, &images).map_err(VmmError::Boot)?;
    cpu::write_boot_tables(&memory).map_err(VmmError::Boot)?;

    let vm = Vm::new(&memory)?;
    let serial_irq = EventFd::new(0)?;
    vm.register_irqfd(&serial_irq, COM1_IRQ)?;
    let bus = Arc::new(Mutex::new(PioBus::new(serial_irq, serial_out)));
    let cpuid = vm.supported_cpuid()?;
    let mut vcpus = Vec::with_capacity(config.vcpus.into());
    for id in 0..config.vcpus {
        let vcpu = vm.create_vcpu(id)?;
        cpu::setup_vcpu(
            &vcpu,
            cpuid.clone(),
            id,
            config.vcpus,
            (id == 0).then_some(entry),
        )?;
        vcpus.push(vcpu);
    }
    tracing::info!(vcpus = config.vcpus, memory_mib = config.memory_mib, kernel = %config.kernel.display(), %cmdline, "guest starting");
    ready();

    let (stopped_tx, stopped_rx) = mpsc::channel();
    for (id, vcpu) in vcpus.into_iter().enumerate() {
        let (bus, memory, stopped) = (Arc::clone(&bus), Arc::clone(&memory), stopped_tx.clone());
        thread::Builder::new()
            .name(format!("vcpu{id}"))
            .spawn(move || {
                let result = vcpu_loop(vcpu, &bus);
                let _ = stopped.send(result);
                // Keep guest memory mapped until the process exits.
                drop(memory);
            })?;
    }
    drop(stopped_tx);
    let result = stopped_rx
        .recv()
        .map_err(|_| VmmError::Guest("every vCPU thread ended without a result".into()))?;
    bus.lock().unwrap().flush();
    // Closing the VM fd is safe while vCPU threads run: each open vCPU fd keeps the VM alive.
    drop(vm);
    result
}
