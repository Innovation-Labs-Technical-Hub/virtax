# virtax-vmm-hv

Hypervisor backends for `virtax-vmm`; today KVM on Linux x86_64 through `kvm-ioctls`. This is the only crate in
the workspace allowed to use `unsafe`.

## Commands

```sh
cargo clippy -p virtax-vmm-hv -- -D warnings   # also enforces SAFETY comments on unsafe blocks
cargo xtask ci                                  # lint policy checks this crate's exemption
```

## Layout

- `src/kvm.rs`: `Vm` (memory regions, irqchip, PIT, irqfd, supported CPUID), `Vcpu` (registers, MSRs, LAPIC,
  `run`), `Exit`, `available`.
- `src/lib.rs`: platform gating; a backend-neutral trait arrives with the second backend.

## Conventions

- Every `unsafe` block has a `// SAFETY:` comment stating the invariant and who upholds it
  (`clippy::undocumented_unsafe_blocks = "deny"`).
- Expose safe functions only; callers never see raw pointers or file descriptors.
- Translate errors into `HvError` with the ioctl name.

## Gotchas

- Guest memory registered with `Vm::new` must outlive every vCPU; `virtax-vmm-core` hands each vCPU thread an
  `Arc` of the memory for this reason.
- This crate does not inherit the workspace lints (they forbid `unsafe`); it declares its own in `Cargo.toml`.

Rules: `../../.principles/vmm.md`, `../../.principles/rust.md` rule 1. Decision D4 (OpenVMM or rust-vmm) is open:
`openspec/specs/governance/decisions-and-open-questions/`.
