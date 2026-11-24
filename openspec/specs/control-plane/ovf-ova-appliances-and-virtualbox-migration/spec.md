# OVF/OVA appliances & VirtualBox migration Specification

## Purpose
OVA 1.0/2.0 import/export, direct .vbox import (no VBoxSVC), device-compatibility report. Module `CP-09` (OVF/OVA appliances & VirtualBox migration; Vietnamese: "Appliance OVF/OVA & migrate từ VirtualBox") from BRD-02, phase P1, priority Should. Owner crates: virtaxd (importers). Depends on: `control-plane/registry-and-configuration-store`, `control-plane/storage-and-media-management`. VirtualBox reference: `src/VBox/Main/src-server/ApplianceImpl*.cpp, xml/ovfreader.cpp`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Appliance inspect and import, with EULA/metadata (FR-CP09-001)
Virtax SHALL provide "Appliance inspect and import, with EULA/metadata" as specified by FR-CP09-001 (BRD-02, priority Must, phase P1; Vietnamese: "Inspect và import appliance, kèm EULA/metadata").

#### Scenario: Specified behaviour of FR-CP09-001
- **WHEN** the behaviour FR-CP09-001 covers is exercised
- **THEN** it behaves as BRD-02 specifies: `Appliances.Inspect(path)` returns an `ApplianceSummary` (virtual systems, disks, NICs, EULA, product/vendor/version metadata, manifest/signature status); `Appliances.Import(path, options)` creates the VMs. Phase 1 calls `IAppliance::read → interpret → importMachines` on the FFI pool with progress via `Operations.Watch` instead of polling `IProgress`. EULAs must be accepted.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/ApplianceImplImport.cpp:88-1217`

#### Scenario: Acceptance for FR-CP09-001
- **WHEN** the FR-CP09-001 acceptance check is run
- **THEN** Inspecting a 10 GB OVA takes ≤ 2 s (no disk reads); import updates % and MB/s at least once per second; cancel ≤ 2 s with cleanup; a 2-EULA OVA needs both accepted.

### Requirement: OVF versions, manifests and signatures (FR-CP09-002)
Virtax SHALL provide "OVF versions, manifests and signatures" as specified by FR-CP09-002 (BRD-02, priority Must, phase P1; Vietnamese: "Phiên bản OVF, manifest và chữ ký").

#### Scenario: Specified behaviour of FR-CP09-002
- **WHEN** the behaviour FR-CP09-002 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Import OVF 0.9/1.0/2.0 and OVA; verify the `.mf` manifest (SHA-1 < 2.0, SHA-256 for 2.0, SHA-512 accepted) and the `.cert` signature (X.509/PKCS#7) against the OS trust store, showing signer/validity in the import card (replacing W-04b). Export defaults to OVF 1.0 + SHA-256 manifest (Q-02-03); `virtaxctl appliance sign` mirrors `VBoxManage signova`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/ApplianceImplImport.cpp:3172-3655,3964`

#### Scenario: Acceptance for FR-CP09-002
- **WHEN** the FR-CP09-002 acceptance check is run
- **THEN** ≥ 95% of an OVA corpus (VirtualBox 6.1/7.x, VMware Workstation 17, ESXi 8) imports; a one-byte disk change gives VTX-OVF-004 and no registration; a bad signature gives VTX-OVF-006, self-signed an amber warning.

### Requirement: Low-impact export pipeline (FR-CP09-003)
Virtax SHALL provide "Low-impact export pipeline" as specified by FR-CP09-003 (BRD-02, priority Should, phase P1; Vietnamese: "Pipeline export ít tác động").

