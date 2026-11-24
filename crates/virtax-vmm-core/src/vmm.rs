//! Build a VM from a `VmConfig` and run its boot vCPU until the guest powers off or resets.

use std::io::Write;
use std::path::PathBuf;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use virtax_vmm_hv::kvm::{Exit, HvError, IOAPIC_PINS, Vcpu, Vm};
use vmm_sys_util::eventfd::EventFd;

use crate::devices::{COM1_IRQ, PioAction, PioBus};
use crate::disk::{DiskConfig, DiskImage};
use crate::layout::{MIN_MEMORY_MIB, guest_memory};
use crate::pci::{PciBus, Platform};
use crate::virtio::VirtioPci;
use crate::virtio::block::Block;
use crate::x86::{acpi, boot, cpu};

/// Kernel parameters virtax-vmm needs: serial console, keyboard-controller reboot (which virtax-vmm turns into
/// a VM exit), and fast exit on panic. ACPI tells the kernel there is no PS/2 controller or VGA to
/// probe, and where the PCI host bridge is.
pub const DEFAULT_CMDLINE: &str = "console=ttyS0 reboot=k panic=1";

#[derive(Debug, Clone)]
pub struct VmConfig {
    pub memory_mib: u64,
    pub vcpus: u8,
    pub kernel: PathBuf,
    pub initrd: Option<PathBuf>,
    /// Appended to `DEFAULT_CMDLINE`.
    pub extra_cmdline: String,
    /// Raw images attached as virtio-blk devices, in this order.
    pub disks: Vec<DiskConfig>,
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

/// Requests the host can send to a running guest.
pub struct VmControl {
    power_button: EventFd,
}

impl VmControl {
    /// Press the ACPI power button. The guest decides what to do (usually an orderly shutdown).
    pub fn press_power_button(&self) -> std::io::Result<()> {
        self.power_button.write(1)
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

/// [`Platform`] on KVM: MSI routes, irqfds and ioeventfds of the VM, GSIs above the I/O APIC pins.
struct KvmPlatform {
    vm: Arc<Vm>,
    next_gsi: AtomicU32,
}

fn io_error(e: HvError) -> std::io::Error {
    std::io::Error::other(e.to_string())
}

impl Platform for KvmPlatform {
    fn allocate_gsi(&self) -> u32 {
        self.next_gsi.fetch_add(1, Ordering::Relaxed)
    }
    fn register_irqfd(&self, event: &EventFd, gsi: u32) -> std::io::Result<()> {
        self.vm.register_irqfd(event, gsi).map_err(io_error)
    }
    fn set_msi_route(&self, gsi: u32, address: u64, data: u32) -> std::io::Result<()> {
        self.vm.set_msi_route(gsi, address, data).map_err(io_error)
    }
    fn register_mmio_ioevent(&self, event: &EventFd, addr: u64) -> std::io::Result<()> {
        self.vm.register_mmio_ioevent(event, addr).map_err(io_error)
    }
    fn unregister_mmio_ioevent(&self, event: &EventFd, addr: u64) -> std::io::Result<()> {
        self.vm
            .unregister_mmio_ioevent(event, addr)
            .map_err(io_error)
    }
}

/// Run one vCPU until the guest stops the VM or the vCPU fails.
fn vcpu_loop(mut vcpu: Vcpu, bus: &Mutex<PioBus>, pci: &Mutex<PciBus>) -> Result<VmExit, VmmError> {
    loop {
        match vcpu.run()? {
            Exit::IoIn { port, data } => bus.lock().unwrap().read(port, data),
            Exit::IoOut { port, data } => match bus.lock().unwrap().write(port, data) {
                PioAction::Continue => {}
                PioAction::Reset => return Ok(VmExit::Reset),
                PioAction::PowerOff => return Ok(VmExit::PowerOff),
            },
            // Addresses nothing decodes: reads float high, writes are dropped.
            Exit::MmioRead { addr, data } => {
                if !pci.lock().unwrap().mmio_read(addr, data) {
                    data.fill(0xff);
                }
            }
            Exit::MmioWrite { addr, data } => {
                pci.lock().unwrap().mmio_write(addr, data);
            }
            Exit::Interrupted => {}
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
/// fully set up, just before the first instruction executes, and receives the VM's controls.
///
/// Every vCPU runs on its own thread. `run` returns as soon as any vCPU stops the VM; the other
/// vCPU threads stay parked in the hypervisor until the process exits, which is how `virtax-vmm` (one
/// process per VM) ends a VM.
pub fn run(
    config: &VmConfig,
    serial_out: Box<dyn Write + Send>,
    ready: impl FnOnce(VmControl),
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

    // Images are checked and locked before anything else, so a bad or busy image fails the start
    // before the guest runs.
    let images = config
        .disks
        .iter()
        .map(DiskImage::open)
        .collect::<Result<Vec<_>, _>>()
        .map_err(VmmError::Config)?;

    // vCPU threads hold the memory alive for as long as they may run guest code.
    let memory = Arc::new(guest_memory(config.memory_mib).map_err(VmmError::Config)?);
    let cmdline = config.cmdline();
    let rsdp = acpi::write_tables(&memory, config.vcpus).map_err(VmmError::Boot)?;
    let boot_images = boot::BootImages {
        kernel: &config.kernel,
        initrd: config.initrd.as_deref(),
        cmdline: &cmdline,
        memory_mib: config.memory_mib,
        acpi_rsdp: Some(rsdp),
    };
    let entry = boot::load(&memory, &boot_images).map_err(VmmError::Boot)?;
    cpu::write_boot_tables(&memory).map_err(VmmError::Boot)?;

    let vm = Arc::new(Vm::new(&memory)?);
    let platform: Arc<dyn Platform> = Arc::new(KvmPlatform {
        vm: Arc::clone(&vm),
        next_gsi: AtomicU32::new(IOAPIC_PINS),
    });
    let mut pci_bus = PciBus::new();
    for image in images {
        let serial = image
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let path = image.path.display().to_string();
        let block = Block::new(image, &serial, Arc::clone(&memory))?;
        let device = VirtioPci::new(Box::new(block), Arc::clone(&platform))?;
        let slot = pci_bus.add(Box::new(device)).map_err(VmmError::Config)?;
        tracing::info!(%path, slot, "virtio-blk attached");
    }
    let pci = Arc::new(Mutex::new(pci_bus));
    let serial_irq = EventFd::new(0)?;
    vm.register_irqfd(&serial_irq, COM1_IRQ)?;
    let power_button = EventFd::new(0)?;
    vm.register_irqfd(&power_button, acpi::GED_GSI)?;
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
    ready(VmControl { power_button });

    let (stopped_tx, stopped_rx) = mpsc::channel();
    for (id, vcpu) in vcpus.into_iter().enumerate() {
        let (bus, pci, memory, stopped) = (
            Arc::clone(&bus),
            Arc::clone(&pci),
            Arc::clone(&memory),
            stopped_tx.clone(),
        );
        thread::Builder::new()
            .name(format!("vcpu{id}"))
            .spawn(move || {
                let result = vcpu_loop(vcpu, &bus, &pci);
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
    drop(platform);
    drop(vm);
    result
}
