//! Hypervisor backends for oxvmm. Today: KVM on Linux x86_64.
//!
//! This is the only oxvmm crate allowed to use `unsafe`; every block carries a `SAFETY:` comment.
//! A backend-neutral trait will be extracted when the second backend (HVF or WHP) lands, using the
//! shape of this module as the reference.

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub mod kvm;
