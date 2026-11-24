# Storage & media management Specification

## Purpose
Create/resize/compact/convert disks, diff chains, cached metadata (no full scan at startup), ISO library. Module `CP-05` (Storage & media management; Vietnamese: "Quản lý storage & media") from BRD-06, phase P1, priority Must. Owner crates: virtaxd, virtax-store. Depends on: `control-plane/registry-and-configuration-store`. VirtualBox reference: `src/VBox/Main/src-server/MediumImpl.cpp, src/VBox/Storage (VD backends)`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Media registry with cached metadata (FR-CP05-001)
Virtax SHALL provide "Media registry with cached metadata" as specified by FR-CP05-001 (BRD-06, priority Must, phase P1; Vietnamese: "Registry media có cache metadata").

#### Scenario: Specified behaviour of FR-CP05-001
- **WHEN** the behaviour FR-CP05-001 covers is exercised
- **THEN** it behaves as BRD-06 specifies: virtaxd stores each medium in the `media` table (id, location, kind, format, variant, type, parent_id, logical_size, size, mtime, state, last_error, checked_at). ListMedia answers from the cache without opening image files. In Phase 1 the cache is loaded once from VBoxSVC and kept warm by MediumRegistered/Changed events.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/MediumImpl.cpp:7342-7477 (i_queryInfo → VDOpen); Frontends/VirtualBox/src/medium/UIMediumEnumerator.cpp:71-100`

#### Scenario: Acceptance for FR-CP05-001
- **WHEN** the FR-CP05-001 acceptance check is run
- **THEN** Given 500 registered media, When ListMedia is called, Then daemon p95 ≤ 50 ms and 0 image-file open() calls (checked with strace/ETW).

### Requirement: Lazy re-validation via file watchers (FR-CP05-002)
Virtax SHALL provide "Lazy re-validation via file watchers" as specified by FR-CP05-002 (BRD-06, priority Must, phase P1; Vietnamese: "Re-validate lười theo file watcher").

#### Scenario: Specified behaviour of FR-CP05-002
- **WHEN** the behaviour FR-CP05-002 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Medium state (OK, Not attached, File missing, Inaccessible, Locked, Encrypted-locked) is recomputed when a file watcher (inotify/FSEvents/ReadDirectoryChangesW via the `notify` crate) reports an mtime/size change, right before a VM starts, or when the user clicks Re-check. Network/USB disks are never probed at startup. On registration, images whose size is not a multiple of 512 bytes are flagged (the legacy engine would fall back to buffered, and on Linux synchronous, I/O) with a 'Pad to 512' button.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/MediumImpl.cpp:2299-2318 (refreshState); Frontends/VirtualBox/src/medium/UIMedium.cpp:139-150; VMM/VMMR3/PDMR3AsyncCompletionFile.cpp:955-978`

#### Scenario: Acceptance for FR-CP05-002
- **WHEN** the FR-CP05-002 acceptance check is run
- **THEN** Renaming a disk file outside Virtax shows File missing within ≤ 2 s; unplugging a USB drive holding a disk never stalls the Manager (UI thread ≤ 50 ms).

### Requirement: Engine-aware disk creation (FR-CP05-003)
Virtax SHALL provide "Engine-aware disk creation" as specified by FR-CP05-003 (BRD-06, priority Must, phase P1; Vietnamese: "Tạo disk theo engine").

