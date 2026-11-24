# Quick Create & wizards Specification

## Purpose
One-screen create-from-ISO, OS detection, unattended by default; clone, import/export, new disk. Module `UI-03` (Quick Create & wizards; Vietnamese: "Quick Create & wizard") from BRD-02, phase P1, priority Must. Owner crates: virtax. Depends on: `control-plane/unattended-install-and-os-catalogue`, `control-plane/ovf-ova-appliances-and-virtualbox-migration`, `control-plane/storage-and-media-management`. VirtualBox reference: `src/VBox/Frontends/VirtualBox/src/wizards (newvm, clonevm, exportappliance, importappliance, newvd)`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: One-screen Quick Create (FR-UI03-001)
Virtax SHALL provide "One-screen Quick Create" as specified by FR-UI03-001 (BRD-02, priority Must, phase P1; Vietnamese: "Quick Create một màn hình").

#### Scenario: Specified behaviour of FR-UI03-001
- **WHEN** the behaviour FR-UI03-001 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Replace W-01 with one dialog: ISO drop zone (Browse…, ISO-library shortcuts), Name, Username, Password, Hostname, vCPU/Memory/Disk sliders, Install guest agent / Start after create / Install in background toggles, and Cancel · Customize… · Create & Start. An "Advanced: EFI · NVMe · virtio-net · TPM" line summarises hidden settings.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/wizards/UINativeWizard.cpp:99`

#### Scenario: Acceptance for FR-UI03-001
- **WHEN** the FR-UI03-001 acceptance check is run
- **THEN** Given an Ubuntu 24.04 ISO, When the user drops it, types a password and clicks Create & Start, Then the VM is created in exactly 3 required interactions; ≥ 80% of test users finish unaided.

### Requirement: Asynchronous ISO detection feedback (FR-UI03-002)
Virtax SHALL provide "Asynchronous ISO detection feedback" as specified by FR-UI03-002 (BRD-02, priority Must, phase P1; Vietnamese: "Phản hồi detect ISO bất đồng bộ").

#### Scenario: Specified behaviour of FR-UI03-002
- **WHEN** the behaviour FR-UI03-002 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Dropping an ISO shows its name, size and a "Scanning…" state at once, then "Ubuntu 24.04.1 LTS · x86_64" and "Unattended ✓" chips (or a reason). Users can type while scanning; results never overwrite edited fields; dropping another ISO cancels the previous Operation.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/wizards/newvm/UIWizardNewVM.cpp:628`

#### Scenario: Acceptance for FR-UI03-002
- **WHEN** the FR-UI03-002 acceptance check is run
- **THEN** Scanning a 6 GB ISO on a USB HDD keeps UI frames p99 ≤ 50 ms; on SSD the chip appears within 2 s for 95% of catalogue ISOs; a second ISO cancels the first scan within 1 s.

### Requirement: Host-aware recommended resources (FR-UI03-003)
Virtax SHALL provide "Host-aware recommended resources" as specified by FR-UI03-003 (BRD-02, priority Must, phase P1; Vietnamese: "Tài nguyên khuyến nghị theo host").

#### Scenario: Specified behaviour of FR-UI03-003
- **WHEN** the behaviour FR-UI03-003 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Sliders use `GuestOsTypes.Recommend` (FR-CP10-004) instead of fixed defaults (Ubuntu 1 vCPU/2 GB/25 GB). Helper labels show "host 12 cores", "412 GB free" and a "recommended" badge; exceeding 75% of host RAM or the logical CPU count shows an inline amber warning with the reason.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-all/Global.cpp:213-216,489`

#### Scenario: Acceptance for FR-UI03-003
- **WHEN** the FR-UI03-003 acceptance check is run
- **THEN** On a 12-CPU/32 GB host Ubuntu defaults to 4 vCPUs/8 GB/64 GB; dragging RAM to 28 GB warns within 100 ms; a fixed disk larger than free space is blocked.

### Requirement: Inline validation with Fix (FR-UI03-004)
Virtax SHALL provide "Inline validation with Fix" as specified by FR-UI03-004 (BRD-02, priority Must, phase P1; Vietnamese: "Validate inline có nút Fix").

#### Scenario: Specified behaviour of FR-UI03-004
- **WHEN** the behaviour FR-UI03-004 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Validate while typing (150 ms debounce): RFC 1123 hostnames, per-OS usernames (Windows reserved names; Linux `[a-z_][a-z0-9_-]*`), mandatory Windows password, unique VM name and folder. Errors sit under the field (colour + icon + text) with Fix when an automatic correction exists.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/wizards/editors/UIWizardNewVMEditors.cpp`

#### Scenario: Acceptance for FR-UI03-004
- **WHEN** the FR-UI03-004 acceptance check is run
- **THEN** Hostname "ubuntu dev" shows "no spaces" and Fix changes it to "ubuntu-dev" in one click; Create stays disabled until no errors remain.

