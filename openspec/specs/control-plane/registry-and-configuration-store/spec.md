# Registry & configuration store Specification

## Purpose
SQLite (WAL) registry + read model; per-VM machine.toml; .vbox 1.3-1.22 migration; ui_prefs separated from VMs. Module `CP-02` (Registry & configuration store; Vietnamese: "Registry & lưu cấu hình") from BRD-03, phase P1, priority Must. Owner crates: virtax-store, virtax-model. Depends on: none. VirtualBox reference: `src/VBox/Main/xml (Settings.cpp 10.5k), include/VBox/settings.h, VirtualBox-settings.xsd`. Implementation status (2026-11-24): partial — SQLite registry with migrations, `machine.toml` + JSON Schema + validation, atomic writes with `.prev`, folder locks, etag, field-mask updates with dry run, pending changes, startup reconcile; no `.vbox` shadowing or `ui_prefs` API yet.

## Requirements

### Requirement: Per-user SQLite WAL registry (FR-CP02-001)
Virtax SHALL provide "Per-user SQLite WAL registry" as specified by FR-CP02-001 (BRD-03, priority Must, phase P1; Vietnamese: "Registry SQLite WAL cho mỗi user").

#### Scenario: Specified behaviour of FR-CP02-001
- **WHEN** the behaviour FR-CP02-001 covers is exercised
- **THEN** it behaves as BRD-03 specifies: One `registry.db` per user on a local disk (`~/.local/share/virtax/`, `~/Library/Application Support/Virtax/`, `%LOCALAPPDATA%/Virtax/`) with `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON`, `busy_timeout=5000`. A single writer task in `virtaxd` batches transactions; reads go through a connection pool. The registry only holds the registry, read model and caches: authoritative configuration lives in `.vbox` (legacy) or `machine.toml` (virtax-vmm).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/xml/Settings.cpp:2456-2870 (MainConfigFile, VirtualBox.xml)`

#### Scenario: Acceptance for FR-CP02-001
- **WHEN** the FR-CP02-001 acceptance check is run
- **THEN** Deleting `registry.db` and restarting rebuilds the registry from VBoxSVC/manifests, losing only `ui_prefs` and Operation history; a network-drive path is refused with `VTX-CFG-020`.

### Requirement: Controlled schema and migrations (FR-CP02-002)
Virtax SHALL provide "Controlled schema and migrations" as specified by FR-CP02-002 (BRD-03, priority Must, phase P1; Vietnamese: "Schema và migration có kiểm soát").

#### Scenario: Specified behaviour of FR-CP02-002
- **WHEN** the behaviour FR-CP02-002 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Tables: `schema_migrations`, `machines`, `machine_groups`, `machine_tags`, `pending_changes`, `snapshots`, `media`, `medium_refs`, `networks`, `operations`, `event_journal`, `ui_prefs`, plus derived index tables `machine_attachments` and `machine_nics`. Numbered, forward-only migrations run in a transaction at daemon start after a `VACUUM INTO` backup; a registry newer than the daemon is refused with a clear error.
- **AND** the VirtualBox reference for the current behaviour is `new (VirtualBox bumps settings versions in Settings.cpp:2603)`

#### Scenario: Acceptance for FR-CP02-002
- **WHEN** the FR-CP02-002 acceptance check is run
- **THEN** A migration failing midway rolls back, leaves the old version intact and reports `VTX-CFG-021`; a downgraded daemon never writes to a newer registry.

### Requirement: `summary_json` read model kept warm by events (FR-CP02-003)
Virtax SHALL provide "`summary_json` read model kept warm by events" as specified by FR-CP02-003 (BRD-03, priority Must, phase P1; Vietnamese: "Read model `summary_json` giữ ấm bằng event").

#### Scenario: Specified behaviour of FR-CP02-003
- **WHEN** the behaviour FR-CP02-003 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Each VM has a `summary_json` (MachineSummary DTO: name, state, OS, groups, tags, RAM, vCPU, current snapshot, snapshot count, engine, accessible, error code) updated from engine events; `ListMachines` returns everything in one message instead of ≈1,250 COM calls for 50 VMs. At startup the read model is served at once and refreshed in the background (stale-while-revalidate).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManageList.cpp:2377-2420; UIVirtualMachineItemLocal.cpp:68`

