# VM tools (Logs, Performance, Files, Guest shell) Specification

## Purpose
Log viewer with filter/bookmark, streaming charts, host↔guest file manager, guest shell via agent. Module `UI-06` (VM tools (Logs, Performance, Files, Guest shell); Vietnamese: "Công cụ VM (Logs, Performance, Files, Guest shell)") from BRD-05, phase P1, priority Should. Owner crates: virtax. Depends on: `control-plane/metrics-logs-and-diagnostics`, `control-plane/guest-integration`. VirtualBox reference: `src/VBox/Frontends/VirtualBox/src/logviewer, activity, guestctrl, snapshots`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Tool tabs in the VM workspace (FR-UI06-001)
Virtax SHALL provide "Tool tabs in the VM workspace" as specified by FR-UI06-001 (BRD-05, priority Must, phase P1; Vietnamese: "Tab công cụ trong workspace VM").

#### Scenario: Specified behaviour of FR-UI06-001
- **WHEN** the behaviour FR-UI06-001 covers is exercised
- **THEN** it behaves as BRD-05 specifies: The VM workspace has Overview, Snapshots, Logs, Performance, Files and Shell tabs, remembers the last tab per VM and supports deep links `virtax://vm/<id>/logs?line=…`; any tab can pop out into a window (replacing M-08 and the duplicate R-09/R-13 windows).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/UIMachineToolsWidget.cpp`

#### Scenario: Acceptance for FR-UI06-001
- **WHEN** the FR-UI06-001 acceptance check is run
- **THEN** Switching tabs takes ≤ 100 ms; a deep link opens the right VM, tab and log line; pop-out keeps scroll/filter state.

### Requirement: Snapshots tab (FR-UI06-002)
Virtax SHALL provide "Snapshots tab" as specified by FR-UI06-002 (BRD-05, priority Must, phase P1; Vietnamese: "Tab Snapshots").

#### Scenario: Specified behaviour of FR-UI06-002
- **WHEN** the behaviour FR-UI06-002 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Snapshot tree with Take/Restore/Clone/Delete, a details pane (time, size, hardware, note, thumbnail), the disk-chain line and space used by snapshots, and a delete-progress card with ETA + Cancel (MK-02); keyboard tree navigation.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/snapshots/UISnapshotPane.cpp`

#### Scenario: Acceptance for FR-UI06-002
- **WHEN** the FR-UI06-002 acceptance check is run
- **THEN** Every tree action is keyboard reachable; deleting a snapshot never locks the tab (other snapshots stay selectable).

### Requirement: Virtualized log viewer with follow (FR-UI06-003)
Virtax SHALL provide "Virtualized log viewer with follow" as specified by FR-UI06-003 (BRD-05, priority Must, phase P1; Vietnamese: "Log viewer ảo hoá, có follow").

#### Scenario: Specified behaviour of FR-UI06-003
- **WHEN** the behaviour FR-UI06-003 covers is exercised
- **THEN** it behaves as BRD-05 specifies: The daemon indexes lines and serves ranges (`Logs.Read`) and a stream (`Logs.Tail`); the UI renders only visible lines and has a follow mode that auto-scrolls as the VM logs; the whole file is never loaded into memory as the Qt version does (256 MB cap).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/logviewer/UIVMLogViewerWidget.cpp:73,308-330`

#### Scenario: Acceptance for FR-UI06-003
- **WHEN** the FR-UI06-003 acceptance check is run
- **THEN** A 256 MB log opens within 1 s, scrolls at 60 fps with ≤ 64 MB extra RAM; new lines appear in follow mode within 500 ms.

### Requirement: Log search and filters (FR-UI06-004)
Virtax SHALL provide "Log search and filters" as specified by FR-UI06-004 (BRD-05, priority Must, phase P1; Vietnamese: "Tìm kiếm và lọc log").

#### Scenario: Specified behaviour of FR-UI06-004
- **WHEN** the behaviour FR-UI06-004 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Plain/regex search with match count and next/prev; filter chips by component (NEM, HM, PDM, VD, GIM, GUI) and severity; time-range filtering; searches run in the daemon on the index without blocking the UI.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/logviewer/UIVMLogViewerSearchWidget.cpp, UIVMLogViewerFilterWidget.cpp`

