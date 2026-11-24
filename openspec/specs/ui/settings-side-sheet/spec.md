# Settings side sheet Specification

## Purpose
Searchable settings, inline validation, Live/Next boot/Power-off badges, diff review before Apply. Module `UI-04` (Settings side sheet; Vietnamese: "Settings dạng side sheet") from BRD-03, phase P1, priority Must. Owner crates: virtax. Depends on: `control-plane/registry-and-configuration-store`, `control-plane/control-plane-api-and-ipc`. VirtualBox reference: `src/VBox/Frontends/VirtualBox/src/settings (global, machine, editors)`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Non-modal side sheet reachable from anywhere (FR-UI04-001)
Virtax SHALL provide "Non-modal side sheet reachable from anywhere" as specified by FR-UI04-001 (BRD-03, priority Must, phase P1; Vietnamese: "Side sheet không modal, mở từ mọi nơi").

#### Scenario: Specified behaviour of FR-UI04-001
- **WHEN** the behaviour FR-UI04-001 covers is exercised
- **THEN** it behaves as BRD-03 specifies: VM settings are a right-side sheet (≥ 720 px wide; full window below 1,100 px) with the workspace still visible and VM state still live behind it. It opens from a card's Edit button, the palette or the `virtax://vm/<id>/settings/<page>#<field>` deep link; one sheet per VM; Esc closes it, asking first if unapplied changes remain. It also works while the VM runs (replacing R-19, opened modally from the runtime window).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/UIAdvancedSettingsDialog.cpp:1571-1620`

#### Scenario: Acceptance for FR-UI04-001
- **WHEN** the FR-UI04-001 acceptance check is run
- **THEN** Edit shows the sheet on the right page in ≤ 150 ms with the linked field focused; while it is open, the VM list and header keep updating live.

### Requirement: Search as the primary entry point (FR-UI04-002)
Virtax SHALL provide "Search as the primary entry point" as specified by FR-UI04-002 (BRD-03, priority Must, phase P1; Vietnamese: "Search là lối vào chính").

#### Scenario: Specified behaviour of FR-UI04-002
- **WHEN** the behaviour FR-UI04-002 covers is exercised
- **THEN** it behaves as BRD-03 specifies: The search box at the top of the sheet (`/` or Ctrl/⌘-Shift-P) matches field labels, VI/EN synonyms, help text and current values (e.g. "trim", "port forward", "cổng", "bộ nhớ tạm"); results are grouped by page, Enter jumps to and highlights the field, and collapsed advanced fields are found too.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/UIAdvancedSettingsDialog.cpp:1040 (filter)`

#### Scenario: Acceptance for FR-UI04-002
- **WHEN** the FR-UI04-002 acceptance check is run
- **THEN** ≤ 30 ms per keystroke; 20 VI/EN test queries (including unaccented) all return the right field in the top 3; Enter expands advanced sections when needed.

### Requirement: Machine page set replacing S-M1…S-M10 (FR-UI04-003)
Virtax SHALL provide "Machine page set replacing S-M1…S-M10" as specified by FR-UI04-003 (BRD-03, priority Must, phase P1; Vietnamese: "Bộ trang máy thay S-M1…S-M10").

#### Scenario: Specified behaviour of FR-UI04-003
- **WHEN** the behaviour FR-UI04-003 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Pages: General (name, OS, groups, tags, description), System (motherboard, CPU, acceleration), Display, Storage, Network, Audio, Ports (serial/parallel), USB, Shared folders, Integration (clipboard, drag-and-drop, guest agent) and Security (encryption, TPM, Secure Boot). The User Interface page (S-M10) becomes Console preferences stored in `ui_prefs`. The full mapping is in the additional content.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/UIAdvancedSettingsDialogSpecific.cpp:640-740`

#### Scenario: Acceptance for FR-UI04-003
- **WHEN** the FR-UI04-003 acceptance check is run
- **THEN** 100% of VirtualBox's 61 editors map to a field or are explicitly marked dropped/P2 in the mapping table; every field has its own deep link.

### Requirement: Per-field Live / Next boot / Power off badges (FR-UI04-004)
Virtax SHALL provide "Per-field Live / Next boot / Power off badges" as specified by FR-UI04-004 (BRD-03, priority Must, phase P1; Vietnamese: "Badge Live / Next boot / Power off theo từng field").

