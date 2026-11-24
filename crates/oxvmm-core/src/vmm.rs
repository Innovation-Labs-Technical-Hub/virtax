//! Build a VM from a `VmConfig` and run its boot vCPU until the guest powers off or resets.

use std::io::Write;
use std::path::PathBuf;

use oxvmm_hv::kvm::{Exit, HvError, Vm};
use vmm_sys_util::eventfd::EventFd;

use crate::devices::{COM1_IRQ, PioAction, PioBus};
use crate::layout::{MIN_MEMORY_MIB, guest_memory};
use crate::x86::{boot, cpu};

/// Kernel parameters oxvmm needs: serial console, keyboard-controller reboot (which oxvmm turns into
/// a VM exit), no PCI scan, and fast exit on panic.
pub const DEFAULT_CMDLINE: &str =
    "console=ttyS0 reboot=k panic=1 pci=off i8042.noaux i8042.nomux i8042.nopnp i8042.dumbkbd";

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
    /// The guest asked for a reboot through the keyboard controller (Linux `reboot`, or `panic=N`).
    Reset,
    /// The vCPU triple-faulted or KVM reported a shutdown.
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

/// Boot the guest and run it to completion on the calling thread. `ready` runs once the VM is fully
/// set up, just before the first instruction executes.
pub fn run(
    config: &VmConfig,
    serial_out: Box<dyn Write + Send>,
    ready: impl FnOnce(),
) -> Result<VmExit, VmmError> {
    if config.vcpus != 1 {
        return Err(VmmError::Config(format!(
            "{} vCPUs requested; oxvmm boots a single vCPU until SMP tables (MP/ACPI) are implemented",
            config.vcpus
        )));
    }
    if config.memory_mib < MIN_MEMORY_MIB {
        return Err(VmmError::Config(format!(
            "at least {MIN_MEMORY_MIB} MiB of memory is required"
        )));
    }

    // Field order matters for drop order: the VM (which references guest memory) goes first.
    let memory = guest_memory(config.memory_mib).map_err(VmmError::Config)?;
    let cmdline = config.cmdline();
    let images = boot::BootImages {
        kernel: &config.kernel,
        initrd: config.initrd.as_deref(),
        cmdline: &cmdline,
        memory_mib: config.memory_mib,
    };
    let entry = boot::load(&memory, &images).map_err(VmmError::Boot)?;
    cpu::write_boot_tables(&memory).map_err(VmmError::Boot)?;

    let vm = Vm::new(&memory)?;
    let serial_irq = EventFd::new(0)?;
    vm.register_irqfd(&serial_irq, COM1_IRQ)?;
    let mut bus = PioBus::new(serial_irq, serial_out);
    let mut vcpu = vm.create_vcpu(0)?;
    cpu::setup_boot_vcpu(&vcpu, vm.supported_cpuid()?, entry)?;
    tracing::info!(memory_mib = config.memory_mib, kernel = %config.kernel.display(), %cmdline, "guest starting");
    ready();

    let exit = loop {
        match vcpu.run()? {
            Exit::IoIn { port, data } => bus.read(port, data),
            Exit::IoOut { port, data } => {
                if bus.write(port, data) == PioAction::Reset {
                    break VmExit::Reset;
                }
            }
            // No MMIO devices yet: reads float high, writes are dropped.
            Exit::MmioRead { data, .. } => data.fill(0xff),
            Exit::MmioWrite { .. } | Exit::Interrupted => {}
            Exit::Shutdown => break VmExit::TripleFault,
            Exit::Hlt => {
                return Err(VmmError::Guest(
                    "vCPU halted with no interrupt controller to wake it".into(),
                ));
            }
            Exit::Other(reason) => {
                return Err(VmmError::Guest(format!("unexpected vCPU exit: {reason}")));
            }
        }
    };
    bus.flush();
    drop(vcpu);
    drop(vm);
    drop(memory);
    Ok(exit)
}