#### Scenario: Specified behaviour of FR-CP09-003
- **WHEN** the behaviour FR-CP09-003 covers is exercised
- **THEN** it behaves as BRD-02 specifies: `Appliances.Export` streams streamOptimized VMDK straight into the OVA tar (no disk-sized temp file) and computes the manifest while writing; I/O runs at background priority (`ioprio` IDLE on Linux, `PROCESS_MODE_BACKGROUND_BEGIN` on Windows, `IOPOL_THROTTLE` on macOS) so running VMs do not stutter.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/ApplianceImplExport.cpp:2371-2443`

#### Scenario: Acceptance for FR-CP09-003
- **WHEN** the FR-CP09-003 acceptance check is run
- **THEN** Exporting a 20 GB VM uses ≤ 64 MiB extra temp space; throughput ≥ 80% of sequential write speed when idle; other VMs see ≤ 20% higher p95 disk latency.

### Requirement: Pre-flight checks and conflict handling (FR-CP09-004)
Virtax SHALL provide "Pre-flight checks and conflict handling" as specified by FR-CP09-004 (BRD-02, priority Must, phase P1; Vietnamese: "Pre-flight check và xử lý xung đột").

#### Scenario: Specified behaviour of FR-CP09-004
- **WHEN** the behaviour FR-CP09-004 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Before import/export/convert, check free space, write access, Windows long paths, name and machine/disk UUID clashes and guest architecture, shown as one checklist with fixes. UUID clashes ask once per batch: copy (new UUIDs) / skip / replace (listing exactly what is overwritten, confirmed twice).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/ApplianceImplImport.cpp:5553-5560`

#### Scenario: Acceptance for FR-CP09-004
- **WHEN** the FR-CP09-004 acceptance check is run
- **THEN** 10 GB free for a 30 GB OVA gives VTX-OVF-002 "Needs 30 GB, 10 GB free" with 0 bytes written; re-importing the same OVA defaults to `<name> (2)` with new UUIDs and MACs.

### Requirement: Device-compatibility report (FR-CP09-005)
Virtax SHALL provide "Device-compatibility report" as specified by FR-CP09-005 (BRD-02, priority Must, phase P1; Vietnamese: "Báo cáo tương thích thiết bị").

#### Scenario: Specified behaviour of FR-CP09-005
- **WHEN** the behaviour FR-CP09-005 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Each VM imported from an OVA or `.vbox` gets a per-device report: Supported / Mapped (LsiLogic → AHCI, vmxnet3 → E1000) / Unsupported on engine (VRDE, USB 2/3 needing the Extension Pack, 3D on `virtax-vmm`) / Blocked (architecture); covering the current engine and "Move to virtax-vmm" readiness (Phase 2); exportable as JSON/Markdown.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/include/VBox/settings.h:1166-1419`

#### Scenario: Acceptance for FR-CP09-005
- **WHEN** the FR-CP09-005 acceptance check is run
- **THEN** A 50-file `.vbox` corpus yields reports within 1 s per VM with no device missed versus `settings.h`; every Unsupported item has an explanation + remedy.

### Requirement: Direct .vbox reader without VBoxSVC (FR-CP09-006)
Virtax SHALL provide "Direct .vbox reader without VBoxSVC" as specified by FR-CP09-006 (BRD-02, priority Must, phase P1; Vietnamese: "Bộ đọc .vbox trực tiếp, không cần VBoxSVC").

#### Scenario: Specified behaviour of FR-CP09-006
- **WHEN** the behaviour FR-CP09-006 covers is exercised
- **THEN** it behaves as BRD-02 specifies: A Rust crate reads `.vbox`/`VirtualBox.xml` settings 1.3-1.22 with `quick-xml` + `serde` (types mirroring `settings.h`) without VBoxSVC; used for previews, compatibility reports, discovery and (Phase 2) `machine.toml` conversion. Read-only; it never writes `.vbox`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/xml/Settings.cpp`

#### Scenario: Acceptance for FR-CP09-006
- **WHEN** the FR-CP09-006 acceptance check is run
- **THEN** Against VBoxSVC DTOs on a 1.3-1.22 corpus: 0 semantic differences in supported fields; parsing a 50-snapshot `.vbox` takes ≤ 20 ms.

### Requirement: Discover and import existing VirtualBox VMs (FR-CP09-007)
Virtax SHALL provide "Discover and import existing VirtualBox VMs" as specified by FR-CP09-007 (BRD-02, priority Must, phase P1; Vietnamese: "Phát hiện và import VM VirtualBox có sẵn").