#### Scenario: Acceptance for FR-CP02-003
- **WHEN** the FR-CP02-003 acceptance check is run
- **THEN** `ListMachines` for 500 VMs ≤ 20 ms p95 from the registry; a VM state change updates `summary_json` within ≤ 200 ms; a full resync matches engine data 100%.

### Requirement: `machine.toml` with the `virtax.machine/1` JSON Schema (FR-CP02-004)
Virtax SHALL provide "`machine.toml` with the `virtax.machine/1` JSON Schema" as specified by FR-CP02-004 (BRD-03, priority Must, phase P1; Vietnamese: "`machine.toml` có JSON Schema `virtax.machine/1`").

#### Scenario: Specified behaviour of FR-CP02-004
- **WHEN** the behaviour FR-CP02-004 covers is exercised
- **THEN** it behaves as BRD-03 specifies: The manifest has `schema`, `id`, `name`, `os_type`, `groups`, `tags`, `engine` and the `[platform]`, `[cpu]`, `[memory]`, `[display]`, `[[storage.controllers]]`, `[[nics]]`, `[integration]`, `[engine.vbox_legacy]` tables. A JSON Schema generated from the serde types (`schemars`) ships with the package (`share/virtax/schema/machine-1.json`, `virtaxctl schema machine`) and is validated on every read/write; `[x-*]` tables and unknown keys survive round trips. It never holds passwords or keys, only keystore references.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/xml/VirtualBox-settings.xsd (not validated at runtime, Settings.cpp:55-57)`

#### Scenario: Acceptance for FR-CP02-004
- **WHEN** the FR-CP02-004 acceptance check is run
- **THEN** `virtaxctl vm validate` reports line/column for 30 error samples; round trips keep `[x-*]` intact; a CI scan finds 0 secrets in sample and test-generated manifests.

### Requirement: Atomic write protocol (FR-CP02-005)
Virtax SHALL provide "Atomic write protocol" as specified by FR-CP02-005 (BRD-03, priority Must, phase P1; Vietnamese: "Giao thức ghi nguyên tử").

#### Scenario: Specified behaviour of FR-CP02-005
- **WHEN** the behaviour FR-CP02-005 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Manifest write: (1) hold the folder lock, (2) serialize + validate, (3) write `machine.toml.tmp` in the same folder and fsync (`F_FULLFSYNC` on macOS, `FlushFileBuffers` on Windows), (4) keep the old file as `machine.toml.prev`, (5) atomically rename tmp → `machine.toml` (`MoveFileExW` + `MOVEFILE_WRITE_THROUGH` on Windows), (6) fsync the folder, (7) in one SQLite transaction bump `config_rev` and update `manifest_hash`, `summary_json` and `event_journal`. No fsync ever runs on a UI thread or under a global lock.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Runtime/r3/xml.cpp:2334-2375 (XmlFileWriter -tmp/-prev)`

#### Scenario: Acceptance for FR-CP02-005
- **WHEN** the FR-CP02-005 acceptance check is run
- **THEN** 10,000 kill -9 / simulated power cuts at each of steps (1)-(7) always leave a valid `machine.toml` (new or old) after restart, with the registry consistent after reconcile.

### Requirement: VM folder lock (FR-CP02-006)
Virtax SHALL provide "VM folder lock" as specified by FR-CP02-006 (BRD-03, priority Must, phase P1; Vietnamese: "Khoá thư mục VM").