#### Scenario: Acceptance for FR-UI06-004
- **WHEN** the FR-UI06-004 acceptance check is run
- **THEN** A regex search over 1 million lines takes ≤ 300 ms; toggling a filter chip takes ≤ 100 ms.

### Requirement: Bookmarks, log sets and export (FR-UI06-005)
Virtax SHALL provide "Bookmarks, log sets and export" as specified by FR-UI06-005 (BRD-05, priority Should, phase P1; Vietnamese: "Bookmark, chọn bộ log và xuất").

#### Scenario: Specified behaviour of FR-UI06-005
- **WHEN** the behaviour FR-UI06-005 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Bookmark lines with notes (stored in `ui_prefs` by file hash); choose the log set: `VBox.log` + `.1…N`, hardening log, per-VM `virtaxd` log, installer logs; export a selection or "Add to diagnostics bundle" (CP-11).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/logviewer/UIVMLogViewerBookmarksWidget.cpp`

#### Scenario: Acceptance for FR-UI06-005
- **WHEN** the FR-UI06-005 acceptance check is run
- **THEN** Bookmarks survive an app restart; exporting 10,000 lines takes ≤ 1 s.

### Requirement: Jump to the error line from a crash (FR-UI06-006)
Virtax SHALL provide "Jump to the error line from a crash" as specified by FR-UI06-006 (BRD-05, priority Should, phase P1; Vietnamese: "Nhảy tới dòng lỗi từ thông báo crash").

#### Scenario: Specified behaviour of FR-UI06-006
- **WHEN** the behaviour FR-UI06-006 covers is exercised
- **THEN** it behaves as BRD-05 specifies: The "Open log at error" button on a crash error (FR-CP03-007) opens the Logs tab at the first recognised error line (Guru Meditation, `VERR_*`, assertion), highlighted with ±20 lines of context.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-UI06-006
- **WHEN** the FR-UI06-006 acceptance check is run
- **THEN** 10 crash fixtures land on the right error line ≥ 9/10, opening within 1 s.

### Requirement: Streaming Performance tab (FR-UI06-007)
Virtax SHALL provide "Streaming Performance tab" as specified by FR-UI06-007 (BRD-05, priority Must, phase P1; Vietnamese: "Tab Performance dạng stream").

#### Scenario: Specified behaviour of FR-UI06-007
- **WHEN** the behaviour FR-UI06-007 covers is exercised
- **THEN** it behaves as BRD-05 specifies: CPU guest/VMM, Memory (used/balloon), Disk R/W and Network RX/TX charts via 1 Hz `Metrics.Watch`; 2 min/15 min/1 h windows from the daemon ring buffer; area charts with text values and a hover crosshair; subscribe when visible, unsubscribe when hidden.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/activity/vmactivity/UIVMActivityMonitor.cpp:1502,1524`

#### Scenario: Acceptance for FR-UI06-007
- **WHEN** the FR-UI06-007 acceptance check is run
- **THEN** Tab open: ≤ 0.5% host CPU; tab hidden: subscription closed within 1 s and 0 samples taken; switching to 15 min renders at once from the ring buffer.

### Requirement: Pause stream and export CSV (FR-UI06-008)
Virtax SHALL provide "Pause stream and export CSV" as specified by FR-UI06-008 (BRD-05, priority Must, phase P1; Vietnamese: "Pause stream và xuất CSV").

