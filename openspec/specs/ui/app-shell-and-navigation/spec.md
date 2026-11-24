# App shell & navigation Specification

## Purpose
Global rail, VM list, tabbed workspace, command palette, task center, shortcuts, multi-window. Module `UI-01` (App shell & navigation; Vietnamese: "Khung ứng dụng & điều hướng") from BRD-01, phase P1, priority Must. Owner crates: virtax (Manager), virtax-ui-kit. Depends on: `control-plane/control-plane-api-and-ipc`, `ui/design-system-theming-a11y-i18n`. VirtualBox reference: `src/VBox/Frontends/VirtualBox/src/manager, globals/UIActionPool*, notificationcenter`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: One navigation model: rail → list → tabs (FR-UI01-001)
Virtax SHALL provide "One navigation model: rail → list → tabs" as specified by FR-UI01-001 (BRD-01, priority Must, phase P1; Vietnamese: "Một mô hình điều hướng: rail → danh sách → tab").

#### Scenario: Specified behaviour of FR-UI01-001
- **WHEN** the behaviour FR-UI01-001 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The Manager has exactly three fixed regions: a 60 px global rail (VMs, Media, Network, Resources, Plugins; Tasks and Preferences at the bottom — icon + always-visible label, count badges), the VM list with an always-visible search, and the tabbed workspace of the selected VM. No horizontal tool strip and no window-only tools (M-22, M-26); no Cloud in P1.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/tools/UIToolsModel.cpp:504-525; UIVirtualBoxManager.cpp:1026`

#### Scenario: Acceptance for FR-UI01-001
- **WHEN** the FR-UI01-001 acceptance check is run
- **THEN** Given the Manager is open, when any tab of any VM is needed, then it takes ≤ 2 actions; a 25-task audit finds 0 tasks needing a secondary window; the Tasks badge updates in ≤ 250 ms.

### Requirement: Per-VM tabbed workspace (FR-UI01-002)
Virtax SHALL provide "Per-VM tabbed workspace" as specified by FR-UI01-002 (BRD-01, priority Must, phase P1; Vietnamese: "Workspace dạng tab theo từng VM").

#### Scenario: Specified behaviour of FR-UI01-002
- **WHEN** the behaviour FR-UI01-002 covers is exercised
- **THEN** it behaves as BRD-01 specifies: A VM workspace has Overview, Snapshots, Logs, Performance, Files and Guest shell tabs (Guest shell appears only when `virtax-agent` is available, Phase 2). The last tab and scroll position are remembered per VM in `ui_prefs`; tab badges (e.g. snapshot count) come from the read model with no extra API call.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/UIToolPane.cpp:135-160; UIMachineToolsWidget.cpp`

#### Scenario: Acceptance for FR-UI01-002
- **WHEN** the FR-UI01-002 acceptance check is run
- **THEN** Switching to another VM and back restores the same tab and position; a tab switch reaches its first frame in ≤ 100 ms (skeleton if data is not there yet).

### Requirement: Ctrl/⌘-K command palette (FR-UI01-003)
Virtax SHALL provide "Ctrl/⌘-K command palette" as specified by FR-UI01-003 (BRD-01, priority Must, phase P1; Vietnamese: "Command palette Ctrl/⌘-K").

#### Scenario: Specified behaviour of FR-UI01-003
- **WHEN** the behaviour FR-UI01-003 covers is exercised
- **THEN** it behaves as BRD-01 specifies: A fuzzy palette over actions (start, snapshot, attach ISO…), VMs, groups, tags, Settings pages/fields and help topics. Each result shows its kind (VM, Device, Settings, Engine), its shortcut and the equivalent `virtaxctl` command, ranking the selected VM's context first. Vietnamese matches with or without diacritics.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIActionPoolManager.cpp; UIActionPoolRuntime.cpp`

#### Scenario: Acceptance for FR-UI01-003
- **WHEN** the FR-UI01-003 acceptance check is run
- **THEN** With 500 VMs and ≈300 actions, results update ≤ 30 ms per keystroke; "may ao" matches "máy ảo"; Enter gives first feedback in ≤ 100 ms.

### Requirement: Shared action registry with disabled reasons (FR-UI01-004)
Virtax SHALL provide "Shared action registry with disabled reasons" as specified by FR-UI01-004 (BRD-01, priority Must, phase P1; Vietnamese: "Action registry dùng chung, luôn có lý do khi bị tắt").

#### Scenario: Specified behaviour of FR-UI01-004
- **WHEN** the behaviour FR-UI01-004 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Every command is an `ActionDescriptor` declared once: id, VI/EN labels, Lucide icon, shortcut, enable predicate, disabled reason and `virtaxctl` command. Menus, context menus, toolbars, the palette and shortcuts are all generated from it, so a disabled action always carries a reason and a next step (e.g. "VM is running — Shut down & apply").
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIActionPool.cpp (≈320 action indices)`

#### Scenario: Acceptance for FR-UI01-004
- **WHEN** the FR-UI01-004 acceptance check is run
- **THEN** 100% of menu/toolbar actions appear in the palette; an automated test checks that every disabled action returns a non-empty `disabled_reason` in both VI and EN.

### Requirement: `virtax://` deep links and single instance (FR-UI01-005)
Virtax SHALL provide "`virtax://` deep links and single instance" as specified by FR-UI01-005 (BRD-01, priority Should, phase P1; Vietnamese: "Deep link `virtax://` và single-instance").