### Requirement: Full Customize… form, single implementation (FR-UI03-005)
Virtax SHALL provide "Full Customize… form, single implementation" as specified by FR-UI03-005 (BRD-02, priority Must, phase P1; Vietnamese: "Form Customize… đầy đủ, một bản cài đặt").

#### Scenario: Specified behaviour of FR-UI03-005
- **WHEN** the behaviour FR-UI03-005 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Customize… opens a side sheet sharing Settings (UI-04) components: General, System, Storage, Network, Unattended (locale, time zone, keyboard, proxy, extra packages, post-install, edition), Advanced. No global Basic/Expert mode; values sync both ways with Quick Create.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/wizards/newvm/UIWizardNewVMExpertPage.cpp`

#### Scenario: Acceptance for FR-UI03-005
- **WHEN** the FR-UI03-005 acceptance check is run
- **THEN** Opens within 150 ms; the mapping table shows 100% of W-01 Basic and Expert fields present; switching the controller to SATA updates the Advanced line at once.

### Requirement: Windows edition and product key (FR-UI03-006)
Virtax SHALL provide "Windows edition and product key" as specified by FR-UI03-006 (BRD-02, priority Must, phase P1; Vietnamese: "Chọn edition Windows và product key").

#### Scenario: Specified behaviour of FR-UI03-006
- **WHEN** the behaviour FR-UI03-006 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Windows ISOs get an "Edition" dropdown from the `install.wim`/`.esd` image list, defaulting to Pro. The product key is optional (5×5 validation). Windows 11 VMs automatically get TPM 2.0, Secure Boot, ≥ 4 GB RAM and ≥ 64 GB disk so Setup's hardware checks pass without registry bypasses.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/UnattendedImpl.cpp:1115,4273-4290`

#### Scenario: Acceptance for FR-UI03-006
- **WHEN** the FR-UI03-006 acceptance check is run
- **THEN** A multi-edition Windows 11 24H2 ISO lists every edition with "Windows 11 Pro" preselected; the VM passes hardware checks (no `LabConfig`) on x64 and arm64.

### Requirement: Block mismatched-architecture ISOs (FR-UI03-007)
Virtax SHALL provide "Block mismatched-architecture ISOs" as specified by FR-UI03-007 (BRD-02, priority Must, phase P1; Vietnamese: "Chặn ISO khác kiến trúc").

#### Scenario: Specified behaviour of FR-UI03-007
- **WHEN** the behaviour FR-UI03-007 covers is exercised
- **THEN** it behaves as BRD-02 specifies: An ISO whose architecture differs from the host (amd64 on M-series Macs/Windows on ARM, arm64 on x64 PCs) shows error VTX-ISO-003 "Virtax only runs guests with this machine's architecture", a catalogue link to the right download and a "Choose another ISO" button; Create is disabled and no VM is created.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/UnattendedImpl.cpp:736-756`

#### Scenario: Acceptance for FR-UI03-007
- **WHEN** the FR-UI03-007 acceptance check is run
- **THEN** Dropping an amd64 Ubuntu ISO on an M2 Mac shows the message + arm64 link within 2 s with 0 files written; `virtaxctl vm create --iso` exits 5 with VTX-ISO-003.

### Requirement: Install progress in the Task center (FR-UI03-008)
Virtax SHALL provide "Install progress in the Task center" as specified by FR-UI03-008 (BRD-02, priority Must, phase P1; Vietnamese: "Tiến độ cài đặt trong Task center").

#### Scenario: Specified behaviour of FR-UI03-008
- **WHEN** the behaviour FR-UI03-008 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Create & Start closes the dialog within 300 ms and the VM appears at once ("Installing"); the Task center shows the Operation by step (disk → media → boot → installing n/5 → agent → ready) with % and ETA, Show console and Cancel; on completion a "Ubuntu is ready" toast appears and the Console opens.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/UIVirtualBoxManager.cpp:3033-3037`

#### Scenario: Acceptance for FR-UI03-008
- **WHEN** the FR-UI03-008 acceptance check is run
- **THEN** The VM appears within 500 ms; the Manager stays smooth during install (frame p99 ≤ 50 ms); Cancel asks "Delete the incomplete VM?" and cleans up.

### Requirement: Clone VM side sheet (FR-UI03-009)
Virtax SHALL provide "Clone VM side sheet" as specified by FR-UI03-009 (BRD-02, priority Must, phase P1; Vietnamese: "Clone VM dạng side sheet").