#### Scenario: Specified behaviour of FR-UI06-008
- **WHEN** the behaviour FR-UI06-008 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Pause freezes the charts for inspection (data keeps flowing into the ring buffer); Export CSV/JSON for the selected window with units and ISO 8601 timestamps; a table view replaces the charts on demand.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/activity/vmactivity/UIVMActivityMonitorContainer.cpp`

#### Scenario: Acceptance for FR-UI06-008
- **WHEN** the FR-UI06-008 acceptance check is run
- **THEN** Resuming after a 5-minute pause loses no samples; a 1-hour CSV (3,600 rows × 8 columns) exports within 1 s and opens correctly in LibreOffice/Excel.

### Requirement: Hypervisor exits panel (advanced) (FR-UI06-009)
Virtax SHALL provide "Hypervisor exits panel (advanced)" as specified by FR-UI06-009 (BRD-05, priority Could, phase P1; Vietnamese: "Panel hypervisor exits (nâng cao)").

#### Scenario: Specified behaviour of FR-UI06-009
- **WHEN** the behaviour FR-UI06-009 covers is exercised
- **THEN** it behaves as BRD-05 specifies: An advanced panel shows exits/s by reason (MMIO, HLT, APIC timer…) with hints (e.g. "enable ioeventfd on virtax-vmm"), sampled only while open; Phase 1 reads the engine's STAM statistics, Phase 2 `virtax-vmm` shared-memory counters.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/MachineDebuggerImpl.cpp (getStats)`

#### Scenario: Acceptance for FR-UI06-009
- **WHEN** the FR-UI06-009 acceptance check is run
- **THEN** Closing the panel stops all STAM reads; an open panel adds ≤ 1% host CPU.

### Requirement: Host↔guest file manager (FR-UI06-010)
Virtax SHALL provide "Host↔guest file manager" as specified by FR-UI06-010 (BRD-05, priority Should, phase P1; Vietnamese: "File manager host↔guest").

#### Scenario: Specified behaviour of FR-UI06-010
- **WHEN** the behaviour FR-UI06-010 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Two panes with breadcrumbs, drag-and-drop between panes and from the host desktop; an operations queue with progress/cancel; Phase 1 uses guest control (needs GA, credentials optionally in the OS keystore) with parallel requests; Phase 2 uses `virtax-agent` over vsock with no credentials.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/GuestSessionImplTasks.cpp:420-425`

#### Scenario: Acceptance for FR-UI06-010
- **WHEN** the FR-UI06-010 acceptance check is run
- **THEN** Copying a 1 GB file host→guest in Phase 1 is ≥ 2× VirtualBox 7.2's File Manager; Phase 2 ≥ 200 MB/s; cancel within 1 s.

### Requirement: Guest shell (FR-UI06-011)
Virtax SHALL provide "Guest shell" as specified by FR-UI06-011 (BRD-05, priority Should, phase P1; Vietnamese: "Guest shell").

#### Scenario: Specified behaviour of FR-UI06-011
- **WHEN** the behaviour FR-UI06-011 covers is exercised
- **THEN** it behaves as BRD-05 specifies: A terminal tab (VT/xterm emulator) runs a guest shell: Phase 1 via an `IGuestSession` process (stdin/stdout, no PTY, labelled "basic shell"); Phase 2 a full PTY via the agent (resize, colours, curses programs), copy/paste and scrollback search.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/guestctrl/UIGuestControlConsole.cpp`

#### Scenario: Acceptance for FR-UI06-011
- **WHEN** the FR-UI06-011 acceptance check is run
- **THEN** Phase 1: `uname -a` returns output within 1 s; Phase 2: `htop` renders correctly and resizes with the window within 100 ms.

### Requirement: Tool availability states and accessibility (FR-UI06-012)
Virtax SHALL provide "Tool availability states and accessibility" as specified by FR-UI06-012 (BRD-05, priority Must, phase P1; Vietnamese: "Trạng thái khả dụng và accessibility của công cụ").

#### Scenario: Specified behaviour of FR-UI06-012
- **WHEN** the behaviour FR-UI06-012 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Unavailable tools state why and how to enable them (VM off → Files needs a running VM + GA/agent, with an "Install guest agent" button) instead of silently greying out; charts have a table alternative, logs are keyboard navigable and finished Operations are announced to screen readers.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIAccessible.cpp`

#### Scenario: Acceptance for FR-UI06-012
- **WHEN** the FR-UI06-012 acceptance check is run
- **THEN** Every tab has an empty/unavailable state with text + action; the accessibility audit reports 0 critical issues; contrast ≥ 4.5:1.