#### Scenario: Specified behaviour of FR-UI01-005
- **WHEN** the behaviour FR-UI01-005 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Virtax registers the `virtax://` scheme with the OS (Info.plist on macOS, MSIX protocol/registry on Windows, `x-scheme-handler/virtax` in the .desktop file on Linux). Links open the exact screen, e.g. `virtax://vm/<id>/settings/storage#cache_mode`, `virtax://vm/<id>/snapshots`, `virtax://op/<id>`, `virtax://prefs/input`. If the Manager is running, the link is forwarded to the existing instance. Links only navigate; they never execute an action.
- **AND** the VirtualBox reference for the current behaviour is `new (replaces src/VBox/Frontends/VirtualBox/src/manager/UIVirtualBoxManager.cpp:1026-1075)`

#### Scenario: Acceptance for FR-UI01-005
- **WHEN** the FR-UI01-005 acceptance check is run
- **THEN** Clicking a link from a terminal or browser while the Manager runs focuses the window and navigates in ≤ 500 ms; an unknown id shows an `VTX-UI-004` toast with an "Open VM list" button and never crashes.

### Requirement: Task center for every Operation (FR-UI01-006)
Virtax SHALL provide "Task center for every Operation" as specified by FR-UI01-006 (BRD-01, priority Must, phase P1; Vietnamese: "Task center cho mọi Operation").

#### Scenario: Specified behaviour of FR-UI01-006
- **WHEN** the behaviour FR-UI01-006 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The Task center (replacing M-24) is a right-side drawer listing every Operation from `Operations.Watch`: name, current step, %, ETA, a Cancel button and context actions (Show console, Open log). Errors show the `VTX-<AREA>-<NNN>` code, a bilingual message, a quick fix and "Copy technical details" with the original VERR_*/OS code. History comes from the `operations` table (30 days or 500 entries).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/notificationcenter/UINotificationCenter.cpp; UINotificationProgress.cpp`

#### Scenario: Acceptance for FR-UI01-006
- **WHEN** the FR-UI01-006 acceptance check is run
- **THEN** Given an OVA export and a VM creation are running, when the Task center opens, then both entries show % and ETA updated ≥ once per second; Cancel moves to Cancelling in ≤ 200 ms; history survives a Manager restart.

### Requirement: Non-blocking confirmations, toasts and undo (FR-UI01-007)
Virtax SHALL provide "Non-blocking confirmations, toasts and undo" as specified by FR-UI01-007 (BRD-01, priority Must, phase P1; Vietnamese: "Xác nhận không chặn, toast và Undo").

#### Scenario: Specified behaviour of FR-UI01-007
- **WHEN** the behaviour FR-UI01-007 covers is exercised
- **THEN** it behaves as BRD-01 specifies: There are no nested event loops. Confirmation for destructive actions is an asynchronous dialog that states the consequence (e.g. "Delete 2 disks, 14.2 GiB") with a safe default button; information is a toast that hides after 5 s; reversible actions (remove from list, regroup, untag) use Undo in the toast instead of asking first.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/notificationcenter/UINotificationQuestion.cpp:1302-1347`

#### Scenario: Acceptance for FR-UI01-007
- **WHEN** the FR-UI01-007 acceptance check is run
- **THEN** A static check finds 0 nested-loop calls in UI code; while a confirmation is open the VM list still updates live; Undo restores the state within a 10 s window.

### Requirement: OS notifications while in the background (FR-UI01-008)
Virtax SHALL provide "OS notifications while in the background" as specified by FR-UI01-008 (BRD-01, priority Could, phase P1; Vietnamese: "Thông báo hệ điều hành khi app ở nền").

#### Scenario: Specified behaviour of FR-UI01-008
- **WHEN** the behaviour FR-UI01-008 covers is exercised
- **THEN** it behaves as BRD-01 specifies: When the window is not focused, completed/failed Operations and Aborted VMs raise OS notifications (UNUserNotificationCenter, Windows App Notifications, `org.freedesktop.Notifications`) with a deep link back. Users choose which kinds to receive in Preferences; OS Focus/Do Not Disturb is respected.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-UI01-008
- **WHEN** the FR-UI01-008 acceptance check is run
- **THEN** A 20 GiB clone finishing while the Manager is in the background raises 1 notification within ≤ 2 s and clicking opens `virtax://op/<id>`; disabling "Completed" stops that kind.

### Requirement: Shortcuts and full keyboard operation (FR-UI01-009)
Virtax SHALL provide "Shortcuts and full keyboard operation" as specified by FR-UI01-009 (BRD-01, priority Must, phase P1; Vietnamese: "Phím tắt và dùng hoàn toàn bằng bàn phím").

