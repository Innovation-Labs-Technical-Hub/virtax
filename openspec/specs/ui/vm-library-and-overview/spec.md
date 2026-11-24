# VM library & overview Specification

## Purpose
Groups, tags, search/filter, virtualized list, editable config cards, live preview, quick actions. Module `UI-02` (VM library & overview; Vietnamese: "Thư viện VM & tổng quan") from BRD-01, phase P1, priority Must. Owner crates: virtax. Depends on: `control-plane/control-plane-api-and-ipc`, `control-plane/registry-and-configuration-store`, `control-plane/metrics-logs-and-diagnostics`. VirtualBox reference: `src/VBox/Frontends/VirtualBox/src/manager/chooser, manager/details`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Virtualized VM list for 500+ VMs (FR-UI02-001)
Virtax SHALL provide "Virtualized VM list for 500+ VMs" as specified by FR-UI02-001 (BRD-01, priority Must, phase P1; Vietnamese: "Danh sách VM ảo hoá cho 500+ VM").

#### Scenario: Specified behaviour of FR-UI02-001
- **WHEN** the behaviour FR-UI02-001 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The list only builds visible rows plus a buffer, with a fixed 56 px row height. Each row shows an OS tile (SVG logo or monogram), name, state + duration, RAM and current snapshot; group headers show "n · m running". Memory and paint time do not grow with the VM count.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/chooser/UIChooserItemMachine.cpp:1066; UIChooserView.cpp`

#### Scenario: Acceptance for FR-UI02-001
- **WHEN** the FR-UI02-001 acceptance check is run
- **THEN** 2,000 VMs: continuous scrolling at p95 frame ≤ 16.7 ms; RSS grows ≤ 15 MB versus 50 VMs; ↑/↓ selection responds in ≤ 50 ms.

### Requirement: Always-visible search (FR-UI02-002)
Virtax SHALL provide "Always-visible search" as specified by FR-UI02-002 (BRD-01, priority Must, phase P1; Vietnamese: "Search luôn hiện").

#### Scenario: Specified behaviour of FR-UI02-002
- **WHEN** the behaviour FR-UI02-002 covers is exercised
- **THEN** it behaves as BRD-01 specifies: A fixed search box at the top of the list matches name, OS, tags, group, guest IP and notes, with or without diacritics; typing while the list has focus goes into the box; matches are highlighted; Esc clears; Ctrl/⌘-F focuses the box.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/chooser/UIChooserModel.cpp:1028-1035`

#### Scenario: Acceptance for FR-UI02-002
- **WHEN** the FR-UI02-002 acceptance check is run
- **THEN** 500 VMs: ≤ 30 ms per keystroke to results; "ubu 24" matches "ubuntu-24.04"; the screen reader announces the result count ("12 results").

### Requirement: Filter chips, sorting and saved views (FR-UI02-003)
Virtax SHALL provide "Filter chips, sorting and saved views" as specified by FR-UI02-003 (BRD-01, priority Should, phase P1; Vietnamese: "Chip lọc, sắp xếp và view đã lưu").

#### Scenario: Specified behaviour of FR-UI02-003
- **WHEN** the behaviour FR-UI02-003 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Filter chips by state (Running, Saved, Off, Aborted), arch, engine (vbox-legacy/virtax-vmm), OS family and tag (AND across chip groups, OR within one); sort by name, state, last used, RAM or a manual in-group order as in VirtualBox. A filter + sort combination is saved as a named view in `ui_prefs` and appears in the palette.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/chooser/UIChooserModel.cpp:989-996 (sort, no filters)`

#### Scenario: Acceptance for FR-UI02-003
- **WHEN** the FR-UI02-003 acceptance check is run
- **THEN** Filtering "Running + tag ci" over 500 VMs takes ≤ 30 ms; saved views survive restarts; chip counts match the daemon-side `ListMachines` filter.

### Requirement: Nested groups compatible with VirtualBox (FR-UI02-004)
Virtax SHALL provide "Nested groups compatible with VirtualBox" as specified by FR-UI02-004 (BRD-01, priority Must, phase P1; Vietnamese: "Nhóm lồng nhau tương thích VirtualBox").

#### Scenario: Specified behaviour of FR-UI02-004
- **WHEN** the behaviour FR-UI02-004 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Path-style groups (`/Dev lab/QA`) compatible with VirtualBox groups; collapse/expand (remembered in `ui_prefs`), drag-and-drop of one or many VMs between groups, create/rename groups. A regroup sends `UpdateMachine(groups)`; for legacy VMs the value goes through `IMachine::groups`, so the VirtualBox GUI sees the same structure.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/chooser/UIChooserItemGroup.cpp; UIChooserNodeGroup.cpp`