#### Scenario: Specified behaviour of FR-UI03-009
- **WHEN** the behaviour FR-UI03-009 covers is exercised
- **THEN** it behaves as BRD-02 specifies: W-02 (3 pages) becomes a side sheet: name, folder, Full/Linked, snapshot scope, MAC policy, keep disk names, keep hardware UUIDs. It defaults to Linked when the source has snapshots on the same volume, shows size/time estimates and runs via `Machines.Clone` (CP-04, BRD-05).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImplCloneVM.cpp:890-1182`

#### Scenario: Acceptance for FR-UI03-009
- **WHEN** the FR-UI03-009 acceptance check is run
- **THEN** A linked clone of a 40 GB VM takes ≤ 5 s; a full clone's ETA is within ±20% after 10%; insufficient space blocks before any write with VTX-VM-020.

### Requirement: Export appliance side sheet (FR-UI03-010)
Virtax SHALL provide "Export appliance side sheet" as specified by FR-UI03-010 (BRD-02, priority Should, phase P1; Vietnamese: "Export appliance dạng side sheet").

#### Scenario: Specified behaviour of FR-UI03-010
- **WHEN** the behaviour FR-UI03-010 covers is exercised
- **THEN** it behaves as BRD-02 specifies: W-03 becomes a side sheet: pick VMs, OVF 1.0 (default) or 2.0 (0.9 under Advanced), OVA or folder, SHA-256 manifest on, include ISOs, MAC policy, per-VM metadata. "Export to OCI" is dropped. Runs in the background with MB/s and ETA.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/wizards/exportappliance/`

#### Scenario: Acceptance for FR-UI03-010
- **WHEN** the FR-UI03-010 acceptance check is run
- **THEN** Exporting a 20 GB VM to OVA gives a stable ETA after 15 s, cancels within 2 s removing partial files; the OVA re-imports into VirtualBox 7.2 and Virtax.

### Requirement: Unified OVA/OVF/.vbox import (FR-UI03-011)
Virtax SHALL provide "Unified OVA/OVF/.vbox import" as specified by FR-UI03-011 (BRD-02, priority Must, phase P1; Vietnamese: "Import hợp nhất OVA/OVF/.vbox").

#### Scenario: Specified behaviour of FR-UI03-011
- **WHEN** the behaviour FR-UI03-011 covers is exercised
- **THEN** it behaves as BRD-02 specifies: One Import entry point accepts OVA, OVF, `.vbox` or a VM folder (drag-and-drop works). W-04's raw VSD tree becomes a summary card per VM (OS, CPU, RAM, disks, NICs) plus the compatibility report (FR-CP09-005); the EULA (W-04a) and signature (W-04b) are shown inline, not modally.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/wizards/importappliance/UIWizardImportApp.cpp:51`

#### Scenario: Acceptance for FR-UI03-011
- **WHEN** the FR-UI03-011 acceptance check is run
- **THEN** A 2-VM OVA with an EULA enables Import only after "I agree"; a self-signed signature shows an inline amber warning; modal depth ≤ 1.

### Requirement: Inline disk form, medium selector popover and VISO (FR-UI03-012)
Virtax SHALL provide "Inline disk form, medium selector popover and VISO" as specified by FR-UI03-012 (BRD-02, priority Should, phase P1; Vietnamese: "Form disk inline, medium selector popover và VISO").

#### Scenario: Specified behaviour of FR-UI03-012
- **WHEN** the behaviour FR-UI03-012 covers is exercised
- **THEN** it behaves as BRD-02 specifies: W-05 (3 pages) becomes an inline form (VDI/VMDK/VHD, qcow2 in Phase 2; dynamic/fixed; size beside free space) and W-08 a searchable popover (ISO library, disks, recent, inline create); W-09 becomes a VISO tool (drag host files → `.viso`, attached straight to the optical drive). Copy/convert is an Operation with ETA. Modal depth ≤ 2 (VirtualBox: 4-5).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/medium/UIMediumSelector.cpp`

#### Scenario: Acceptance for FR-UI03-012
- **WHEN** the FR-UI03-012 acceptance check is run
- **THEN** A 64 GB dynamic disk is created within 1 s; attaching a new ISO from Settings › Storage takes ≤ 3 interactions; a 500-file VISO builds within 1 s with correct guest checksums; modal depth ≤ 2.

### Requirement: Keyboard and accessibility for VM creation (FR-UI03-013)
Virtax SHALL provide "Keyboard and accessibility for VM creation" as specified by FR-UI03-013 (BRD-02, priority Must, phase P1; Vietnamese: "Bàn phím và accessibility cho luồng tạo VM").

#### Scenario: Specified behaviour of FR-UI03-013
- **WHEN** the behaviour FR-UI03-013 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Quick Create and all side sheets are 100% keyboard operable (Enter on the drop zone = Browse…) with an always-visible #60A5FA focus ring; chips/sliders/toggles have accessible names and sliders announce values with units; detection results and errors use a live region; targets ≥ 32 px, contrast ≥ 4.5:1.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIAccessible.cpp`

#### Scenario: Acceptance for FR-UI03-013
- **WHEN** the FR-UI03-013 acceptance check is run
- **THEN** A keyboard-only creation script completes with NVDA, VoiceOver and Orca; the automated accessibility audit reports 0 critical issues.
