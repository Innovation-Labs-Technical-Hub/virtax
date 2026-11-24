---
phase: 4
title: "Storage, media and snapshots"
status: pending
priority: P1
effort: "8 person-weeks (estimate)"
dependencies: [2]
---

# Phase 4: Storage, media and snapshots

## Goal

Machines on `virtax-vmm` have qcow2 disks, a media registry, offline and live snapshots, and clone/move.

## Scope

- EN-05 block backends (`engine/block-backends`): qcow2 with backing chains (FR-EN05-001/009), io_uring backend
  (002), one queue per vCPU (006), discard and write-zeroes (008), throttling (010), metrics (013). Encryption
  (011) moves to phase 7 with key management.
- CP-05 storage and media management (`control-plane/storage-and-media-management`): media registry, disk
  creation, resize, compact, conversion, clone/move, ISO library (FR-CP05-001 to 015).
- CP-04 snapshots, clone, move (`control-plane/snapshots-clone-move`): FR-CP04-001 to 011.
- `virtaxctl media` and `virtaxctl snapshot` commands (CP-15 parity).

## Gates

- **Q-06-01** default disk format for new VMs (proposed: qcow2) — needed before disk creation ships.
- **Q-06-02** convert VDI on "Move to virtax-vmm" (proposed: no) — affects CP-05 conversion.
- **Q-10-04** saved-state format (proposed: postcard + per-unit version tags) — needed before live snapshots.
- **Q-05-04** live or paused snapshots of running VMs by default.

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

Split into OpenSpec changes, each a PR, in this order (scout each before proposing it):
- [ ] 1. `add-qcow2-images`: qcow2 read/write with backing files, dirty bit and check on open.
- [ ] 2. `add-media-registry`: CP-05 registry, disk creation and resize, `virtaxctl media`.
- [ ] 3. `add-offline-snapshots`: overlays, snapshot tree read model, restore with safety snapshot, background merge.
- [ ] 4. `add-device-state-save`: saved state for vCPUs and devices (needs Q-10-04), then live snapshots and save/restore.
- [ ] 5. `add-clone-and-move`: full and linked clones, move VM, conflict handling.
- [ ] 6. `add-io-uring-multiqueue`: io_uring backend and one queue per vCPU behind the existing device.

## Verification

- Each change's acceptance scenarios pass as `kvm_*` or e2e tests; the fio targets of NFR-06-* are measured and
  reported (performance gates arrive in phase 13).
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- qcow2 corruption on crash → dirty bit, check on open, and crash-injection tests on the metadata write path.
- Live snapshot consistency → snapshot only after vCPUs pause and in-flight I/O drains; tested under write load.
