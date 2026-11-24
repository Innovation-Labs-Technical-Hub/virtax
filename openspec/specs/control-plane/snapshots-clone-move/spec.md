# Snapshots, clone, move Specification

## Purpose
Live/offline snapshots, snapshot tree, background delete/merge with ETA, full/linked clone, VM move. Module `CP-04` (Snapshots, clone, move; Vietnamese: "Snapshot, clone, move") from BRD-05, phase P1, priority Must. Owner crates: virtaxd. Depends on: `control-plane/vm-lifecycle-and-runtime-orchestration`, `control-plane/storage-and-media-management`. VirtualBox reference: `src/VBox/Main/src-server/SnapshotImpl.cpp, MachineImplCloneVM.cpp, MachineImplMoveVM.cpp`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Live and offline snapshots (FR-CP04-001)
Virtax SHALL provide "Live and offline snapshots" as specified by FR-CP04-001 (BRD-05, priority Must, phase P1; Vietnamese: "Chụp snapshot live và offline").

#### Scenario: Specified behaviour of FR-CP04-001
- **WHEN** the behaviour FR-CP04-001 covers is exercised
- **THEN** it behaves as BRD-05 specifies: `Snapshots.Take(name, description, live)` defaults to live when running (no pause) and offline when off; the Operation shows % of RAM saved; Phase 2 asks the agent to quiesce filesystems (fsfreeze/VSS) first.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/SnapshotImpl.cpp:1634-1729`

#### Scenario: Acceptance for FR-CP04-001
- **WHEN** the FR-CP04-001 acceptance check is run
- **THEN** On a running 8 GB VM no guest-visible pause exceeds 500 ms; an offline snapshot takes ≤ 2 s.

### Requirement: Free-space and feasibility pre-check (FR-CP04-002)
Virtax SHALL provide "Free-space and feasibility pre-check" as specified by FR-CP04-002 (BRD-05, priority Must, phase P1; Vietnamese: "Kiểm tra chỗ trống và khả năng trước khi chạy").

#### Scenario: Specified behaviour of FR-CP04-002
- **WHEN** the behaviour FR-CP04-002 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Before take: RAM size (live) + diff estimate; before delete: total merge size versus free space (VirtualBox only finds out midway); warn about immutable/shareable disks (as R-12 does); every check completes before any lock is taken.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/SnapshotImpl.cpp:3276-3294`

#### Scenario: Acceptance for FR-CP04-002
- **WHEN** the FR-CP04-002 acceptance check is run
- **THEN** Insufficient space blocks with VTX-SNP-002 "Needs 9.2 GB, 4 GB free" + an Open Media manager button; 0 locks taken, 0 bytes written.

### Requirement: Snapshot tree read model (FR-CP04-003)
Virtax SHALL provide "Snapshot tree read model" as specified by FR-CP04-003 (BRD-05, priority Must, phase P1; Vietnamese: "Read model cây snapshot").

#### Scenario: Specified behaviour of FR-CP04-003
- **WHEN** the behaviour FR-CP04-003 covers is exercised
- **THEN** it behaves as BRD-05 specifies: `Snapshots.List` returns the tree with name, time, live/offline, added size (disk + state), hardware diff versus now, thumbnail and the "Current state" position; it updates through by-value `SnapshotTaken/Deleted/Restored` events.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/SnapshotImpl.cpp:1330-1379`

#### Scenario: Acceptance for FR-CP04-003
- **WHEN** the FR-CP04-003 acceptance check is run
- **THEN** A 50-snapshot tree comes from the read model within 50 ms; after take/delete the UI tree updates within 200 ms without a full reload.

### Requirement: Restore with a safety snapshot (FR-CP04-004)
Virtax SHALL provide "Restore with a safety snapshot" as specified by FR-CP04-004 (BRD-05, priority Must, phase P1; Vietnamese: "Restore có snapshot an toàn").

#### Scenario: Specified behaviour of FR-CP04-004
- **WHEN** the behaviour FR-CP04-004 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Restore the selected snapshot; if the current state changed, offer "Snapshot current state first" (on by default); a running VM gets the save/power-off step in the same flow; a "Revert to current snapshot" quick action exists.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/SnapshotImpl.cpp:2219-2337`

#### Scenario: Acceptance for FR-CP04-004
- **WHEN** the FR-CP04-004 acceptance check is run
- **THEN** Restoring a running VM completes in one flow with ≤ 3 interactions; the prior state stays in the tree as "Auto · before restore".

### Requirement: Snapshot delete/merge as a background Operation with ETA (FR-CP04-005)
Virtax SHALL provide "Snapshot delete/merge as a background Operation with ETA" as specified by FR-CP04-005 (BRD-05, priority Must, phase P1; Vietnamese: "Xoá/merge snapshot là Operation nền có ETA").

#### Scenario: Specified behaviour of FR-CP04-005
- **WHEN** the behaviour FR-CP04-005 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Delete one snapshot, a subtree or a range; the VM keeps running (online merge on the legacy engine); progress per disk, ETA from measured throughput; cancel at the next safe point (between disks) with the tree always consistent.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/SnapshotImpl.cpp:2669-2727,2992-3131`

#### Scenario: Acceptance for FR-CP04-005
- **WHEN** the FR-CP04-005 acceptance check is run
- **THEN** ETA within ±20% after 10% progress; the UI and other VMs are unaffected; mid-way cancellation leaves a valid tree in 100/100 runs.

### Requirement: I/O-friendly merge (FR-CP04-006)
Virtax SHALL provide "I/O-friendly merge" as specified by FR-CP04-006 (BRD-05, priority Should, phase P1; Vietnamese: "Merge thân thiện I/O").

#### Scenario: Specified behaviour of FR-CP04-006
- **WHEN** the behaviour FR-CP04-006 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Merges run at background I/O priority with rate limiting while the VM is in the foreground; optionally defer on battery < 20%; never run two merges on the same physical drive at once.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Devices/Storage/DrvVD.cpp:5049-5053`

