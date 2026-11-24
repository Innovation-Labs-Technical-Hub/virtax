//! KVM backend: VM and vCPU file descriptors with safe wrappers.

use std::path::Path;

/// Register and CPUID types the boot code fills in.
pub use kvm_bindings;

use kvm_bindings::{
    CpuId, KVM_MAX_CPUID_ENTRIES, KVM_PIT_SPEAKER_DUMMY, Msrs, kvm_fpu, kvm_lapic_state,
    kvm_msr_entry, kvm_pit_config, kvm_regs, kvm_sregs, kvm_userspace_memory_region,
};
use kvm_ioctls::{Kvm, VcpuExit, VcpuFd, VmFd};
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

pub struct Vm {
    kvm: Kvm,
    fd: VmFd,
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
        Ok(Self { kvm, fd })
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

// errno values (Linux, all architectures oxvmm supports).
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
