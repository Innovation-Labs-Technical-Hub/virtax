---
phase: 11
title: "Legacy VirtualBox engine"
status: pending
priority: P1
effort: "8 person-weeks (estimate)"
dependencies: [1]
---

# Phase 11: Legacy VirtualBox engine

## Goal

`virtaxd` drives VirtualBox 7.2 VMs through VBoxCAPI behind the `Engine` trait, so existing VirtualBox users can use Virtax immediately.

## Scope

- EN-01 engine trait and legacy adapter, remaining FRs (`engine/engine-trait-and-legacy-adapter`): `vbox-sys`
  bindings (FR-EN01-002), XIDL codegen (003), FFI threads with deadlines (004), event bridge (005), VBoxSVC
  supervision (006), detection and version gate (007/008), error mapping (011), extradata profiles (010).
- CP-02 `.vbox` shadowing (FR-CP02-010) and EN-02 `vbx-shmfb` adapter (FR-EN02-001, 012, 013).

## Gates

- **D2** Phase 1 engine distribution (proposed: BYO VirtualBox 7.2 for alpha, Engine Pack for beta).
- **Q-10-05** Engine Pack with HM for Windows x64; **Q-02-02** Guest Additions with a BYO engine.
- Environment: VirtualBox 7.2 installed on a development host and a CI runner (not available today).

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

- [ ] 1. Install VirtualBox 7.2 on a dev host and a self-hosted runner; add a `vbox_*` test class like `kvm_*`.
- [ ] 2. `add-vbox-sys`: bindgen over the vendored `VBoxCAPI_v7_2.h`, runtime loading and version check (`unsafe`
      allowed here; add to `UNSAFE_ALLOWED` in review).
- [ ] 3. `add-vbox-adapter`: XIDL-generated safe API, FFI worker pool with deadlines, `LegacyEngine` implementing the
      trait, passing the shared engine contract suite (FR-EN01-001: ≥ 60 cases).
- [ ] 4. `add-vbox-event-bridge` and VBoxSVC supervision; then the `vbx-shmfb` adapter for the console.

## Verification

- The engine contract suite passes on both `VirtaxVmmEngine` and `LegacyEngine`; `vbox_*` tests create, start and
  stop a VirtualBox VM through `virtaxctl`.
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- A hung VBoxSVC freezes callers → every FFI call has a deadline on its own thread pool (FR-EN01-004).
- Licence: VBoxCAPI headers are GPL; Extension Pack (PUEL) code is never distributed (Q-06-05).