#### Scenario: Acceptance for FR-CP04-006
- **WHEN** the FR-CP04-006 acceptance check is run
- **THEN** During a merge the foreground VM's p95 disk latency stays ≤ 1.5× baseline; with the VM idle the merge reaches ≥ 70% of sequential throughput.

### Requirement: Edit snapshot details (FR-CP04-007)
Virtax SHALL provide "Edit snapshot details" as specified by FR-CP04-007 (BRD-05, priority Should, phase P1; Vietnamese: "Sửa chi tiết snapshot").

#### Scenario: Specified behaviour of FR-CP04-007
- **WHEN** the behaviour FR-CP04-007 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Rename, add notes (plain text), view the hardware at snapshot time and the screenshot; edits never rewrite the whole configuration (Phase 2 keeps one `snapshots/<uuid>.toml` per snapshot).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/include/VBox/settings.h:1465-1487`

#### Scenario: Acceptance for FR-CP04-007
- **WHEN** the FR-CP04-007 acceptance check is run
- **THEN** Renaming a snapshot responds within 100 ms; the `SnapshotChanged` event reaches every client within 200 ms.

### Requirement: Full/linked clone backend (FR-CP04-008)
Virtax SHALL provide "Full/linked clone backend" as specified by FR-CP04-008 (BRD-05, priority Must, phase P1; Vietnamese: "Backend clone full/linked").

#### Scenario: Specified behaviour of FR-CP04-008
- **WHEN** the behaviour FR-CP04-008 covers is exercised
- **THEN** it behaves as BRD-05 specifies: `Machines.Clone(source, snapshot?, mode, scope, options)` → Operation mapping `CloneMode`/`CloneOptions`; a linked clone creates a base snapshot when none exists; free-space pre-check; Phase 2 uses reflink/`clonefile` (btrfs/XFS/APFS/ReFS) for instant full clones.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImplCloneVM.cpp:862-1182`

#### Scenario: Acceptance for FR-CP04-008
- **WHEN** the FR-CP04-008 acceptance check is run
- **THEN** A linked clone of a 40 GB VM takes ≤ 5 s; a 40 GB full clone on APFS/btrfs (Phase 2) ≤ 2 s; "regenerate" yields 100% new MACs.

### Requirement: Move VM (FR-CP04-009)
Virtax SHALL provide "Move VM" as specified by FR-CP04-009 (BRD-05, priority Should, phase P1; Vietnamese: "Move VM").

#### Scenario: Specified behaviour of FR-CP04-009
- **WHEN** the behaviour FR-CP04-009 covers is exercised
- **THEN** it behaves as BRD-05 specifies: `Machines.Move(target)` → Operation; same-volume moves rename (O(1)) instead of copying; cross-volume moves copy, verify, then delete the source; failures roll back leaving the source intact; the VM must be Off/Saved.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImplMoveVM.cpp:454-498,705-883`

#### Scenario: Acceptance for FR-CP04-009
- **WHEN** the FR-CP04-009 acceptance check is run
- **THEN** Moving a 40 GB VM on the same volume takes ≤ 2 s; cross-volume shows an ETA; unplugging the target midway leaves the source runnable.

### Requirement: Automatic snapshot policies (FR-CP04-010)
Virtax SHALL provide "Automatic snapshot policies" as specified by FR-CP04-010 (BRD-05, priority Could, phase P1; Vietnamese: "Chính sách snapshot tự động").

#### Scenario: Specified behaviour of FR-CP04-010
- **WHEN** the behaviour FR-CP04-010 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Take snapshots automatically before risky actions (restore, "Move to virtax-vmm", chipset/firmware changes) and on a schedule (daily/weekly) with "keep last N" retention; automatic snapshots are labelled "Auto" and pruned by retention.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP04-010
- **WHEN** the FR-CP04-010 acceptance check is run
- **THEN** A daily schedule keeping 7 leaves exactly 7 Auto snapshots after 10 days; cleanup merges run while the VM is idle or off.

### Requirement: Next-generation snapshot storage for virtax-vmm (FR-CP04-011)
Virtax SHALL provide "Next-generation snapshot storage for virtax-vmm" as specified by FR-CP04-011 (BRD-05, priority Should, phase P2; Vietnamese: "Lưu trữ snapshot thế hệ mới cho virtax-vmm").

#### Scenario: Specified behaviour of FR-CP04-011
- **WHEN** the behaviour FR-CP04-011 covers is exercised
- **THEN** it behaves as BRD-05 specifies: `virtax-vmm` uses qcow2 overlays with background block-commit (VM I/O continues), asynchronous RAM-state writes and one TOML file per snapshot; no more full Hardware copies nested in one large config file.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:9864-9926`

#### Scenario: Acceptance for FR-CP04-011
- **WHEN** the FR-CP04-011 acceptance check is run
- **THEN** Deleting a 20 GB overlay snapshot commits at ≥ 70% disk throughput while the VM runs fio; adding the 50th snapshot does not increase config save time.