#### Scenario: Specified behaviour of FR-CP02-006
- **WHEN** the behaviour FR-CP02-006 covers is exercised
- **THEN** it behaves as BRD-03 specifies: `virtaxd` holds an exclusive advisory lock (`flock` / `LockFileEx`) on the `.virtax.lock` of every VM it manages; the file records {host, user, pid, since}. A second daemon (another user, or another machine over a shared drive) gets `VTX-CFG-030` "VM in use by <user>@<host>". Dead locks (pid gone on the same host) are reclaimed automatically; a different host needs an explicit "Take over". VM folders on NFS/SMB get a warning because advisory locks are unreliable there.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Runtime/r3/xml.cpp:232-246 (RTFILE_O_DENY_NONE, no file locks)`

#### Scenario: Acceptance for FR-CP02-006
- **WHEN** the FR-CP02-006 acceptance check is run
- **THEN** Two daemons (2 users) opening the same VM: only one can write and the other shows a clear error; killing and restarting the daemon reclaims the lock within ≤ 1 s with no user action.

### Requirement: etag / `config_rev` optimistic concurrency (FR-CP02-007)
Virtax SHALL provide "etag / `config_rev` optimistic concurrency" as specified by FR-CP02-007 (BRD-03, priority Must, phase P1; Vietnamese: "etag / `config_rev` cho optimistic concurrency").

#### Scenario: Specified behaviour of FR-CP02-007
- **WHEN** the behaviour FR-CP02-007 covers is exercised
- **THEN** it behaves as BRD-03 specifies: `config_rev` increases monotonically per VM and backs the `etag`. Every `UpdateMachine` must send the etag; a mismatch returns `ABORTED` with the current etag and the changed fields. For legacy VMs `config_rev` is bumped on `OnMachineDataChanged` (including changes from the VirtualBox GUI or VBoxManage), so stale edits cannot overwrite.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:11934-12190 (commit/rollback, no versioning)`

#### Scenario: Acceptance for FR-CP02-007
- **WHEN** the FR-CP02-007 acceptance check is run
- **THEN** Two clients editing with the same old etag: exactly 1 succeeds and 1 gets `ABORTED` with the field list; a VBoxManage change alters the etag within ≤ 1 s.

### Requirement: Field-mask updates with dry run and per-field ApplyMode (FR-CP02-008)
Virtax SHALL provide "Field-mask updates with dry run and per-field ApplyMode" as specified by FR-CP02-008 (BRD-03, priority Must, phase P1; Vietnamese: "Update theo field mask, có dry-run và ApplyMode từng field").

#### Scenario: Specified behaviour of FR-CP02-008
- **WHEN** the behaviour FR-CP02-008 covers is exercised
- **THEN** it behaves as BRD-03 specifies: `UpdateMachine(machine, update_mask, etag, validate_only, apply_policy)` touches only masked fields. The daemon asks `Engine.apply` for the `ApplyMode` (Live | NextBoot | NeedsPowerOff) and returns a `FieldResult{path, apply_mode, issues[]}` per field. `validate_only=true` feeds badges and diffs and writes nothing. On the legacy engine, mutations run on the VM's actor: Lock → set N attributes → `saveSettings` → Unlock, with a 10 s deadline.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:9864-9926 (i_saveSettings full rewrite)`

#### Scenario: Acceptance for FR-CP02-008
- **WHEN** the FR-CP02-008 acceptance check is run
- **THEN** A dry run returns ApplyMode for 100% of fields in ≤ 20 ms; a 10-field legacy update uses exactly 1 `saveSettings`; a hung VBoxSVC yields deadline error `VTX-CFG-040` after 10 s with no UI hang.

### Requirement: `pending_changes` queue applied at power-off (FR-CP02-009)
Virtax SHALL provide "`pending_changes` queue applied at power-off" as specified by FR-CP02-009 (BRD-03, priority Must, phase P1; Vietnamese: "Hàng đợi `pending_changes` áp khi tắt máy").

#### Scenario: Specified behaviour of FR-CP02-009
- **WHEN** the behaviour FR-CP02-009 covers is exercised
- **THEN** it behaves as BRD-03 specifies: NextBoot/NeedsPowerOff changes on a running VM are stored in `pending_changes(machine_id, field_path, value_json, apply_mode, created_at)` and applied automatically when the daemon sees PoweredOff, before the next start; clashes with newer edits are resolved by etag. Overview and virtaxctl list pending items and users can cancel each one.
- **AND** the VirtualBox reference for the current behaviour is `new (VirtualBox rejects changes when ConfigurationAccessLevel is Partial_Running)`