#### Scenario: Acceptance for FR-UI02-004
- **WHEN** the FR-UI02-004 acceptance check is run
- **THEN** Dragging 3 VMs into a new group moves all 3 within ≤ 300 ms with Undo; the VirtualBox 7.2 GUI shows the same groups; 4-level nesting renders with correct indentation.

### Requirement: Free-form tags, never written into .vbox (FR-UI02-005)
Virtax SHALL provide "Free-form tags, never written into .vbox" as specified by FR-UI02-005 (BRD-01, priority Must, phase P1; Vietnamese: "Tag tự do, không ghi vào .vbox").

#### Scenario: Specified behaviour of FR-UI02-005
- **WHEN** the behaviour FR-UI02-005 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Many-to-many tags shown as text chips (color is secondary); added/removed from Overview, the context menu or bulk actions. Tags live in the registry (`machine_tags` table, BRD-03) and in `machine.toml` for virtax-vmm VMs; they are never written into `.vbox`; `virtaxctl vm list --tag` reads the same data.
- **AND** the VirtualBox reference for the current behaviour is `new (VirtualBox has groups only: include/VBox/settings.h llGroups)`

#### Scenario: Acceptance for FR-UI02-005
- **WHEN** the FR-UI02-005 acceptance check is run
- **THEN** Bulk-tagging 20 VMs with "ci" makes `virtaxctl vm list --tag ci --json` return exactly 20 entries; the `.vbox` checksums of the legacy VMs are unchanged.

### Requirement: Overview cards with explicit Edit (FR-UI02-006)
Virtax SHALL provide "Overview cards with explicit Edit" as specified by FR-UI02-006 (BRD-01, priority Must, phase P1; Vietnamese: "Card Overview có nút Edit rõ ràng").

#### Scenario: Specified behaviour of FR-UI02-006
- **WHEN** the behaviour FR-UI02-006 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The Overview tab (replacing Details M-09, M-11, M-12) has General, System, Storage, Network, Display, Shared folders & integration and Description cards. Each card has an **Edit** button that opens the Settings sheet at the right page/field via a deep link; no hidden hyperlinks or `exec()` popups remain. Users hide/show and reorder cards; technical values use JetBrains Mono.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/details/UIDetailsElement.cpp:480-505,671-1281`

#### Scenario: Acceptance for FR-UI02-006
- **WHEN** the FR-UI02-006 acceptance check is run
- **THEN** Every card has a visible, Tab-focusable Edit button; Edit on Storage opens the sheet on the Storage page in ≤ 150 ms; there are 0 modal popups in Overview.

### Requirement: Card quick actions with ApplyMode badges (FR-UI02-007)
Virtax SHALL provide "Card quick actions with ApplyMode badges" as specified by FR-UI02-007 (BRD-01, priority Must, phase P1; Vietnamese: "Quick action trên card có badge ApplyMode").

#### Scenario: Specified behaviour of FR-UI02-007
- **WHEN** the behaviour FR-UI02-007 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Cards offer quick actions for frequent tasks: attach/eject ISO, change the NIC attachment (NAT/bridged/host-only), change RAM/vCPU, toggle clipboard. Each action calls `UpdateMachine` with a small field mask and shows a Live / Next boot / Power off badge (from the engine `ApplyMode`) before confirmation; locked controls have a reason tooltip and a suggested action.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/details/UIDetailsElement.cpp:671-1281 (popup editors)`

