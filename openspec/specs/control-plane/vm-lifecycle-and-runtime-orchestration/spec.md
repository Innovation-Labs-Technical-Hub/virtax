# VM lifecycle & runtime orchestration Specification

## Purpose
Start/stop/pause/save/reset, per-VM actor, deadlines & cancel, crash supervisor, autostart. Module `CP-03` (VM lifecycle & runtime orchestration; Vietnamese: "Vòng đời VM & điều phối runtime") from BRD-05, phase P1, priority Must. Owner crates: virtaxd (actors, supervisor). Depends on: `control-plane/control-plane-api-and-ipc`, `engine/engine-trait-and-legacy-adapter`. VirtualBox reference: `src/VBox/Main/src-server/MachineImpl.cpp, src-client/ConsoleImpl*.cpp, SessionImpl.cpp, ClientWatcher.cpp`. Implementation status (2026-11-24): partial — start, ACPI stop and forced power-off as Operations on the collapsed state machine; no pause, save state or supervisor yet.

## Requirements

### Requirement: Lifecycle commands as Operations (FR-CP03-001)
Virtax SHALL provide "Lifecycle commands as Operations" as specified by FR-CP03-001 (BRD-05, priority Must, phase P1; Vietnamese: "Lệnh vòng đời là Operation").

#### Scenario: Specified behaviour of FR-CP03-001
- **WHEN** the behaviour FR-CP03-001 covers is exercised
- **THEN** it behaves as BRD-05 specifies: `Runtime.Start/Stop(mode=acpi|poweroff|save)/Pause/Resume/Reset/DiscardSavedState` return an Operation at once; commands are idempotent (starting a running VM is a successful no-op); state preconditions are checked and errors list the valid actions.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/ConsoleImpl.cpp:2358-2947`

#### Scenario: Acceptance for FR-CP03-001
- **WHEN** the FR-CP03-001 acceptance check is run
- **THEN** Every command returns within 100 ms p95; an illegal transition returns VTX-VM-001 with the allowed actions.

### Requirement: Per-VM actor with deadlines and cancellation (FR-CP03-002)
Virtax SHALL provide "Per-VM actor with deadlines and cancellation" as specified by FR-CP03-002 (BRD-05, priority Must, phase P1; Vietnamese: "Actor theo VM có deadline và huỷ").

#### Scenario: Specified behaviour of FR-CP03-002
- **WHEN** the behaviour FR-CP03-002 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Each VM has one tokio task serialising every mutation (lifecycle, config apply, snapshot, media attach) with per-command deadlines (start 60 s, ACPI 90 s, save scaled by RAM); FFI calls run on a dedicated pool; one hung VM never affects another; the queue is inspectable.
- **AND** the VirtualBox reference for the current behaviour is `src/libs/xpcom18a4/ipc/ipcd/client/src/ipcDConnectService.cpp:89`

#### Scenario: Acceptance for FR-CP03-002
- **WHEN** the FR-CP03-002 acceptance check is run
- **THEN** With VM A's FFI call artificially hung, VM B still starts with ≤ 100 ms extra p95; deadline expiry → Operation FAILED VTX-VM-010 and the FFI thread is quarantined.

### Requirement: Unified state model (FR-CP03-003)
Virtax SHALL provide "Unified state model" as specified by FR-CP03-003 (BRD-05, priority Must, phase P1; Vietnamese: "Mô hình trạng thái thống nhất").

#### Scenario: Specified behaviour of FR-CP03-003
- **WHEN** the behaviour FR-CP03-003 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Map the 28 `MachineState` values to 8 UI states (Off, Saved, Starting, Running, Paused, Busy, Aborted, Stuck) per the table in the additional content; the API keeps the raw state; every state has a colour + text + icon chip and its valid actions (diagram 30-state-vm).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/idl/VirtualBox.xidl (MachineState)`

#### Scenario: Acceptance for FR-CP03-003
- **WHEN** the FR-CP03-003 acceptance check is run
- **THEN** Tests cover 28/28 values; the UI never shows a raw enum name; invalid buttons are hidden or disabled with a reason tooltip.

### Requirement: ACPI shutdown with countdown and escalation (FR-CP03-004)
Virtax SHALL provide "ACPI shutdown with countdown and escalation" as specified by FR-CP03-004 (BRD-05, priority Must, phase P1; Vietnamese: "Shut down ACPI có countdown và leo thang").

