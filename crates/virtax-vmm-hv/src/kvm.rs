//! KVM backend: VM and vCPU file descriptors with safe wrappers.

use std::path::Path;

/// Register and CPUID types the boot code fills in.
pub use kvm_bindings;

use std::collections::BTreeMap;
use std::sync::Mutex;

use kvm_bindings::{
    CpuId, KVM_IRQ_ROUTING_IRQCHIP, KVM_IRQ_ROUTING_MSI, KVM_IRQCHIP_IOAPIC,
    KVM_IRQCHIP_PIC_MASTER, KVM_IRQCHIP_PIC_SLAVE, KVM_MAX_CPUID_ENTRIES, KVM_PIT_SPEAKER_DUMMY,
    KvmIrqRouting, Msrs, kvm_fpu, kvm_irq_routing_entry, kvm_irq_routing_irqchip,
    kvm_irq_routing_msi, kvm_lapic_state, kvm_msr_entry, kvm_pit_config, kvm_regs, kvm_sregs,
    kvm_userspace_memory_region,
};
use kvm_ioctls::{IoEventAddress, Kvm, NoDatamatch, VcpuExit, VcpuFd, VmFd};
use vm_memory::{GuestMemoryBackend, GuestMemoryMmap, GuestMemoryRegion};
use vmm_sys_util::eventfd::EventFd;

/// Guest-physical address of the 3-page region KVM needs for the TSS on Intel hosts. It sits just
/// below the 4 GiB boundary, inside the MMIO gap, where it cannot collide with guest RAM.
pub const KVM_TSS_ADDRESS: usize = 0xfffb_d000;

#[derive(Debug, thiserror::Error)]
pub enum HvError {
    #[error("KVM is not available: {0}")]
    Unavailable(String),
    #[error("{what}: {source}")]
    Kvm {
        what: &'static str,
        source: kvm_ioctls::Error,
    },
    #[error("guest memory region at {0:#x} is not backed by host memory")]
    UnbackedMemory(u64),
}

pub type Result<T> = std::result::Result<T, HvError>;

fn kvm_err(what: &'static str) -> impl FnOnce(kvm_ioctls::Error) -> HvError {
    move |source| HvError::Kvm { what, source }
}

/// Whether `/dev/kvm` can be opened read-write by this process.
pub fn available() -> std::result::Result<(), String> {
    let path = Path::new("/dev/kvm");
    if !path.exists() {
        return Err("/dev/kvm does not exist (is virtualization enabled in firmware and the kvm module loaded?)".into());
    }
    Kvm::new()
        .map(|_| ())
        .map_err(|e| format!("cannot open /dev/kvm: {e} (is the user in the `kvm` group?)"))
}

/// A vCPU exit that the VMM must handle. Port and MMIO data buffers borrow the vCPU's run area.
#[derive(Debug)]
pub enum Exit<'a> {
    IoIn {
        port: u16,
        data: &'a mut [u8],
    },
    IoOut {
        port: u16,
        data: &'a [u8],
    },
    MmioRead {
        addr: u64,
        data: &'a mut [u8],
    },
    MmioWrite {
        addr: u64,
        data: &'a [u8],
    },
    /// Triple fault or an explicit shutdown: the guest reset itself.
    Shutdown,
    /// The vCPU halted (only seen without an in-kernel irqchip).
    Hlt,
    /// A signal interrupted `KVM_RUN`; run again.
    Interrupted,
    /// Anything else; fatal for the VM.
    Other(String),
}

/// Interrupt pins of the in-kernel I/O APIC; MSI routes use the GSIs above them.
pub const IOAPIC_PINS: u32 = 24;

pub struct Vm {
    kvm: Kvm,
    fd: VmFd,
    /// MSI routes by GSI. KVM replaces the whole routing table on every update, so the VMM keeps
    /// the full set and re-sends it together with the default interrupt-controller routes.
    msi_routes: Mutex<BTreeMap<u32, (u64, u32)>>,
}