#### Scenario: Specified behaviour of FR-CP09-007
- **WHEN** the behaviour FR-CP09-007 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Read the MachineRegistry in `VirtualBox.xml` plus the default machine folder and list existing VMs (state, size, snapshots, encryption, compatibility); multi-select to register them in place with the legacy engine (`openMachine` + `registerMachine`) without copying, shadowing the read model into SQLite and writing no `GUI/*`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/VirtualBoxImpl.cpp:926-956`

#### Scenario: Acceptance for FR-CP09-007
- **WHEN** the FR-CP09-007 acceptance check is run
- **THEN** 12 VMs are discovered within 2 s and registered within 10 s; the VirtualBox GUI still opens them (FR-GEN-005); a VM locked elsewhere is reported clearly, not as a raw COM error.

### Requirement: Convert VMs to machine.toml for virtax-vmm (FR-CP09-008)
Virtax SHALL provide "Convert VMs to machine.toml for virtax-vmm" as specified by FR-CP09-008 (BRD-02, priority Should, phase P2; Vietnamese: "Chuyển VM sang machine.toml cho virtax-vmm").

#### Scenario: Specified behaviour of FR-CP09-008
- **WHEN** the behaviour FR-CP09-008 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Convert a legacy VM or OVA to `machine.toml` without VBoxSVC: AHCI/IDE → NVMe/virtio-blk, E1000 → virtio-net (virtio-win warning for Windows), VMSVGA → virtio-gpu 2D; the original `.vbox` stays for rollback; VDI disks are used in place or converted to qcow2.
- **AND** the VirtualBox reference for the current behaviour is `new (analysis 02 §4.7)`

#### Scenario: Acceptance for FR-CP09-008
- **WHEN** the FR-CP09-008 acceptance check is run
- **THEN** Ubuntu 24.04 and Windows 11 convert within 30 s (excluding disk conversion) and boot on `virtax-vmm`; a one-click rollback restores a working legacy VM.

### Requirement: Disk format conversion on import (FR-CP09-009)
Virtax SHALL provide "Disk format conversion on import" as specified by FR-CP09-009 (BRD-02, priority Should, phase P1; Vietnamese: "Chuyển định dạng disk khi import").

#### Scenario: Specified behaviour of FR-CP09-009
- **WHEN** the behaviour FR-CP09-009 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Keep streamOptimized VMDK or convert to VDI (legacy engine, matching `ImportOptions_ImportToVDI`) and sparse qcow2/raw (Phase 2); only allocated blocks are written, in the background with ETA.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/ApplianceImplImport.cpp:4242`

#### Scenario: Acceptance for FR-CP09-009
- **WHEN** the FR-CP09-009 acceptance check is run
- **THEN** A 20 GB logical disk holding 6 GB of data yields a target ≤ 6.5 GB in ≤ 1.3× the time of a sequential 6 GB read on the same drive.

### Requirement: virtaxctl parity for import/export (FR-CP09-010)
Virtax SHALL provide "virtaxctl parity for import/export" as specified by FR-CP09-010 (BRD-02, priority Must, phase P1; Vietnamese: "Lệnh virtaxctl tương đương import/export").

#### Scenario: Specified behaviour of FR-CP09-010
- **WHEN** the behaviour FR-CP09-010 covers is exercised
- **THEN** it behaves as BRD-02 specifies: `virtaxctl appliance inspect|import|export` and `virtaxctl vm discover|import-vbox` mirror `VBoxManage import` (`-n`, `--vsys`, `--options keepallmacs|keepnatmacs|importtovdi`), `export` (`--ovf10|--ovf20`, `--manifest`, `--iso`) and `registervm`, with `--json` and `--no-wait`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManageAppliance.cpp`

#### Scenario: Acceptance for FR-CP09-010
- **WHEN** the FR-CP09-010 acceptance check is run
- **THEN** 20 VBoxManage import/export scenarios give equivalent results; `virtaxctl appliance inspect --json` validates against the published JSON schema.

### Requirement: Importing encrypted VMs (FR-CP09-011)
Virtax SHALL provide "Importing encrypted VMs" as specified by FR-CP09-011 (BRD-02, priority Could, phase P1; Vietnamese: "Import VM đã mã hoá").

#### Scenario: Specified behaviour of FR-CP09-011
- **WHEN** the behaviour FR-CP09-011 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Fully encrypted VirtualBox VMs (AES-XTS) are flagged in the report; Phase 1 registers them through the legacy engine and asks for the password once, optionally saving it to the OS keystore (CP-12, BRD-11); Phase 2 reports "not yet supported on virtax-vmm".
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:16337`

#### Scenario: Acceptance for FR-CP09-011
- **WHEN** the FR-CP09-011 acceptance check is run
- **THEN** An encrypted VM imports and starts after the password is entered; the password never appears in logs or the registry.