#### Scenario: Specified behaviour of FR-UI04-004
- **WHEN** the behaviour FR-UI04-004 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Each field shows a badge from the engine `ApplyMode` via `UpdateMachine(validate_only=true)`, recomputed when the VM state changes. **Live**: applied to the running VM now. **Next boot**: saved now (or queued for legacy VMs) and applied automatically at the next start. **Power off**: boot-critical hardware that needs the VM off. For a Saved VM, fields that would discard the saved state get their own warning. Locked fields always show a reason + action.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/UISettingsDefs.h:50-62; UIAdvancedSettingsDialog.cpp:1519`

#### Scenario: Acceptance for FR-UI04-004
- **WHEN** the FR-UI04-004 acceptance check is run
- **THEN** On a running VM, 100% of fields show one of the 3 badges matching the ApplyMode rule table (contract-tested against VirtualBox 7.2); a Running → Off transition updates badges within ≤ 250 ms.

### Requirement: Inline validation with Fix (FR-UI04-005)
Virtax SHALL provide "Inline validation with Fix" as specified by FR-UI04-005 (BRD-03, priority Must, phase P1; Vietnamese: "Validate inline có nút Fix").

#### Scenario: Specified behaviour of FR-UI04-005
- **WHEN** the behaviour FR-UI04-005 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Validation runs while typing (client-side JSON Schema + daemon semantic checks); errors appear right under the field with an `VTX-CFG-NNN` code and a bilingual sentence, plus a Fix button when a fix exists (port 2222 in use → 2223; RAM > 75% of host → suggested value; too little VRAM for 2 monitors). The footer summarises errors and focuses the field on click; Apply is disabled while blocking errors remain.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/UIAdvancedSettingsDialog.cpp:1655 (UISettingsWarningPane)`

#### Scenario: Acceptance for FR-UI04-005
- **WHEN** the FR-UI04-005 acceptance check is run
- **THEN** Errors appear ≤ 100 ms after typing; a port-forward clash between 2 VMs is caught before Apply; screen readers announce errors via a live region; 0 invalid configurations are saved.

### Requirement: Diff review before Apply, with queuing (FR-UI04-006)
Virtax SHALL provide "Diff review before Apply, with queuing" as specified by FR-UI04-006 (BRD-03, priority Must, phase P1; Vietnamese: "Xem diff trước khi Apply, có hàng đợi").

#### Scenario: Specified behaviour of FR-UI04-006
- **WHEN** the behaviour FR-UI04-006 covers is exercised
- **THEN** it behaves as BRD-03 specifies: The footer always shows "N changes · x next boot · y need power off". Review diff lists old → new grouped by ApplyMode with the equivalent `virtaxctl` command. Apply sends one `UpdateMachine(update_mask, etag)`. With Power-off fields on a running VM it offers "Shut down & apply", "Apply at shutdown" (stored as `pending_changes`) or "Cancel". Discard restores the original values.
- **AND** the VirtualBox reference for the current behaviour is `new (VirtualBox applies all-or-nothing in UISettingsSerializer.cpp:239-257)`

#### Scenario: Acceptance for FR-UI04-006
- **WHEN** the FR-UI04-006 acceptance check is run
- **THEN** 3 changes (1 Live, 1 Next boot, 1 Power off) show exactly 3 diff rows; choosing "Apply at shutdown" applies the Live one now and the other 2 within ≤ 2 s after the VM reaches PoweredOff.

### Requirement: "Changed elsewhere" with per-field merge (FR-UI04-007)
Virtax SHALL provide ""Changed elsewhere" with per-field merge" as specified by FR-UI04-007 (BRD-03, priority Must, phase P1; Vietnamese: ""Đã thay đổi ở nơi khác" và gộp theo field").

#### Scenario: Specified behaviour of FR-UI04-007
- **WHEN** the behaviour FR-UI04-007 covers is exercised
- **THEN** it behaves as BRD-03 specifies: The sheet keeps the etag (`config_rev`) from when it opened and holds no lock while editing. If `virtaxctl`, another window or the VirtualBox GUI changes the VM, a "Changed elsewhere" banner appears; non-overlapping fields merge automatically and conflicting fields offer "Keep mine / Take theirs".
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:2649-3043 (lockMachine)`

#### Scenario: Acceptance for FR-UI04-007
- **WHEN** the FR-UI04-007 acceptance check is run
- **THEN** Changing RAM via `virtaxctl` while the sheet edits CPU shows the banner within ≤ 1 s, and Apply sends CPU with the new etag and keeps the new RAM; nothing is ever silently overwritten.

### Requirement: Dynamic NIC list with inline port forwarding (FR-UI04-008)
Virtax SHALL provide "Dynamic NIC list with inline port forwarding" as specified by FR-UI04-008 (BRD-03, priority Must, phase P1; Vietnamese: "Danh sách NIC động, port forwarding inline").

#### Scenario: Specified behaviour of FR-UI04-008
- **WHEN** the behaviour FR-UI04-008 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Network is a list of NIC cards that can be added/removed up to the engine/chipset limit (PIIX3 8, ICH9 36, ARMv8 64); each card has model, attachment (NAT, bridged, host-only, internal, NAT network), MAC (regenerate), cable and an inline port-forwarding table (replacing dialog S-X1) that checks clashes with other VMs and busy host ports.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/machine/UIMachineSettingsNetwork.cpp:455`

