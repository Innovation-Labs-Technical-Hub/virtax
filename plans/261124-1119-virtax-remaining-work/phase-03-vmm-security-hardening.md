---
phase: 3
title: "VMM security hardening"
status: pending
priority: P1
effort: "2 person-weeks (estimate)"
dependencies: [2]
---

# Phase 3: VMM security hardening

## Goal

Every guest-facing parser in `virtax-vmm` is fuzzed in CI, and the VMM process runs under a seccomp filter after setup.

## Scope

- FR-CP12-007 fuzz every guest-facing parser (Must, P2); FR-CP12-006 sandboxing for the VMM process (Must, P2),
  Linux part (seccomp); macOS App Sandbox and Windows AppContainer move to phase 12.
- Parsers in scope once phase 2 lands: port-I/O and MMIO dispatch, PCI configuration space, MSI-X table, virtio
  common configuration, virtio descriptor chains and virtio-blk request headers.

## Gates

- None. The fuzzing toolchain is a pinned nightly used only by the fuzz job; the product stays on stable.

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

- [ ] 1. Scout the device code from phase 2 and list every function that reads guest-controlled data.
- [ ] 2. Add `fuzz/` (cargo-fuzz) with one target per parser, plus seed corpora from the KVM tests; add `cargo xtask
      fuzz --smoke` (60 s per target) to CI and a longer nightly run.
- [ ] 3. Add a seccomp allowlist (rust-vmm `seccompiler`) applied after VM setup and before the vCPU threads start;
      build it from the syscalls the KVM tests actually use.
- [ ] 4. Add a `kvm_*` test proving a disallowed syscall is refused, and keep every existing KVM test green.
- [ ] 5. Update `.principles/vmm.md` and `.principles/security.md` enforcement tables.

## Verification

- `cargo xtask fuzz --smoke` green in CI; each target found no crash in its smoke budget.
- All `kvm_*` tests pass with the seccomp filter on.
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- The seccomp allowlist is too tight for a rare path (for example I/O errors) → run the full KVM suite and the
  fuzz corpus under the filter; log the denied syscall number on a kill.
