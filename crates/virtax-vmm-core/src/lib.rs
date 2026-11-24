//! virtax-vmm core: guest memory, x86_64 direct kernel boot, legacy devices and the vCPU run loop.
//! Hypervisor access goes through `virtax-vmm-hv`; this crate contains no `unsafe`.

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod devices;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub mod layout;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod rtc;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod vmm;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod x86;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub use vmm::{DEFAULT_CMDLINE, VmConfig, VmControl, VmExit, VmmError, run};

/// Whether this host can run virtax-vmm guests, with the reason when it cannot.
pub fn host_supported() -> Result<(), String> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        virtax_vmm_hv::kvm::available()
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        Err("virtax-vmm currently supports Linux x86_64 hosts only (KVM)".into())
    }
}