#### Scenario: Acceptance for FR-UI04-008
- **WHEN** the FR-UI04-008 acceptance check is run
- **THEN** An ICH9 VM can add NICs 5-8 without VBoxManage; a rule clashing on a host port shows an inline error + Fix within ≤ 100 ms; changing the attachment on a running VM shows a Live badge.

### Requirement: Dynamic serial/parallel port list (FR-UI04-009)
Virtax SHALL provide "Dynamic serial/parallel port list" as specified by FR-UI04-009 (BRD-03, priority Should, phase P1; Vietnamese: "Danh sách cổng serial/parallel động").

#### Scenario: Specified behaviour of FR-UI04-009
- **WHEN** the behaviour FR-UI04-009 covers is exercised
- **THEN** it behaves as BRD-03 specifies: The Ports page adds/removes ports up to the engine limit (4 serial on the legacy engine), with Disconnected / Host device / Pipe / File / TCP modes and path/port checks; TCP and pipe modes offer a copyable connect command (e.g. `socat`, `nc`).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/machine/UIMachineSettingsSerial.cpp:312`

#### Scenario: Acceptance for FR-UI04-009
- **WHEN** the FR-UI04-009 acceptance check is run
- **THEN** Adding a 5th port on the legacy engine disables Add with the reason "Engine supports at most 4"; a malformed pipe path shows an inline error.

### Requirement: Storage page: controller tree and inline medium picker (FR-UI04-010)
Virtax SHALL provide "Storage page: controller tree and inline medium picker" as specified by FR-UI04-010 (BRD-03, priority Must, phase P1; Vietnamese: "Trang Storage: cây controller và chọn medium inline").

#### Scenario: Specified behaviour of FR-UI04-010
- **WHEN** the behaviour FR-UI04-010 covers is exercised
- **THEN** it behaves as BRD-03 specifies: A drag-and-drop controller → attachment tree; an Add device menu (new/existing disk, optical, floppy); inline medium picking from the Media library (BRD-06) instead of the modal Medium Selector; attachment properties (SSD, discard, hot-plug, cache) and an info panel (format, virtual/actual size, diff chain, other VMs using it).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/editors/UIStorageSettingsEditor.cpp (≈5.4k LOC)`

#### Scenario: Acceptance for FR-UI04-010
- **WHEN** the FR-UI04-010 acceptance check is run
- **THEN** Attaching an existing disk takes ≤ 3 clicks and opens 0 windows; dragging a disk to the NVMe controller shows a Power off badge; the info panel loads in ≤ 100 ms from the media cache.

### Requirement: Per-section progressive disclosure (FR-UI04-011)
Virtax SHALL provide "Per-section progressive disclosure" as specified by FR-UI04-011 (BRD-03, priority Must, phase P1; Vietnamese: "Progressive disclosure theo section").

#### Scenario: Specified behaviour of FR-UI04-011
- **WHEN** the behaviour FR-UI04-011 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Each section shows basic fields first and an "Advanced" toggle reveals the rest (remembered in `ui_prefs`); smart defaults come from `GuestOSType`; fields that differ from the default are always shown even when collapsed. There is no global Basic/Expert mode.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/editors/UIEditor.cpp:66-94`

#### Scenario: Acceptance for FR-UI04-011
- **WHEN** the FR-UI04-011 acceptance check is run
- **THEN** Every non-default field is visible on opening its page without expanding Advanced; UI code has 0 references to an Expert flag.

### Requirement: Inline sub-editors for shared folders and USB filters (FR-UI04-012)
Virtax SHALL provide "Inline sub-editors for shared folders and USB filters" as specified by FR-UI04-012 (BRD-03, priority Should, phase P1; Vietnamese: "Editor con inline cho shared folder và USB filter").

#### Scenario: Specified behaviour of FR-UI04-012
- **WHEN** the behaviour FR-UI04-012 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Shared folders (host path, name, mount point, read-only, auto-mount) and USB filters (vendor/product ID, serial, port…) are edited inline in table rows, replacing dialogs S-X2 and S-X3; USB filters can be created "from a plugged-in device"; host paths are checked for existence and permissions.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/editors/UISharedFolderDetailsEditor.cpp; UIUSBFilterDetailsEditor.cpp`