impl Vm {
    /// Create a VM with an in-kernel interrupt controller and PIT, backed by `memory`.
    pub fn new(memory: &GuestMemoryMmap) -> Result<Self> {
        let kvm = Kvm::new().map_err(|e| HvError::Unavailable(e.to_string()))?;
        let fd = kvm.create_vm().map_err(kvm_err("KVM_CREATE_VM"))?;
        for (slot, region) in memory.iter().enumerate() {
            let host = region
                .get_host_address(vm_memory::MemoryRegionAddress(0))
                .map_err(|_| HvError::UnbackedMemory(region.start_addr().0))?;
            let mem_region = kvm_userspace_memory_region {
                slot: slot as u32,
                flags: 0,
                guest_phys_addr: region.start_addr().0,
                memory_size: region.len(),
                userspace_addr: host as u64,
            };
            // SAFETY: `host` points at the start of a live mmap of `region.len()` bytes owned by
            // `memory`. The caller keeps `memory` alive for as long as this VM exists (the VMM owns
            // both and drops the VM first), so KVM never accesses unmapped host memory.
            unsafe { fd.set_user_memory_region(mem_region) }
                .map_err(kvm_err("KVM_SET_USER_MEMORY_REGION"))?;
        }
        fd.set_tss_address(KVM_TSS_ADDRESS)
            .map_err(kvm_err("KVM_SET_TSS_ADDR"))?;
        fd.create_irq_chip()
            .map_err(kvm_err("KVM_CREATE_IRQCHIP"))?;
        let pit = kvm_pit_config {
            flags: KVM_PIT_SPEAKER_DUMMY,
            ..Default::default()
        };
        fd.create_pit2(pit).map_err(kvm_err("KVM_CREATE_PIT2"))?;
        Ok(Self {
            kvm,
            fd,
            msi_routes: Mutex::new(BTreeMap::new()),
        })
    }

    /// Route `gsi` to the MSI message `(address, data)`, so that writing an irqfd registered on
    /// `gsi` delivers that message. `gsi` must be at or above [`IOAPIC_PINS`].
    pub fn set_msi_route(&self, gsi: u32, address: u64, data: u32) -> Result<()> {
        assert!(gsi >= IOAPIC_PINS, "GSI {gsi} belongs to the I/O APIC");
        let mut routes = self.msi_routes.lock().unwrap();
        routes.insert(gsi, (address, data));
        let routing =
            KvmIrqRouting::from_entries(&routing_entries(&routes)).map_err(|_| HvError::Kvm {
                what: "too many interrupt routes",
                source: kvm_ioctls::Error::new(libc_e2big()),
            })?;
        self.fd
            .set_gsi_routing(&routing)
            .map_err(kvm_err("KVM_SET_GSI_ROUTING"))
    }

    /// Signal `event` instead of exiting to the VMM when the guest writes to the MMIO address `addr`.
    pub fn register_mmio_ioevent(&self, event: &EventFd, addr: u64) -> Result<()> {
        self.fd
            .register_ioevent(event, &IoEventAddress::Mmio(addr), NoDatamatch)
            .map_err(kvm_err("KVM_IOEVENTFD"))
    }

    /// Undo [`Vm::register_mmio_ioevent`].
    pub fn unregister_mmio_ioevent(&self, event: &EventFd, addr: u64) -> Result<()> {
        self.fd
            .unregister_ioevent(event, &IoEventAddress::Mmio(addr), NoDatamatch)
            .map_err(kvm_err("KVM_IOEVENTFD deassign"))
    }

    /// Deliver interrupt `gsi` to the guest whenever `event` is written.
    pub fn register_irqfd(&self, event: &EventFd, gsi: u32) -> Result<()> {
        self.fd
            .register_irqfd(event, gsi)
            .map_err(kvm_err("KVM_IRQFD"))
    }

    /// The CPUID leaves this host's KVM can expose.
    pub fn supported_cpuid(&self) -> Result<CpuId> {
        self.kvm
            .get_supported_cpuid(KVM_MAX_CPUID_ENTRIES)
            .map_err(kvm_err("KVM_GET_SUPPORTED_CPUID"))
    }

    pub fn create_vcpu(&self, id: u8) -> Result<Vcpu> {
        let fd = self
            .fd
            .create_vcpu(id.into())
            .map_err(kvm_err("KVM_CREATE_VCPU"))?;
        Ok(Vcpu { fd })
    }
}

pub struct Vcpu {
    fd: VcpuFd,
}

impl Vcpu {
    pub fn set_cpuid(&self, cpuid: &CpuId) -> Result<()> {
        self.fd.set_cpuid2(cpuid).map_err(kvm_err("KVM_SET_CPUID2"))
    }

    pub fn set_regs(&self, regs: &kvm_regs) -> Result<()> {
        self.fd.set_regs(regs).map_err(kvm_err("KVM_SET_REGS"))
    }

    pub fn sregs(&self) -> Result<kvm_sregs> {
        self.fd.get_sregs().map_err(kvm_err("KVM_GET_SREGS"))
    }

    pub fn set_sregs(&self, sregs: &kvm_sregs) -> Result<()> {
        self.fd.set_sregs(sregs).map_err(kvm_err("KVM_SET_SREGS"))
    }

    pub fn set_fpu(&self, fpu: &kvm_fpu) -> Result<()> {
        self.fd.set_fpu(fpu).map_err(kvm_err("KVM_SET_FPU"))
    }

