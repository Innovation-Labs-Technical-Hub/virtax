---
phase: 12
title: "Cross-platform hosts and hypervisors"
status: pending
priority: P2
effort: "12 person-weeks (estimate)"
dependencies: [2, 3]
---

# Phase 12: Cross-platform hosts and hypervisors

## Goal

Virtax runs on macOS and Windows hosts and on arm64, as the six-combination platform matrix requires.

## Scope

- EN-03 hypervisor backends (`engine/virtax-vmm-core-and-hypervisor-backends`): HVF on macOS arm64, WHP on Windows,
  KVM on Linux arm64; a backend-neutral trait extracted from the KVM backend once the second backend exists.
- EN-04 arm64 virt platform (FR-EN04-007), firmware boot with OVMF/AAVMF (008/009).
- CP-01 Windows named pipe transport with a per-user DACL (FR-CP01-001); PL-03 OS integration and power profiles
  (FR-PL03-003 to 012); CP-12 sandboxing on macOS and Windows (FR-CP12-006).
- NFR-00-11 one codebase for six host combinations; FR-GEN-002 platform matrix.

## Gates

- **D4** must be CONFIRMED before writing the second backend (it decides OpenVMM crates or rust-vmm).
- **Q-10-02** macOS 14 support; **Q-12-04** self-hosted HVF/WHP runners (buy or rent).

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

- [ ] 1. Extract the hypervisor trait from `virtax-vmm-hv` with the KVM backend as its first implementation (no
      behaviour change; all `kvm_*` tests pass).
- [ ] 2. `add-kvm-arm64` (Linux arm64 runner already in CI), then `add-hvf-backend`, then `add-whp-backend`, each with
      its own boot tests on a matching runner.
- [ ] 3. `add-uefi-boot` with pinned OVMF/AAVMF fetched by `xtask` (FR-PL01-009).
- [ ] 4. `add-windows-named-pipe` and the Windows build of `virtaxd`/`virtaxctl`; add Windows to the CI matrix.

## Verification

- Boot tests pass on each host combination's runner; CI matrix includes Windows; `cargo xtask ci` stays one command.
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- No self-hosted macOS/Windows hypervisor runners yet → Q-12-04 decides; until then these backends are tested
  manually and marked so in the specs.