#### Scenario: Specified behaviour of FR-CP05-003
- **WHEN** the behaviour FR-CP05-003 covers is exercised
- **THEN** it behaves as BRD-06 specifies: CreateMedium is an Operation with %, ETA and cancel. Legacy engine: VDI (default), VMDK, VHD; dynamic or fixed. virtax-vmm: qcow2 (default) or sparse raw. Free space is checked before creating a fixed disk, with a warning when a dynamic disk exceeds 90% of free space.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/MediumImpl.cpp (createBaseStorage); Frontends/VirtualBox/src/wizards/newvd/*`

#### Scenario: Acceptance for FR-CP05-003
- **WHEN** the FR-CP05-003 acceptance check is run
- **THEN** Creating a 64 GB fixed VDI on a drive with 40 GB free is blocked with VTX-STO-010 and a dynamic suggestion; cancelling midway leaves no stray file.

### Requirement: Disk resize (grow) (FR-CP05-004)
Virtax SHALL provide "Disk resize (grow)" as specified by FR-CP05-004 (BRD-06, priority Must, phase P1; Vietnamese: "Resize disk (tăng dung lượng)").

#### Scenario: Specified behaviour of FR-CP05-004
- **WHEN** the behaviour FR-CP05-004 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Resize is an Operation. Phase 1 allows VDI/VHD/VMDK via IMedium::resize while the VM is off; qcow2/raw on the legacy engine is blocked with an explanation. Phase 2 virtax-vmm resizes qcow2/raw, including live for virtio-blk (config-change interrupt). After a resize Virtax suggests growing the partition (growpart via the agent or a guide).
- **AND** the VirtualBox reference for the current behaviour is `Storage/VDI.cpp:3283 (vdiResize); Storage/QCOW.cpp:2555-2556 (pfnResize NULL); Storage/RAW.cpp:1222-1223`

#### Scenario: Acceptance for FR-CP05-004
- **WHEN** the FR-CP05-004 acceptance check is run
- **THEN** Resizing a VDI from 40 to 80 GB finishes in ≤ 5 s and the guest sees 80 GB; resizing qcow2 on legacy shows VTX-STO-021 with a 'Convert' link.

### Requirement: Compact with reclaimable estimate (FR-CP05-005)
Virtax SHALL provide "Compact with reclaimable estimate" as specified by FR-CP05-005 (BRD-06, priority Should, phase P1; Vietnamese: "Compact và ước tính dung lượng thu hồi").

#### Scenario: Specified behaviour of FR-CP05-005
- **WHEN** the behaviour FR-CP05-005 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Each disk shows 'Reclaimable ≈ N GB' (on-disk minus guest usage from the agent/GA when available, otherwise estimated from zero blocks). Phase 1 compacts VDI/VHD via IMedium::compact and guides the user to run fstrim/sdelete first. Phase 2 uses discard → hole punching (FR-EN05-008), so offline compaction is not needed.
- **AND** the VirtualBox reference for the current behaviour is `Storage/VDI.cpp:2332 (vdiCompact); Storage/VHD.cpp:2354; Storage/VMDK.cpp:9192-9193 (pfnCompact NULL)`

#### Scenario: Acceptance for FR-CP05-005
- **WHEN** the FR-CP05-005 acceptance check is run
- **THEN** A 61 GB on-disk VDI with 47 GB real data shrinks to ≤ 49 GB after compaction; the Compact button is disabled for VMDK with a tooltip giving the reason.

### Requirement: Verified format conversion (FR-CP05-006)
Virtax SHALL provide "Verified format conversion" as specified by FR-CP05-006 (BRD-06, priority Should, phase P1; Vietnamese: "Convert định dạng có kiểm chứng").

#### Scenario: Specified behaviour of FR-CP05-006
- **WHEN** the behaviour FR-CP05-006 covers is exercised
- **THEN** it behaves as BRD-06 specifies: ConvertMedium turns VMDK/VHD/VHDX/VDI into qcow2, raw or VDI. The original is kept until the new image passes verification (logical-content comparison of a 1% block sample, or everything with 'Full verify'); only then are references in `.vbox`/`machine.toml` updated. Phase 1 uses IMedium::cloneTo with a target format; Phase 2 has a Rust converter in virtax-vmm-block.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/MediumImpl.cpp (cloneTo, i_taskCloneHandler); Storage/VHDX.cpp:2007-2013 (read-only)`

#### Scenario: Acceptance for FR-CP05-006
- **WHEN** the FR-CP05-006 acceptance check is run
- **THEN** Converting an 80 GB VMDK to qcow2 runs as an Operation with ETA and cancel; afterwards the VM boots from the new disk and `qemu-img check` reports 0 errors.

### Requirement: Atomic medium clone and move (FR-CP05-007)
Virtax SHALL provide "Atomic medium clone and move" as specified by FR-CP05-007 (BRD-06, priority Must, phase P1; Vietnamese: "Clone và move medium nguyên tử").

#### Scenario: Specified behaviour of FR-CP05-007
- **WHEN** the behaviour FR-CP05-007 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Clone creates a new UUID; move relocates the file and updates the registry, `medium_refs` and manifests in one journaled transaction. If the copy fails or is cancelled, state rolls back; two registered copies with the same UUID never exist.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/MediumImpl.cpp (moveTo, cloneTo); VirtualBoxImpl.cpp:4417-4447 (i_findHardDiskByLocation)`

#### Scenario: Acceptance for FR-CP05-007
- **WHEN** the FR-CP05-007 acceptance check is run
- **THEN** Killing virtaxd during a 30 GB move → after restart, reconcile restores exactly one valid location and the VM still boots.

### Requirement: Diff/overlay chains with depth warnings (FR-CP05-008)
Virtax SHALL provide "Diff/overlay chains with depth warnings" as specified by FR-CP05-008 (BRD-06, priority Should, phase P1; Vietnamese: "Chain diff/overlay và cảnh báo độ sâu").

#### Scenario: Specified behaviour of FR-CP05-008
- **WHEN** the behaviour FR-CP05-008 covers is exercised
- **THEN** it behaves as BRD-06 specifies: The API returns the base → diff/overlay tree with the VM and snapshot using each node and each layer's own size. When a chain is deeper than 8 layers, Virtax warns about the O(depth) read cost and suggests deleting/merging old snapshots; orphan diffs (no references) are listed for cleanup.
- **AND** the VirtualBox reference for the current behaviour is `Storage/VD.cpp:1755-1785 (chain walk); include/VBox/settings.h:144-163 (medium tree)`

#### Scenario: Acceptance for FR-CP05-008
- **WHEN** the FR-CP05-008 acceptance check is run
- **THEN** A VM with 10 snapshots shows an 11-layer tree in Media, a 'Deep chain' badge and a 'View snapshots' button; orphan diffs appear under the 'Orphans' filter.

### Requirement: Medium types with explanations (FR-CP05-009)
Virtax SHALL provide "Medium types with explanations" as specified by FR-CP05-009 (BRD-06, priority Should, phase P1; Vietnamese: "Loại medium có giải thích").

#### Scenario: Specified behaviour of FR-CP05-009
- **WHEN** the behaviour FR-CP05-009 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Supports Normal, Immutable (overlay discarded at power-off), Writethrough (excluded from snapshots), Shareable (multiple VMs write, fixed disks only), Readonly and Multi-attach (shared base, per-VM overlay). Each type has a one-line description; type changes are validated (e.g. Shareable rejects dynamic disks).
- **AND** the VirtualBox reference for the current behaviour is `Main/idl/VirtualBox.xidl (MediumType: Normal/Immutable/Writethrough/Shareable/Readonly/MultiAttach)`

#### Scenario: Acceptance for FR-CP05-009
- **WHEN** the FR-CP05-009 acceptance check is run
- **THEN** Setting a qcow2/VDI to Multi-attach and attaching it to 20 VMs creates 20 separate overlays; setting Shareable on a dynamic disk gives VTX-STO-030 with the reason.

### Requirement: Explicit cache mode per attachment (FR-CP05-010)
Virtax SHALL provide "Explicit cache mode per attachment" as specified by FR-CP05-010 (BRD-06, priority Must, phase P1; Vietnamese: "Cache mode rõ ràng cho mỗi attachment").

#### Scenario: Specified behaviour of FR-CP05-010
- **WHEN** the behaviour FR-CP05-010 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Each attachment has `cache` = writeback (default) | writethrough | none | unsafe. In Phase 1 virtaxd maps it to host I/O cache, BlockCache and IgnoreFlush/IgnoreFlushAsync via extradata; in Phase 2 to per-host open flags and flush calls (table in the additional content). Unsafe is only shown under 'Advanced' with a red warning.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Storage/DrvVD.cpp:84-94,1665-1667,4984; Main/xml/Settings.cpp:4012-4013; Main/src-server/StorageControllerImpl.cpp:138-142`

#### Scenario: Acceptance for FR-CP05-010
- **WHEN** the FR-CP05-010 acceptance check is run
- **THEN** Choosing writeback on the legacy engine yields IgnoreFlush=0 in the LUN CFGM (read back via the VM log); a guest `sync` followed by killing the VM keeps the synced data intact.

### Requirement: Phase-1 I/O tuning via extradata (FR-CP05-011)
Virtax SHALL provide "Phase-1 I/O tuning via extradata" as specified by FR-CP05-011 (BRD-06, priority Must, phase P1; Vietnamese: "Tinh chỉnh I/O Pha 1 qua extradata").

#### Scenario: Specified behaviour of FR-CP05-011
- **WHEN** the behaviour FR-CP05-011 covers is exercised
- **THEN** it behaves as BRD-06 specifies: For Virtax-managed VMs virtaxd sets per LUN: IoBufMax 32-64 MiB (unencrypted disks only, capped by host RAM), NVMe WorkerThreadsMax = vCPU count, host cache off for NVMe/AHCI/virtio-scsi. Values live in `[engine.vbox_legacy]` so they can be inspected and rolled back.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Storage/DrvVD.cpp:5011 (IoBufMax 5 MiB); Devices/Storage/DevNVMe.cpp:94,7207 (2 workers)`

#### Scenario: Acceptance for FR-CP05-011
- **WHEN** the FR-CP05-011 acceptance check is run
- **THEN** On an NVMe host, guest fio 1 MiB seq read QD32 ≥ 1.5× the same VM on VBox defaults; disabling the tuning restores the exact original values.

### Requirement: Discard/TRIM and solid-state flags (FR-CP05-012)
Virtax SHALL provide "Discard/TRIM and solid-state flags" as specified by FR-CP05-012 (BRD-06, priority Should, phase P1; Vietnamese: "Discard/TRIM và solid-state").

#### Scenario: Specified behaviour of FR-CP05-012
- **WHEN** the behaviour FR-CP05-012 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Attachments have `discard` and `ssd`. In Phase 1 discard only works for VDI (legacy engine), so the UI states 'marks only, does not return space' for other formats. In Phase 2 discard returns space to the host for qcow2/raw (FR-EN05-008). New Virtax-created VMs enable discard + ssd by default.
- **AND** the VirtualBox reference for the current behaviour is `Main/xml/Settings.cpp:4329,6533 (discard off by default); Storage/VDI.cpp:1221-1290`

#### Scenario: Acceptance for FR-CP05-012
- **WHEN** the FR-CP05-012 acceptance check is run
- **THEN** A newly created Ubuntu VM reports DISC-GRAN > 0 in `lsblk -D`; a VMDK disk shows the label 'TRIM does not return space'.

### Requirement: Encrypted disks: status and unlock (FR-CP05-013)
Virtax SHALL provide "Encrypted disks: status and unlock" as specified by FR-CP05-013 (BRD-06, priority Should, phase P1; Vietnamese: "Disk mã hoá: trạng thái và mở khoá").

#### Scenario: Specified behaviour of FR-CP05-013
- **WHEN** the behaviour FR-CP05-013 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Media shows the cipher (AES-XTS-128/256), lock state and the VMs using the disk. When starting a VM with encrypted disks, passwords come from the OS keystore (CP-12) or are asked once in a sheet instead of the R-17 table dialog. Re-keying is an Operation. Phase 1 goes through the legacy engine; Phase 2 through FR-EN05-011.
- **AND** the VirtualBox reference for the current behaviour is `Storage/VDFilterCrypt.cpp:59-62,141-142; Storage/VDKeyStore.cpp:100-144; Frontends/VirtualBox/src/widgets/UIAddDiskEncryptionPasswordDialog.cpp`

#### Scenario: Acceptance for FR-CP05-013
- **WHEN** the FR-CP05-013 acceptance check is run
- **THEN** A VM with 2 disks sharing a password asks once; with 'Remember in Keychain' the next start asks nothing; the password never appears in logs or the diagnostics bundle.

### Requirement: ISO library (FR-CP05-014)
Virtax SHALL provide "ISO library" as specified by FR-CP05-014 (BRD-06, priority Should, phase P1; Vietnamese: "ISO library").

#### Scenario: Specified behaviour of FR-CP05-014
- **WHEN** the behaviour FR-CP05-014 covers is exercised
- **THEN** it behaves as BRD-06 specifies: ISOs are registered from user-chosen folders (home is not scanned by default). Each ISO has OS/version/architecture detected asynchronously (via the CP-10 detector), a SHA-256, checksum status against the distro SUMS file when present, and the VMs mounting it. Duplicate hashes are merged; ISOs of a different architecture are labelled and unavailable for new VMs.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/UnattendedImpl.cpp:575-625 (detectIsoOS on caller thread); Main/xml/Settings.cpp (llDvdImages)`

#### Scenario: Acceptance for FR-CP05-014
- **WHEN** the FR-CP05-014 acceptance check is run
- **THEN** Adding a folder with 12 ISOs lists them immediately and fills OS metadata within ≤ 10 s without blocking the UI; an amd64 ISO on an M-series Mac is labelled 'Different architecture'.

### Requirement: Separate registry removal and file deletion (FR-CP05-015)
Virtax SHALL provide "Separate registry removal and file deletion" as specified by FR-CP05-015 (BRD-06, priority Must, phase P1; Vietnamese: "Gỡ khỏi registry và xoá file tách bạch").

#### Scenario: Specified behaviour of FR-CP05-015
- **WHEN** the behaviour FR-CP05-015 covers is exercised
- **THEN** it behaves as BRD-06 specifies: 'Remove from registry' only unregisters. 'Delete files' lists the exact files, sizes and child chains to be deleted, requires user confirmation, and moves them to Trash/Recycle Bin when the OS supports it. A medium attached to a running VM is never deleted.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/MediumImpl.cpp (deleteStorage, close); Frontends/VirtualBox/src/medium/UIMediumManager.cpp`

#### Scenario: Acceptance for FR-CP05-015
- **WHEN** the FR-CP05-015 acceptance check is run
- **THEN** Deleting a disk with 2 child diffs lists 3 files and the total size in the confirmation; for a running VM's disk the Delete button is disabled with the reason.