#### Scenario: Specified behaviour of FR-CP03-004
- **WHEN** the behaviour FR-CP03-004 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Shut down sends the ACPI power button and shows a "Shutting down… 0:45" chip; if the guest has not entered ACPI mode (`GuestEnteredACPIMode` = false) Save/Power off are offered at once; on timeout the per-VM policy applies (ask | save | power off); Phase 2 shuts down via the agent.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxAutostart/VBoxAutostartStop.cpp:226-262`

#### Scenario: Acceptance for FR-CP03-004
- **WHEN** the FR-CP03-004 acceptance check is run
- **THEN** A guest ignoring ACPI gets the choice after the 90 s default (interactive) or is saved (autostop); no command waits past its deadline.

### Requirement: Console close choices (FR-CP03-005)
Virtax SHALL provide "Console close choices" as specified by FR-CP03-005 (BRD-05, priority Must, phase P1; Vietnamese: "Lựa chọn khi đóng Console").

#### Scenario: Specified behaviour of FR-CP03-005
- **WHEN** the behaviour FR-CP03-005 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Closing the Console opens a sheet: keep running in the background (default), save state, shut down, power off (danger button needing confirmation) and "restore current snapshot after power off"; the choice is remembered per VM in `ui_prefs` with "Don't ask again"; replaces R-15.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIVMCloseDialog.cpp:143-182`

#### Scenario: Acceptance for FR-CP03-005
- **WHEN** the FR-CP03-005 acceptance check is run
- **THEN** Closing with the OS shortcut (⌘W/Alt+F4) never powers off; "keep running" leaves the VM Running with a chip in the Manager.

### Requirement: Event-driven supervisor (FR-CP03-006)
Virtax SHALL provide "Event-driven supervisor" as specified by FR-CP03-006 (BRD-05, priority Must, phase P1; Vietnamese: "Supervisor event-driven").

#### Scenario: Specified behaviour of FR-CP03-006
- **WHEN** the behaviour FR-CP03-006 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Watch VBoxHeadless (and VBoxSVC) PIDs with `pidfd` + epoll on Linux, kqueue `EVFILT_PROC` on macOS and `RegisterWaitForSingleObject` on Windows; no polling timers; an abnormal exit sets Aborted and emits an event.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/ClientWatcher.cpp:51,1005-1036`

#### Scenario: Acceptance for FR-CP03-006
- **WHEN** the FR-CP03-006 acceptance check is run
- **THEN** `kill -9` on the VM process yields Aborted + event within 1 s on all 6 host combos; the supervisor has 0 periodic wakeups with 5 VMs running.

### Requirement: Crash diagnosis and human errors (FR-CP03-007)
Virtax SHALL provide "Crash diagnosis and human errors" as specified by FR-CP03-007 (BRD-05, priority Must, phase P1; Vietnamese: "Chẩn đoán crash và lỗi dễ hiểu").

#### Scenario: Specified behaviour of FR-CP03-007
- **WHEN** the behaviour FR-CP03-007 covers is exercised
- **THEN** it behaves as BRD-05 specifies: On abnormal exit: collect exit code/signal, the last 200 `VBox.log` lines, the hardening log (Windows) and the Guru Meditation flag; map to VTX codes (VT-x busy, out of memory, disk full, hardening failure…) with Open log at error, Restart, Restart with safe settings and Copy technical details.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIErrorString.cpp`

#### Scenario: Acceptance for FR-CP03-007
- **WHEN** the FR-CP03-007 acceptance check is run
- **THEN** 10 common failure fixtures map to the right VTX code ≥ 9/10; ≥ 80% of test users understand the cause.

### Requirement: Start pre-flight and per-phase timings (FR-CP03-008)
Virtax SHALL provide "Start pre-flight and per-phase timings" as specified by FR-CP03-008 (BRD-05, priority Should, phase P1; Vietnamese: "Pre-flight khi start và đo thời gian từng phase").

#### Scenario: Specified behaviour of FR-CP03-008
- **WHEN** the behaviour FR-CP03-008 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Before start, check hypervisor availability/conflicts (KVM vs vboxdrv, Hyper-V), free RAM, existing disks/ISOs, port-forward clashes and encrypted-disk keys; the Operation records spawn → config → power-up → first-frame timings for tuning and reports.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/HostDrivers/Support/win/SUPR3HardenedMain-win.cpp:4787`

#### Scenario: Acceptance for FR-CP03-008
- **WHEN** the FR-CP03-008 acceptance check is run
- **THEN** A deleted ISO triggers a warning with an "Eject and start" fix instead of a hard error; click→first frame p50 ≤ 3 s (Linux/macOS), ≤ 4 s (Windows).

### Requirement: VBoxSVC residency and recovery (FR-CP03-009)
Virtax SHALL provide "VBoxSVC residency and recovery" as specified by FR-CP03-009 (BRD-05, priority Must, phase P1; Vietnamese: "Giữ VBoxSVC thường trú và phục hồi").

#### Scenario: Specified behaviour of FR-CP03-009
- **WHEN** the behaviour FR-CP03-009 covers is exercised
- **THEN** it behaves as BRD-05 specifies: `virtaxd` holds a long-lived reference so VBoxSVC never auto-exits after 5 s; it detects VBoxSVC death, marks affected VMs Aborted with the cause "Engine service crashed", restarts VBoxSVC (at most 3 times per 10 minutes) and reconciles state.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/xpcom/server.cpp:121-124; Frontends/VBoxHeadless/VBoxHeadless.cpp:136-149`

