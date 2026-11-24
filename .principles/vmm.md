# VMM principles

Scope: `apps/virtax-vmm`, `crates/virtax-vmm-core`, `crates/virtax-vmm-hv`, and the engine that drives them
(`crates/virtax-engine/src/vmm.rs`).

## Principles

1. **The guest is hostile.** Everything a guest writes (port and MMIO data, descriptor chains, configuration
   space) is untrusted input to a process running as the user (architecture §"Security & access control").
2. **`unsafe` lives in one small crate.** All hypervisor calls go through `virtax-vmm-hv`, behind safe wrappers
   with documented invariants, so the device model stays safe Rust (NFR-00-12).
3. **Emulate only what guests need, and tell the guest the truth.** The ACPI tables describe exactly the devices
   that exist, so guests never probe for hardware that is not there.
4. **One process per VM.** A crashing or compromised VM takes down only its own process; the engine supervises it
   through its exit code and signals (architecture §"C4 Level 3 — virtax-vmm").
5. **Prove it on a real guest.** A device is done when a real kernel uses it under KVM, not when unit tests pass.

## Rules

1. MUST put every `unsafe` block in `crates/virtax-vmm-hv`, each with a `SAFETY:` comment naming the invariant the
   caller upholds (for example, that guest memory outlives the VM).
2. MUST validate guest-provided addresses, lengths and indices before use, through `vm-memory` accessors that
   bounds-check; MUST NOT turn a guest value into a host pointer.
3. MUST describe every ISA or platform device a guest uses in the DSDT. A hardware-reduced platform has no legacy
   PIC, and Linux only wires interrupts the DSDT describes.
4. MUST keep the ACPI table builders and AML encoders covered by unit tests that check lengths, checksums and
   hand-assembled byte sequences (`x86/acpi.rs`, `x86/aml.rs`).
5. MUST keep the process contract: the ready line `virtax-vmm: guest running` on stderr; exit 0 for power-off, 3
   for a triple fault, 1 for VMM errors; a reboot re-executes the process; SIGTERM presses the power button;
   SIGKILL is the hard stop. Changing it means changing `VirtaxVmmEngine` in the same commit.
6. MUST add a `kvm_*` test that boots the pinned test guest for every device or boot-path change, and run
   `cargo xtask kvm-test` before merging.
7. MUST NOT keep emulation code that a test has not proven necessary: when an experiment does not fix the problem,
   remove it (the unused MADT interrupt override was removed this way).
8. SHOULD follow hardware and Linux behaviour over the letter of a spec when they disagree, and record the reason
   in a comment (Linux on x86 reads the CMOS clock as BCD whatever status register B says).
9. SHOULD do device I/O off the vCPU threads (ioeventfd plus a worker) once a device has real I/O.

## Enforcement

| Rule | Check |
|---|---|
| 1 | `cargo xtask ci` lint policy and `clippy::undocumented_unsafe_blocks = "deny"` in `crates/virtax-vmm-hv/Cargo.toml` |
| 4 | Unit tests in `crates/virtax-vmm-core/src/x86/` |
| 3, 5, 6 | `kvm_*` tests (`apps/virtax-vmm/tests/boot.rs`, `apps/virtaxctl/tests/e2e.rs`) in CI job `kvm boot tests` |
| 2, 7–9 | Code review. Not yet enforced: `cargo-fuzz` targets for guest-facing parsers and seccomp filters for `virtax-vmm` (architecture §"Security & access control") |

## Anti-patterns

- **A device that works only in the guest's printk.** Serial output appeared, but userspace output was lost,
  because COM1's interrupt was never wired on the hardware-reduced platform. Fix: describe COM1 in the DSDT; test
  the guest's userspace output, not only kernel messages.
- **Trusting a register's advertised mode.** The CMOS clock declared binary mode, and the guest booted into the
  wrong year. Fix: serve BCD, as Linux expects, and test the guest's own `date`.
- **A test that leaks a VM.** A failing test left `virtax-vmm` running. Fix: wrap child processes in a guard that
  kills them on drop (`Guest` in `apps/virtax-vmm/tests/boot.rs`).

## References

- Architecture §"C4 Level 3 — Component" (virtax-vmm), §"Security & access control", ADR-001, decision D4.
- `openspec/specs/engine/` (EN-03, EN-04, EN-05) and `docs/virtax-vmm.md`.
- Linux `Documentation/arch/x86/boot.rst`; ACPI 6.5 §5 and §20; virtio 1.2.