    /// Set MSRs; fails unless every entry was accepted.
    pub fn set_msrs(&self, entries: &[kvm_msr_entry]) -> Result<()> {
        let msrs = Msrs::from_entries(entries).map_err(|_| HvError::Kvm {
            what: "too many MSRs",
            source: kvm_ioctls::Error::new(libc_e2big()),
        })?;
        let set = self.fd.set_msrs(&msrs).map_err(kvm_err("KVM_SET_MSRS"))?;
        if set != entries.len() {
            return Err(HvError::Kvm {
                what: "KVM_SET_MSRS accepted only some MSRs",
                source: kvm_ioctls::Error::new(libc_einval()),
            });
        }
        Ok(())
    }

    pub fn lapic(&self) -> Result<kvm_lapic_state> {
        self.fd.get_lapic().map_err(kvm_err("KVM_GET_LAPIC"))
    }

    pub fn set_lapic(&self, lapic: &kvm_lapic_state) -> Result<()> {
        self.fd.set_lapic(lapic).map_err(kvm_err("KVM_SET_LAPIC"))
    }

    /// Run until the next exit the VMM must handle.
    pub fn run(&mut self) -> Result<Exit<'_>> {
        match self.fd.run() {
            Ok(VcpuExit::IoIn(port, data)) => Ok(Exit::IoIn { port, data }),
            Ok(VcpuExit::IoOut(port, data)) => Ok(Exit::IoOut { port, data }),
            Ok(VcpuExit::MmioRead(addr, data)) => Ok(Exit::MmioRead { addr, data }),
            Ok(VcpuExit::MmioWrite(addr, data)) => Ok(Exit::MmioWrite { addr, data }),
            Ok(VcpuExit::Shutdown) => Ok(Exit::Shutdown),
            Ok(VcpuExit::Hlt) => Ok(Exit::Hlt),
            Ok(VcpuExit::Intr) => Ok(Exit::Interrupted),
            Ok(other) => Ok(Exit::Other(format!("{other:?}"))),
            Err(e) if e.errno() == libc_eintr() || e.errno() == libc_eagain() => {
                Ok(Exit::Interrupted)
            }
            Err(e) => Err(HvError::Kvm {
                what: "KVM_RUN",
                source: e,
            }),
        }
    }
}

/// The routing table KVM installs with the in-kernel irqchip (GSIs 0-15 on the PICs except the
/// cascade, 0-23 on the I/O APIC), followed by the MSI routes.
fn routing_entries(msi: &BTreeMap<u32, (u64, u32)>) -> Vec<kvm_irq_routing_entry> {
    let irqchip = |gsi: u32, chip: u32, pin: u32| {
        let mut e = kvm_irq_routing_entry {
            gsi,
            type_: KVM_IRQ_ROUTING_IRQCHIP,
            ..Default::default()
        };
        e.u.irqchip = kvm_irq_routing_irqchip { irqchip: chip, pin };
        e
    };
    let mut entries = Vec::new();
    for gsi in 0..IOAPIC_PINS {
        if gsi < 16 && gsi != 2 {
            let (chip, pin) = if gsi < 8 {
                (KVM_IRQCHIP_PIC_MASTER, gsi)
            } else {
                (KVM_IRQCHIP_PIC_SLAVE, gsi - 8)
            };
            entries.push(irqchip(gsi, chip, pin));
        }
        entries.push(irqchip(gsi, KVM_IRQCHIP_IOAPIC, gsi));
    }
    for (&gsi, &(address, data)) in msi {
        let mut e = kvm_irq_routing_entry {
            gsi,
            type_: KVM_IRQ_ROUTING_MSI,
            ..Default::default()
        };
        e.u.msi = kvm_irq_routing_msi {
            address_lo: address as u32,
            address_hi: (address >> 32) as u32,
            data,
            ..Default::default()
        };
        entries.push(e);
    }
    entries
}

// errno values (Linux, all architectures virtax-vmm supports).
const fn libc_eintr() -> i32 {
    4
}
const fn libc_eagain() -> i32 {
    11
}
const fn libc_e2big() -> i32 {
    7
}
const fn libc_einval() -> i32 {
    22
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_keeps_the_default_irqchip_routes() {
        let mut msi = BTreeMap::new();
        msi.insert(24, (0xfee0_0000, 0x4041));
        let entries = routing_entries(&msi);
        // 15 PIC routes (16 pins minus the cascade) + 24 I/O APIC routes + 1 MSI route.
        assert_eq!(entries.len(), 15 + 24 + 1);
        let com1: Vec<_> = entries.iter().filter(|e| e.gsi == 4).collect();
        assert_eq!(com1.len(), 2, "GSI 4 reaches both the PIC and the I/O APIC");
        let last = entries.last().unwrap();
        assert_eq!((last.gsi, last.type_), (24, KVM_IRQ_ROUTING_MSI));
        // SAFETY: the entry was built with its `msi` variant just above.
        let m = unsafe { last.u.msi };
        assert_eq!(
            (m.address_lo, m.address_hi, m.data),
            (0xfee0_0000, 0, 0x4041)
        );
    }
}