#### Scenario: Acceptance for FR-UI02-007
- **WHEN** the FR-UI02-007 acceptance check is run
- **THEN** Attaching an ISO to a running VM from the Storage card takes ≤ 2 clicks and completes in ≤ 1 s; changing RAM on a running VM shows a Power off badge and an "Apply at shutdown" option.

### Requirement: Live preview ≤ 1 fps, only while visible (FR-UI02-008)
Virtax SHALL provide "Live preview ≤ 1 fps, only while visible" as specified by FR-UI02-008 (BRD-01, priority Must, phase P1; Vietnamese: "Preview live ≤ 1 fps, chỉ khi hiển thị").

#### Scenario: Specified behaviour of FR-UI02-008
- **WHEN** the behaviour FR-UI02-008 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The preview card reads the latest frame from the data-plane SHM ring (EN-02) through a read-only `OpenDisplay(mode=PREVIEW)`, downscales on the GPU, at most 1 fps and only while the card is visible and the window is not minimized. In the Battery power profile or when the VM is paused/saved it shows the stored thumbnail. This replaces `TakeScreenShotToArray` over IPC.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/details/UIMachinePreview.cpp:324; UIMachineView.cpp:1979`

#### Scenario: Acceptance for FR-UI02-008
- **WHEN** the FR-UI02-008 acceptance check is run
- **THEN** With the card hidden or the Manager minimized, 0 frame reads per second (counter); visible, ≤ 1 per second at ≤ 0.3% of one core extra; a "LIVE · 1 fps" badge is shown.

### Requirement: Subscription-based resource mini-card (FR-UI02-009)
Virtax SHALL provide "Subscription-based resource mini-card" as specified by FR-UI02-009 (BRD-01, priority Should, phase P1; Vietnamese: "Mini-card tài nguyên theo subscription").

#### Scenario: Specified behaviour of FR-UI02-009
- **WHEN** the behaviour FR-UI02-009 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The Resource use card shows 2-minute CPU, RAM, Disk and Network sparklines from `Metrics.Watch`; it subscribes only while visible and unsubscribes when hidden, always printing the current value next to the chart (never color alone). Clicking opens the Performance tab (BRD-05).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/activity/vmactivity/UIVMActivityMonitor.cpp:1502,1524`

#### Scenario: Acceptance for FR-UI02-009
- **WHEN** the FR-UI02-009 acceptance check is run
- **THEN** Hiding the card drops the daemon to 0 subscribers within ≤ 1 s and stops sampling; values lag `virtaxctl vm metrics` by ≤ 1.5 s.

### Requirement: Readable VM states and error card (FR-UI02-010)
Virtax SHALL provide "Readable VM states and error card" as specified by FR-UI02-010 (BRD-01, priority Must, phase P1; Vietnamese: "Trạng thái VM dễ hiểu và error card").

#### Scenario: Specified behaviour of FR-UI02-010
- **WHEN** the behaviour FR-UI02-010 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The 28 `MachineState` values are folded into UI states (Running, Paused, Saved, Off, Aborted, Starting/Stopping/Saving…, Not responding), each with an icon + text + token color. Aborted VMs get a "See log" link; inaccessible VMs show an error card (replacing the M-17 HTML pane) with an `VTX-CFG-xxx` code, a plain cause and fixes (Locate, Remove from list).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/UIErrorPane.cpp; Main/idl/VirtualBox.xidl (MachineState)`

#### Scenario: Acceptance for FR-UI02-010
- **WHEN** the FR-UI02-010 acceptance check is run
- **THEN** Renaming a VM's `.vbox` file shows "Inaccessible" within ≤ 1 s with a Locate button; a unit test maps 28/28 enum values.

### Requirement: VM header with fixed primary actions (FR-UI02-011)
Virtax SHALL provide "VM header with fixed primary actions" as specified by FR-UI02-011 (BRD-01, priority Must, phase P1; Vietnamese: "Header VM với nút chính cố định").

#### Scenario: Specified behaviour of FR-UI02-011
- **WHEN** the behaviour FR-UI02-011 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The workspace header shows the name, state chip + duration, latest snapshot, guest IP (with Guest Additions/agent) and buttons in a fixed order: Open console, Pause/Resume, Snapshot, Shut down ▾ (ACPI, Save state, Power off), ⋯. The primary button follows the state (Start when off) and uses optimistic UI.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/UIVirtualBoxWidget.cpp:323-329 (updateToolbar)`