#### Scenario: Specified behaviour of FR-UI01-009
- **WHEN** the behaviour FR-UI01-009 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Default shortcuts follow the OS (⌘ on macOS, Ctrl on Windows/Linux) and can be customised in Preferences › Input with conflict detection; "?" opens the shortcut sheet; F6/Shift-F6 cycle between rail, list and workspace. Every Manager task can be done with the keyboard alone.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/settings/editors/UIShortcutConfigurationEditor.cpp`

#### Scenario: Acceptance for FR-UI01-009
- **WHEN** the FR-UI01-009 acceptance check is run
- **THEN** A keyboard-only script (find VM, start, open Settings, change RAM, Apply, open Task center) completes 100%; assigning a duplicate chord shows an inline warning and is not saved.

### Requirement: Multi-window over one shared store (FR-UI01-010)
Virtax SHALL provide "Multi-window over one shared store" as specified by FR-UI01-010 (BRD-01, priority Should, phase P1; Vietnamese: "Đa cửa sổ dùng chung một store").

#### Scenario: Specified behaviour of FR-UI01-010
- **WHEN** the behaviour FR-UI01-010 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The Logs and Performance tabs can be popped out into their own windows, and users can open a second Manager window; all windows share one reactive store and one gRPC connection to `virtaxd`. Window position/size is stored in `ui_prefs` per monitor layout and never written into `.vbox` files.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/extensions/QIManagerDialog.cpp; UIExtraDataManager.cpp:363-400`

#### Scenario: Acceptance for FR-UI01-010
- **WHEN** the FR-UI01-010 acceptance check is run
- **THEN** Popping out Logs for VM A and Performance for VM B keeps both live with exactly 1 connection to `virtaxd`; reopening the app restores windows on monitors that still exist.

### Requirement: Startup from the cached read model (FR-UI01-011)
Virtax SHALL provide "Startup from the cached read model" as specified by FR-UI01-011 (BRD-01, priority Must, phase P1; Vietnamese: "Khởi động từ read model cache").

#### Scenario: Specified behaviour of FR-UI01-011
- **WHEN** the behaviour FR-UI01-011 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The Manager paints its window immediately and shows the VM list from the read-model snapshot (`ListMachines` served from `summary_json` in SQLite), then syncs through `Events.Watch(since_seq)`; stale data is marked "refreshing" instead of blocking. The UI startup path contains no engine/COM call.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/UIVirtualMachineItemLocal.cpp:68-100; main.cpp:507-613`

#### Scenario: Acceptance for FR-UI01-011
- **WHEN** the FR-UI01-011 acceptance check is run
- **THEN** 50 VMs: cold start to an interactive list ≤ 1.5 s, warm ≤ 0.4 s; 500 VMs cold ≤ 2.0 s; the startup trace has no span > 50 ms on the UI thread.

### Requirement: Daemon connection state and offline mode (FR-UI01-012)
Virtax SHALL provide "Daemon connection state and offline mode" as specified by FR-UI01-012 (BRD-01, priority Must, phase P1; Vietnamese: "Trạng thái kết nối daemon và chế độ offline").

#### Scenario: Specified behaviour of FR-UI01-012
- **WHEN** the behaviour FR-UI01-012 covers is exercised
- **THEN** it behaves as BRD-01 specifies: The status bar always shows the `virtaxd` state (connected / reconnecting / offline) and the engine (vbox-legacy 7.2.x, virtax-vmm). On disconnect it reconnects with a 0.2 → 5 s backoff and resumes `Events.Watch` from the last `since_seq`; meanwhile the UI is read-only on cached data with a "Start virtaxd" button.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/VirtualBoxClientImpl.cpp:56,747-817`

#### Scenario: Acceptance for FR-UI01-012
- **WHEN** the FR-UI01-012 acceptance check is run
- **THEN** Killing `virtaxd` while the Manager is open shows an offline banner within ≤ 1 s with no crash; when the daemon returns, state is fully synced within ≤ 2 s and no Operation terminal event is lost.

### Requirement: Home/first run instead of Basic/Expert (FR-UI01-013)
Virtax SHALL provide "Home/first run instead of Basic/Expert" as specified by FR-UI01-013 (BRD-01, priority Should, phase P1; Vietnamese: "Home/first-run thay chế độ Basic/Expert").

#### Scenario: Specified behaviour of FR-UI01-013
- **WHEN** the behaviour FR-UI01-013 covers is exercised
- **THEN** it behaves as BRD-01 specifies: Replaces Home/Welcome (M-03) and never asks Basic/Expert. On first run or with an empty list, an empty state offers Quick Create from ISO, Import (.vbox/OVA) and a Health card: engine found (BYO VirtualBox 7.2.x or Engine Pack), hypervisor (KVM/HVF/WHP/HM) and how to fix what is missing. About and License (M-28, M-29) become Preferences pages.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/manager/UIHomePane.cpp:84-116; VBoxAboutDlg.cpp`

#### Scenario: Acceptance for FR-UI01-013
- **WHEN** the FR-UI01-013 acceptance check is run
- **THEN** A fresh machine with no VMs shows the empty state in ≤ 1 s with 3 actions; a machine without KVM shows `VTX-HV-xxx` on the Health card with steps to enable virtualization.