#### Scenario: Acceptance for FR-UI04-012
- **WHEN** the FR-UI04-012 acceptance check is run
- **THEN** Adding a shared folder takes ≤ 3 actions and 0 dialogs; a missing path shows an inline error with a "Create folder" button.

### Requirement: Engine-aware fields (FR-UI04-013)
Virtax SHALL provide "Engine-aware fields" as specified by FR-UI04-013 (BRD-03, priority Should, phase P1; Vietnamese: "Field theo khả năng của engine").

#### Scenario: Specified behaviour of FR-UI04-013
- **WHEN** the behaviour FR-UI04-013 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Choices and limits come from `EngineCaps` (e.g. NIC models, controllers, monitor count). Fields unavailable on the current engine show "Not available on virtax-vmm" with an alternative (E1000 → virtio-net, AHCI → NVMe). In Phase 2, "Move to virtax-vmm" builds its compatibility report from the same data.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/PlatformPropertiesImpl.cpp:223-240`

#### Scenario: Acceptance for FR-UI04-013
- **WHEN** the FR-UI04-013 acceptance check is run
- **THEN** Every form enum comes from `EngineCaps`, none hard-coded (test); an virtax-vmm VM cannot offer E1000.

### Requirement: Non-blocking open and Apply (FR-UI04-014)
Virtax SHALL provide "Non-blocking open and Apply" as specified by FR-UI04-014 (BRD-03, priority Must, phase P1; Vietnamese: "Mở và Apply không chặn").

#### Scenario: Specified behaviour of FR-UI04-014
- **WHEN** the behaviour FR-UI04-014 covers is exercised
- **THEN** it behaves as BRD-03 specifies: The sheet opens from the read model with no modal progress dialog. Apply returns per-field results; slow Live changes (hot-plugging a disk, switching attachments) become Operations in the Task center; one failing field does not roll back other fields already applied Live.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/UISettingsSerializer.cpp:150-205; UIAdvancedSettingsDialog.cpp:1208`

#### Scenario: Acceptance for FR-UI04-014
- **WHEN** the FR-UI04-014 acceptance check is run
- **THEN** 0 modal progress dialogs; Apply on the registry/manifest path returns in ≤ 50 ms p95; the UI thread is never blocked > 50 ms in the edit flow.

### Requirement: Global Preferences on the same sheet (FR-UI04-015)
Virtax SHALL provide "Global Preferences on the same sheet" as specified by FR-UI04-015 (BRD-03, priority Must, phase P1; Vietnamese: "Preferences toàn cục dùng chung sheet").

#### Scenario: Specified behaviour of FR-UI04-015
- **WHEN** the behaviour FR-UI04-015 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Preferences (S-G0…S-G7: General, Input, Update, Language, Display, Proxy, Interface) use the same shell: search, inline validation, diff; the Interface (theme) page exists on all 3 OSes; page content is specified in BRD-06 (UI-07).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/UIAdvancedSettingsDialogSpecific.cpp:200-275`

#### Scenario: Acceptance for FR-UI04-015
- **WHEN** the FR-UI04-015 acceptance check is run
- **THEN** Preferences search finds "host key", "proxy" and "ngôn ngữ"; changes go to `ui_prefs`/the daemon and never into `VirtualBox.xml`.

### Requirement: Engine overrides page (advanced) (FR-UI04-016)
Virtax SHALL provide "Engine overrides page (advanced)" as specified by FR-UI04-016 (BRD-03, priority Could, phase P1; Vietnamese: "Trang Engine overrides (nâng cao)").

#### Scenario: Specified behaviour of FR-UI04-016
- **WHEN** the behaviour FR-UI04-016 covers is exercised
- **THEN** it behaves as BRD-03 specifies: Legacy engine only, behind Advanced: a table of `VBoxInternal/*` keys with autocomplete from known keys (e.g. `/TM/TSCMode`, `/HM/VmxPleGap`), value-type validation and a risk warning; every change appears in the diff and is audited.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/ConsoleImplConfigCommon.cpp:695-760`

#### Scenario: Acceptance for FR-UI04-016
- **WHEN** the FR-UI04-016 acceptance check is run
- **THEN** An unknown key needs an extra confirmation; every override appears in Review diff and in `virtaxctl vm get --json`.