#### Scenario: Acceptance for FR-UI02-011
- **WHEN** the FR-UI02-011 acceptance check is run
- **THEN** Screenshot diffs show the 4 primary buttons in the same place on every tab; Start shows Starting within ≤ 200 ms and rolls back if the Operation fails.

### Requirement: Multi-select and bulk actions (FR-UI02-012)
Virtax SHALL provide "Multi-select and bulk actions" as specified by FR-UI02-012 (BRD-01, priority Should, phase P1; Vietnamese: "Chọn nhiều và thao tác hàng loạt").

#### Scenario: Specified behaviour of FR-UI02-012
- **WHEN** the behaviour FR-UI02-012 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Select many VMs (Shift, Ctrl/⌘, Ctrl/⌘-A on filtered results) to start, shut down, snapshot, tag or regroup them. Each VM gets its own Operation, grouped under one parent entry in the Task center. Multi-VM actions always list the exact VMs that will change and wait for the user's confirmation.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIActionPoolManager.cpp (Start/Stop on selection)`

#### Scenario: Acceptance for FR-UI02-012
- **WHEN** the FR-UI02-012 acceptance check is run
- **THEN** Starting 10 selected VMs shows a confirmation listing all 10 names; the Task center shows one parent entry "10/10"; one failing VM does not block the other 9.

### Requirement: Resources table for all running VMs (FR-UI02-013)
Virtax SHALL provide "Resources table for all running VMs" as specified by FR-UI02-013 (BRD-01, priority Should, phase P1; Vietnamese: "Bảng Resources cho mọi VM đang chạy").

#### Scenario: Specified behaviour of FR-UI02-013
- **WHEN** the behaviour FR-UI02-013 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The Resources rail item (replacing M-23) is a virtualized table of running VMs: guest/VMM CPU, RAM, disk R/W, network RX/TX, exits/s, sortable by column, with a host CPU/RAM strip. Data comes from one aggregated 1 Hz `Metrics.Watch`, only while the view is open.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/activity/overview/UIVMActivityOverviewWidget.cpp`

#### Scenario: Acceptance for FR-UI02-013
- **WHEN** the FR-UI02-013 acceptance check is run
- **THEN** With 30 running VMs the table refreshes at 1 Hz with Manager CPU ≤ 1%; closing the view stops daemon sampling within ≤ 1 s.

### Requirement: Event-driven updates, no polling (FR-UI02-014)
Virtax SHALL provide "Event-driven updates, no polling" as specified by FR-UI02-014 (BRD-01, priority Must, phase P1; Vietnamese: "Cập nhật theo event, không polling").

#### Scenario: Specified behaviour of FR-UI02-014
- **WHEN** the behaviour FR-UI02-014 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The list, header and cards update from `Events.Watch` (by-value events coalesced per resource, latest value wins). During bursts (snapshot storms, bulk imports) the UI applies at most one batch per frame.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIMainEventListener.cpp:177; Main/src-server/EventImpl.cpp:876`

#### Scenario: Acceptance for FR-UI02-014
- **WHEN** the FR-UI02-014 acceptance check is run
- **THEN** Engine event to UI ≤ 250 ms p95; a 10,000-event burst never blocks the UI thread > 50 ms and the final state matches `GetMachine`.
