---
phase: 2
title: "virtio-blk over PCI"
status: pending
priority: P1
effort: "3 person-weeks (estimate)"
dependencies: [1]
---

# Phase 2: virtio-blk over PCI

## Goal

`virtax-vmm` guests get a PCIe bus and virtio-blk disks backed by raw images, attached from `machine.toml`.

## Scope

Executes the OpenSpec change `openspec/changes/add-virtio-blk-pci/` exactly as planned: FR-EN04-001 (block,
single queue), FR-EN04-006 (PCIe root complex, partial), FR-EN05-001 (raw), FR-EN05-007 (writeback flush),
FR-EN05-012 (locking). The change's proposal lists what is out of scope.

## Gates

- **D4 / Q-10-01** (OpenVMM or rust-vmm): proceed on rust-vmm, the as-built default, and say so in the PR.
- **Q-06-01** (default disk format): not decided by this phase; it attaches existing raw images only.

## Files to Create / Modify

See the change's `tasks.md` and `design.md`; ownership is `crates/virtax-vmm-core`, `crates/virtax-vmm-hv`,
`apps/virtax-vmm`, `crates/virtax-engine`, `xtask/src/guest.rs`, `docs/virtax-vmm.md`.

## Tasks & Steps

- [ ] 1. Run `openspec-apply-change` on `add-virtio-blk-pci` and work through its 17 tasks in order (groups 1–5); tick
      each task only when its stated verification passes.
- [ ] 2. Open a PR per task group or one PR for the change, with CI green on all jobs.
- [ ] 3. After merge, archive the change (`openspec archive add-virtio-blk-pci`) so its requirements join the main
      specs; run `openspec validate --all --strict`.

## Verification

- `cargo xtask ci` and `cargo xtask kvm-test` pass, including the new `kvm_*` disk tests (pattern persists,
  read-only, two disks, qcow2 refused, locked image refused, end-to-end through `virtaxctl`).
- Every task in the change's `tasks.md` is checked.

## Risks

- MSI-X routing is new `unsafe` surface → wrapped in `virtax-vmm-hv` with `SAFETY:` comments, covered by a
  two-disk KVM test.
- Removing `pci=off` changes every guest boot → all existing `kvm_*` tests run with PCI enabled.