#### Scenario: Acceptance for FR-CP02-009
- **WHEN** the FR-CP02-009 acceptance check is run
- **THEN** Queuing a RAM change and shutting down applies it within ≤ 2 s after PoweredOff and the next start uses it; a daemon restart in between loses no pending item.

### Requirement: Phase 1: shadow `.vbox`, write through VBoxSVC (FR-CP02-010)
Virtax SHALL provide "Phase 1: shadow `.vbox`, write through VBoxSVC" as specified by FR-CP02-010 (BRD-03, priority Must, phase P1; Vietnamese: "Pha 1: shadow `.vbox`, ghi qua VBoxSVC").

#### Scenario: Specified behaviour of FR-CP02-010
- **WHEN** the behaviour FR-CP02-010 covers is exercised
- **THEN** it behaves as BRD-03 specifies: For `vbox-legacy` VMs the `.vbox` is the source of truth; the daemon reads through VBoxCAPI, normalizes into the `Machine` DTO and stores only the summary + indexes in SQLite. Every write goes through `IMachine` setters + `saveSettings` (VM off) or the shared session (Live changes on a running VM); Virtax never edits `.vbox` files directly.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/VirtualBoxImpl.cpp:926-956 (initMachines, serial parse)`

#### Scenario: Acceptance for FR-CP02-010
- **WHEN** the FR-CP02-010 acceptance check is run
- **THEN** After 200 random Virtax edits, `VBoxManage showvminfo` matches the DTO 100% and the VirtualBox 7.2 GUI opens the VM without warnings.

### Requirement: Rust `.vbox` 1.3-1.22 reader and export-manifest (FR-CP02-011)
Virtax SHALL provide "Rust `.vbox` 1.3-1.22 reader and export-manifest" as specified by FR-CP02-011 (BRD-03, priority Should, phase P1; Vietnamese: "Bộ đọc `.vbox` 1.3-1.22 bằng Rust và export-manifest").

#### Scenario: Specified behaviour of FR-CP02-011
- **WHEN** the behaviour FR-CP02-011 covers is exercised
- **THEN** it behaves as BRD-03 specifies: A VBoxSVC-free `.vbox` reader (`quick-xml` + serde with types mirroring `settings.h`) for versions 1.3-1.22 (`Future` read with a warning), used by direct import (CP-09), the Health page and tests. `virtaxctl vm export-manifest <vm>` exports API → `machine.toml` for comparison; a round-trip suite runs on the `xml/samples` corpus + community files.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/xml/Settings.cpp:23-56,133 (v1.3-1.22 conditionals); include/VBox/settings.h`

#### Scenario: Acceptance for FR-CP02-011
- **WHEN** the FR-CP02-011 acceptance check is run
- **THEN** 100% of the corpus parses; API → TOML → API shows 0 semantic differences; the oldest 1.3 file opens and unsupported fields are listed in a report.

### Requirement: `.vbox` → `machine.toml` migration on "Move to virtax-vmm" (FR-CP02-012)
Virtax SHALL provide "`.vbox` → `machine.toml` migration on "Move to virtax-vmm"" as specified by FR-CP02-012 (BRD-03, priority Must, phase P2; Vietnamese: "Migrate `.vbox` → `machine.toml` khi "Move to virtax-vmm"").