#### Scenario: Acceptance for FR-CP03-009
- **WHEN** the FR-CP03-009 acceptance check is run
- **THEN** Chaos-killing VBoxSVC: the UI shows the cause within 1 s, VBoxSVC is back within 5 s and the VM list reconciles 100%.

### Requirement: Reconcile on daemon restart (FR-CP03-010)
Virtax SHALL provide "Reconcile on daemon restart" as specified by FR-CP03-010 (BRD-05, priority Must, phase P1; Vietnamese: "Reconcile khi daemon khởi động lại").

#### Scenario: Specified behaviour of FR-CP03-010
- **WHEN** the behaviour FR-CP03-010 covers is exercised
- **THEN** it behaves as BRD-05 specifies: On `virtaxd` start: read the registry, query the engine, re-attach running VMs (Phase 1 via the VBoxSVC session; Phase 2 `virtax-vmm` via its own control socket) and flag VMs as "needs attention" instead of deleting them; no VM is killed by a daemon restart.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/VirtualBoxImpl.cpp:487-490`

#### Scenario: Acceptance for FR-CP03-010
- **WHEN** the FR-CP03-010 acceptance check is run
- **THEN** Restarting `virtaxd` with 5 running VMs kills 0 VMs and the Manager reconnects and resyncs within 2 s; Phase 2 meets NFR-00-07.

### Requirement: Autostart at login and at boot (FR-CP03-011)
Virtax SHALL provide "Autostart at login and at boot" as specified by FR-CP03-011 (BRD-05, priority Should, phase P1; Vietnamese: "Autostart khi đăng nhập và khi boot").

#### Scenario: Specified behaviour of FR-CP03-011
- **WHEN** the behaviour FR-CP03-011 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Each VM can "Start at login" (per-user virtaxd: systemd --user, LaunchAgent, Task Scheduler) or "Start at boot" for headless labs (systemd unit + linger, LaunchDaemon, Windows service with a service account), with order and delay; replaces VBoxAutostart and its root-owned config DB.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxAutostart/VBoxAutostartStart.cpp:70-90`

#### Scenario: Acceptance for FR-CP03-011
- **WHEN** the FR-CP03-011 acceptance check is run
- **THEN** 3 VMs with 0/10/20 s delays start in order ±2 s after login; a rebooted Linux arm64 server has them running within 60 s of network-online, no GUI.

### Requirement: Autostop on host shutdown, logout or sleep (FR-CP03-012)
Virtax SHALL provide "Autostop on host shutdown, logout or sleep" as specified by FR-CP03-012 (BRD-05, priority Should, phase P1; Vietnamese: "Autostop khi host tắt, đăng xuất hoặc ngủ").

#### Scenario: Specified behaviour of FR-CP03-012
- **WHEN** the behaviour FR-CP03-012 covers is exercised
- **THEN** it behaves as BRD-05 specifies: Per-VM policy: save (default), ACPI with timeout then save, or power off; uses logind inhibitors (Linux), NSWorkspace willPowerOff/willSleep (macOS) and `WM_QUERYENDSESSION` + `ShutdownBlockReasonCreate` (Windows); host sleep pauses VMs, wake resumes them and resyncs time.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxAutostart/VBoxAutostartStop.cpp:56-160`

#### Scenario: Acceptance for FR-CP03-012
- **WHEN** the FR-CP03-012 acceptance check is run
- **THEN** Shutting down the host with 2 running VMs leaves both Saved, none Aborted; closing the lid for 1 hour then reopening resumes VMs with guest clock skew ≤ 2 s.

### Requirement: As built — Stop requests on the virtax-vmm engine
`virtaxctl vm stop` SHALL press the guest's ACPI power button and wait up to 120 s for it to power off; if it does not, the operation SHALL fail with `VTX-ENG-012` and the machine SHALL keep running. `virtaxctl vm stop --force` SHALL power off immediately. Source: `crates/virtax-engine/src/vmm.rs`.

#### Scenario: Orderly and forced stop
- **WHEN** a running guest that handles the power button is stopped with and without `--force`
- **THEN** both reach `powered_off`, and only the first one runs the guest's power-button action
- **AND** this is verified by `apps/virtaxctl/tests/e2e.rs::kvm_guest_boots_through_virtaxctl_and_powers_off`