#### Scenario: Specified behaviour of FR-CP02-012
- **WHEN** the behaviour FR-CP02-012 covers is exercised
- **THEN** it behaves as BRD-03 specifies: On opt-in the VM moves to `machine.toml` (the new source of truth): devices map to the virtio set, a compatibility report lists devices without an equivalent (IDE/AHCI, E1000, AC97…), disks are referenced by UUID + relative path; the `.vbox` is kept untouched for one-click rollback.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/xml/Settings.cpp:4566-10559 (MachineConfigFile)`

#### Scenario: Acceptance for FR-CP02-012
- **WHEN** the FR-CP02-012 acceptance check is run
- **THEN** Migrate then roll back leaves the `.vbox` checksum identical; a migrated Ubuntu VM boots on virtax-vmm with no blocking items left in the report.

### Requirement: `ui_prefs` separated from VMs, `GUI/*` blocked (FR-CP02-013)
Virtax SHALL provide "`ui_prefs` separated from VMs, `GUI/*` blocked" as specified by FR-CP02-013 (BRD-03, priority Must, phase P1; Vietnamese: "`ui_prefs` tách khỏi VM, chặn `GUI/*`").

#### Scenario: Specified behaviour of FR-CP02-013
- **WHEN** the behaviour FR-CP02-013 covers is exercised
- **THEN** it behaves as BRD-03 specifies: UI preferences (window geometry, last tab, splitters, column widths, visible cards, console mode/scale) live in `ui_prefs(scope, key, value)`, coalesced to ≤ 1 write/s and broadcast through `UiPrefs.Watch`. The API rejects any `GUI/*` extradata write (`VTX-CFG-050`); Virtax never writes `GUI/*` and leaves existing keys untouched.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/extradata/UIExtraDataManager.cpp:363-400; MachineImpl.cpp:4586`

#### Scenario: Acceptance for FR-CP02-013
- **WHEN** the FR-CP02-013 acceptance check is run
- **THEN** A week of Virtax use on 10 legacy VMs adds no new `GUI/*` keys to the `.vbox` diffs; dragging a window for 10 s causes ≤ 10 `ui_prefs` writes.

### Requirement: Controlled `VBoxInternal/*` escape hatch (FR-CP02-014)
Virtax SHALL provide "Controlled `VBoxInternal/*` escape hatch" as specified by FR-CP02-014 (BRD-03, priority Should, phase P1; Vietnamese: "Escape hatch `VBoxInternal/*` có kiểm soát").

#### Scenario: Specified behaviour of FR-CP02-014
- **WHEN** the behaviour FR-CP02-014 covers is exercised
- **THEN** it behaves as BRD-03 specifies: `[engine.vbox_legacy] extradata` (API: `Machine.engine_overrides`) applies to the legacy engine only. The daemon sets a safe knob set for Virtax-created VMs (e.g. `/TM/TSCMode=RealTSCOffset` when the host has an invariant TSC); keys outside the allow-list need `--force`; every override shows in diffs and the audit log and is dropped (with a warning) when moving to virtax-vmm.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/ConsoleImplConfigCommon.cpp:695-760`

#### Scenario: Acceptance for FR-CP02-014
- **WHEN** the FR-CP02-014 acceptance check is run
- **THEN** `virtaxctl vm set --override VBoxInternal/X=1` without `--force` is refused; a new Virtax-created VM shows exactly the default knob set in `VBoxManage getextradata`.

### Requirement: Startup reconcile, recovery and backups (FR-CP02-015)
Virtax SHALL provide "Startup reconcile, recovery and backups" as specified by FR-CP02-015 (BRD-03, priority Must, phase P1; Vietnamese: "Reconcile khi khởi động, khôi phục và backup").

#### Scenario: Specified behaviour of FR-CP02-015
- **WHEN** the behaviour FR-CP02-015 covers is exercised
- **THEN** it behaves as BRD-03 specifies: At startup the daemon compares the registry with manifests (mtime + `manifest_hash` + `config_rev`): a newer manifest is re-imported; a corrupt one is restored from `.prev`, the broken copy kept as `machine.toml.corrupt-<ts>` and the user notified; a missing folder marks the VM "needs attention" and is never deleted automatically. The registry is backed up daily with `VACUUM INTO`, keeping 7 copies.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/VirtualBoxImpl.cpp:742-749,926-956`

#### Scenario: Acceptance for FR-CP02-015
- **WHEN** the FR-CP02-015 acceptance check is run
- **THEN** Reconciling 500 VMs takes ≤ 2 s; a truncated manifest comes back from `.prev` with a notice; 0 VMs are removed from the registry because of read errors.

### Requirement: One manifest file per snapshot (FR-CP02-016)
Virtax SHALL provide "One manifest file per snapshot" as specified by FR-CP02-016 (BRD-03, priority Could, phase P2; Vietnamese: "Mỗi snapshot một file manifest").

#### Scenario: Specified behaviour of FR-CP02-016
- **WHEN** the behaviour FR-CP02-016 covers is exercised
- **THEN** it behaves as BRD-03 specifies: For virtax-vmm VMs each snapshot is `snapshots/<uuid>.toml` holding the frozen hardware + `parent`; the `snapshots` table is an index. Taking a snapshot writes one new file (O(1)) instead of rewriting the whole tree as `.vbox` does.
- **AND** the VirtualBox reference for the current behaviour is `include/VBox/settings.h:1465-1487 (full Hardware copy per snapshot)`

#### Scenario: Acceptance for FR-CP02-016
- **WHEN** the FR-CP02-016 acceptance check is run
- **THEN** With 100 snapshots, one more snapshot writes ≤ 1 new manifest file and ≤ 64 KiB; deleting a mid-tree snapshot fixes its children's `parent`.

### Requirement: As built — Engine overrides table
Engine-specific manifest keys SHALL live under `[engine_overrides.<engine>]`, because TOML cannot hold both `engine = "<kind>"` and an `[engine]` table; this is where FR-CP02-004 and FR-CP02-014 place `[engine.vbox_legacy]`. Source: `crates/virtax-model/src/manifest.rs`.

#### Scenario: Overrides and extensions round-trip
- **WHEN** a manifest with `[engine_overrides.vbox-legacy]` and an `x-` key is parsed and written back
- **THEN** both are preserved unchanged
- **AND** this is verified by `crates/virtax-model/src/manifest.rs::parses_sample_and_round_trips_extensions`

### Requirement: As built — Pending change queuing
On a running machine, `UpdateMachine` SHALL queue fields whose apply mode is NextBoot and SHALL queue NeedsPowerOff fields only when `apply_policy = APPLY_POLICY_QUEUE` (`virtaxctl vm set --at-shutdown`); otherwise it SHALL refuse the whole update with `VTX-RT-002` and queue nothing. The UI default (ask or queue) remains open question Q-03-01. Source: `apps/virtaxd/src/daemon.rs`.

#### Scenario: Refused without the queue policy
- **WHEN** a NeedsPowerOff field is updated on a running machine without `APPLY_POLICY_QUEUE`
- **THEN** the update fails with `VTX-RT-002` and nothing is queued
- **AND** this is verified by `apps/virtaxd/src/daemon.rs::power_off_changes_are_refused_without_queue_policy_and_can_be_cancelled`

### Requirement: As built — Applying pending changes
Queued changes SHALL be applied when the machine reaches PoweredOff or Aborted, at daemon startup for machines that are off, and before every start; a start SHALL fail rather than boot with stale settings when the queue cannot be applied. Source: `apps/virtaxd/src/pending.rs`.

#### Scenario: Applied after power-off
- **WHEN** queued changes exist and the machine powers off
- **THEN** they are applied within 2 s and removed from the queue
- **AND** this is verified by `apps/virtaxd/src/daemon.rs::queued_changes_apply_after_power_off`

#### Scenario: Kept across a daemon restart
- **WHEN** the daemon restarts while changes are queued and the machine is off
- **THEN** the changes are applied during startup reconcile
- **AND** this is verified by `apps/virtaxd/src/daemon.rs::queued_changes_survive_restart_and_apply_at_startup`

### Requirement: As built — Mock engine kind
The registry and manifest SHALL accept the engine kind `mock` for the deterministic MockEngine used in development and tests (FR-PL01-011); it never runs guest code. Source: `crates/virtax-store/src/registry.rs`.

#### Scenario: Machines on the mock engine
- **WHEN** a machine is created, started, updated and deleted on the mock engine through `virtaxctl`
- **THEN** every step succeeds with the documented exit codes
- **AND** this is verified by `apps/virtaxctl/tests/e2e.rs::machine_lifecycle_through_cli`
