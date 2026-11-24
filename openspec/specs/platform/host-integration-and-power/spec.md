# Host integration & power Specification

## Purpose
Minimal privileged helper (TAP/vmnet/HNS/USB), P/E-core QoS, battery mode, hypervisor-conflict detection. Module `PL-03` (Host integration & power; Vietnamese: "Tích hợp host & năng lượng") from BRD-12, phase P1, priority Should. Owner crates: virtax-helper, virtax-platform. Depends on: `control-plane/security-and-encryption`. VirtualBox reference: `src/VBox/HostDrivers (Support, VBoxNetFlt, VBoxNetAdp, VBoxUSB), Runtime/r3 (timers, QoS)`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: virtax-helper operations per OS (FR-PL03-001)
Virtax SHALL provide "virtax-helper operations per OS" as specified by FR-PL03-001 (BRD-12, priority Must, phase P1; Vietnamese: "Thao tác của virtax-helper trên từng OS").

#### Scenario: Specified behaviour of FR-PL03-001
- **WHEN** the behaviour FR-PL03-001 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Linux: create/delete TAP and bridges via netlink (rtnetlink), set the owner UID, open vhost-net permissions, no shell-outs. macOS: vmnet interfaces via an SMAppService helper when the entitlement is missing. Windows: HNS/HCN networks for NAT/internal, WinTun adapters for host-only/bridged. USB claim for CP-08 (P2).
- **AND** the VirtualBox reference for the current behaviour is `HostDrivers/adpctl/VBoxNetAdpCtl.cpp:89-90,359,732`

#### Scenario: Acceptance for FR-PL03-001
- **WHEN** the FR-PL03-001 acceptance check is run
- **THEN** Host-only network creation ≤ 2 s on each OS; the helper API has ≤ 15 methods; every action is audited (CP-12).

### Requirement: OS-native helper activation (FR-PL03-002)
Virtax SHALL provide "OS-native helper activation" as specified by FR-PL03-002 (BRD-12, priority Must, phase P1; Vietnamese: "Kích hoạt helper theo chuẩn OS").

#### Scenario: Specified behaviour of FR-PL03-002
- **WHEN** the behaviour FR-PL03-002 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Linux: polkit actions `org.virtax.helper.*` (auth_admin_keep the first time, optional group rule). macOS: an SMAppService daemon approved in System Settings. Windows: a demand-start service as LocalService + specific privileges, with an ACL'd pipe. The helper exits after 60 s idle.
- **AND** the VirtualBox reference for the current behaviour is `Support/win/VBoxSup.inf:76; VBoxMergeAppCA.wxi:28-35`

#### Scenario: Acceptance for FR-PL03-002
- **WHEN** the FR-PL03-002 acceptance check is run
- **THEN** The first bridged network shows the OS consent once; later operations do not prompt; the helper is not running when idle.

### Requirement: Hypervisor availability and conflict detection (FR-PL03-003)
Virtax SHALL provide "Hypervisor availability and conflict detection" as specified by FR-PL03-003 (BRD-12, priority Must, phase P1; Vietnamese: "Phát hiện hypervisor và xung đột").

#### Scenario: Specified behaviour of FR-PL03-003
- **WHEN** the behaviour FR-PL03-003 covers is exercised
- **THEN** it behaves as BRD-12 specifies: At daemon start (then cached): Windows — Hyper-V/VBS/HVCI on, the WHP feature (WinHvPlatform.dll + CPUID), VT-x/AMD-V in firmware, VBoxSup loaded; Linux — `/dev/kvm` present/accessible, `kvm_intel`/`kvm_amd` loaded (HM conflict), vboxdrv; macOS — `kern.hv_support`, entitlement, version for `hv_gic`. Results appear on the Health page and in `virtaxctl host check --json`.
- **AND** the VirtualBox reference for the current behaviour is `HMR3-x86.cpp:709-717; NEMR3Native-win-x86.cpp:605-634; NEMR3NativeTemplate-linux.cpp.h:1051-1055`

#### Scenario: Acceptance for FR-PL03-003
- **WHEN** the FR-PL03-003 acceptance check is run
- **THEN** The check takes ≤ 500 ms at start and ≤ 20 ms cached; all 6 conflict scenarios (additional table) yield the right VTX-HV code.

### Requirement: Guided quick fixes for conflicts (FR-PL03-004)
Virtax SHALL provide "Guided quick fixes for conflicts" as specified by FR-PL03-004 (BRD-12, priority Must, phase P1; Vietnamese: "Quick-fix có hướng dẫn cho xung đột").

#### Scenario: Specified behaviour of FR-PL03-004
- **WHEN** the behaviour FR-PL03-004 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Each conflict has a fix: "Switch to KVM backend" (the legacy engine runs NEM/driverless when KVM is loaded), "Add me to the kvm group" (via the helper, with a re-login hint), "Enable Windows Hypervisor Platform" (DISM, needs a reboot, confirmed), "Enable VT-x in the BIOS" (vendor-specific steps). Every action needs a user click.
- **AND** the VirtualBox reference for the current behaviour is `Installer/linux/VBox.sh:69-90; UIErrorString.cpp`

#### Scenario: Acceptance for FR-PL03-004
- **WHEN** the FR-PL03-004 acceptance check is run
- **THEN** UX test: ≥ 80% of users resolve conflicts without docs; every quick fix logs to the Task center with "Copy technical details".

### Requirement: Power profiles (FR-PL03-005)
Virtax SHALL provide "Power profiles" as specified by FR-PL03-005 (BRD-12, priority Must, phase P1; Vietnamese: "Profile nguồn điện").

#### Scenario: Specified behaviour of FR-PL03-005
- **WHEN** the behaviour FR-PL03-005 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Performance / Balanced / Battery, picked by power source with per-VM override; maps to thread QoS, halt-polling (P2), `HostHzMax`/extradata (P1), preview rate, metric sampling and background-VM throttling (additional table). A status chip shows the current profile.
- **AND** the VirtualBox reference for the current behaviour is `TMR3.cpp:639; VMR3Emt.cpp:462-512`

#### Scenario: Acceptance for FR-PL03-005
- **WHEN** the FR-PL03-005 acceptance check is run
- **THEN** Unplugging → profile switches within 5 s; an idle VM on Battery uses ≤ 70% of the energy of Balanced (RAPL/powermetrics).

### Requirement: EcoQoS and P-cores on Windows (FR-PL03-006)
Virtax SHALL provide "EcoQoS and P-cores on Windows" as specified by FR-PL03-006 (BRD-12, priority Must, phase P1; Vietnamese: "EcoQoS và P-core trên Windows").

#### Scenario: Specified behaviour of FR-PL03-006
- **WHEN** the behaviour FR-PL03-006 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Focused VM: `PROCESS_POWER_THROTTLING_EXECUTION_SPEED` off (HighQoS) for the VM process and vCPU threads; background/headless VMs: EcoQoS on; UI threads normal. P1 applies it per process to VBoxHeadless, P2 per thread inside `virtax-vmm`.
- **AND** the VirtualBox reference for the current behaviour is `grep: no ProcessPowerThrottling in tree; analysis 05 F7`

#### Scenario: Acceptance for FR-PL03-006
- **WHEN** the FR-PL03-006 acceptance check is run
- **THEN** ETW on an Intel hybrid CPU: the focused VM's vCPUs run on P-cores ≥ 90% of the time; background VMs carry EcoQoS; focused guest benchmarks do not regress.

### Requirement: QoS classes on macOS (FR-PL03-007)
Virtax SHALL provide "QoS classes on macOS" as specified by FR-PL03-007 (BRD-12, priority Must, phase P1; Vietnamese: "QoS class trên macOS").

#### Scenario: Specified behaviour of FR-PL03-007
- **WHEN** the behaviour FR-PL03-007 covers is exercised
- **THEN** it behaves as BRD-12 specifies: `pthread_set_qos_class_self_np`: vCPUs are USER_INTERACTIVE while the Console is focused, USER_INITIATED when visible, UTILITY in the background; I/O threads UTILITY; the daemon UTILITY/BACKGROUND. P1 applies process-level policy to VBoxHeadless (`setpriority`/`taskpolicy`).
- **AND** the VirtualBox reference for the current behaviour is `Runtime/r3/darwin/sched-darwin.cpp:330-351`

#### Scenario: Acceptance for FR-PL03-007
- **WHEN** the FR-PL03-007 acceptance check is run
- **THEN** powermetrics: background VMs run on E-cores and the focused VM on P-cores; the idle Manager's Energy Impact is "Low".

### Requirement: Scheduling hints on Linux (FR-PL03-008)
Virtax SHALL provide "Scheduling hints on Linux" as specified by FR-PL03-008 (BRD-12, priority Should, phase P1; Vietnamese: "Gợi ý lập lịch trên Linux").

#### Scenario: Specified behaviour of FR-PL03-008
- **WHEN** the behaviour FR-PL03-008 covers is exercised
- **THEN** it behaves as BRD-12 specifies: One systemd user scope per VM with `CPUWeight`/`IOWeight` and `cpu.max` for the execution cap; `sched_setattr` uclamp (util_min for focused vCPUs) when allowed; nice/ionice for background VMs; no `CAP_SYS_NICE` needed.
- **AND** the VirtualBox reference for the current behaviour is `Runtime/r3/linux/sched-linux.cpp:421-465; EMR3Nem.cpp:475-490`

#### Scenario: Acceptance for FR-PL03-008
- **WHEN** the FR-PL03-008 acceptance check is run
- **THEN** Background VMs show `CPUWeight=50` via `systemctl --user show`; a 50% cap measures within ±5%; works without root.

### Requirement: Timer-resolution hygiene on Windows (FR-PL03-009)
Virtax SHALL provide "Timer-resolution hygiene on Windows" as specified by FR-PL03-009 (BRD-12, priority Must, phase P1; Vietnamese: "Vệ sinh timer resolution trên Windows").

#### Scenario: Specified behaviour of FR-PL03-009
- **WHEN** the behaviour FR-PL03-009 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Virtax processes never call system-wide `timeBeginPeriod(1)`/`NtSetTimerResolution`; they use `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`. The Health page reports when the legacy engine (VBoxSup's GIP) raises the system timer; `virtax-vmm` in P2 never does.
- **AND** the VirtualBox reference for the current behaviour is `timer-win.cpp:260-290; timer-r0drv-nt.cpp:1070-1085`

#### Scenario: Acceptance for FR-PL03-009
- **WHEN** the FR-PL03-009 acceptance check is run
- **THEN** `powercfg /energy` with Manager + `virtaxd` + an idle `virtax-vmm` VM: no timer requests from Virtax processes; the system timer stays at 15.6 ms with no legacy VM.

### Requirement: Daemon integration with the OS (FR-PL03-010)
Virtax SHALL provide "Daemon integration with the OS" as specified by FR-PL03-010 (BRD-12, priority Must, phase P1; Vietnamese: "Tích hợp daemon với OS").

#### Scenario: Specified behaviour of FR-PL03-010
- **WHEN** the behaviour FR-PL03-010 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Per-user `virtaxd`: a `systemd --user` unit (optional socket activation), a macOS LaunchAgent, an MSIX startup task/Task Scheduler on Windows; event-driven process watching (pidfd, kqueue, `WaitForSingleObject`), no polling; VM autostart (CP-03).
- **AND** the VirtualBox reference for the current behaviour is `ClientWatcher.cpp:51; ObjectsTracker.cpp:1019; VBoxSDS.cpp:29-68`

#### Scenario: Acceptance for FR-PL03-010
- **WHEN** the FR-PL03-010 acceptance check is run
- **THEN** Daemon start ≤ 300 ms; with no VMs it idles at ≤ 5 wakeups/s; after a host reboot autostart VMs come up.

### Requirement: Host sleep and resume (FR-PL03-011)
Virtax SHALL provide "Host sleep and resume" as specified by FR-PL03-011 (BRD-12, priority Should, phase P1; Vietnamese: "Sleep/resume của host").

#### Scenario: Specified behaviour of FR-PL03-011
- **WHEN** the behaviour FR-PL03-011 covers is exercised
- **THEN** it behaves as BRD-12 specifies: On host suspend: pause or keep VMs per policy; after resume resync guest time (Guest Additions in P1, the agent in P2) and re-arm timers; optionally block sleep while a VM is marked "busy" (systemd-inhibit, IOPMAssertion, `SetThreadExecutionState`).
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/HostPower.cpp:63-118`

#### Scenario: Acceptance for FR-PL03-011
- **WHEN** the FR-PL03-011 acceptance check is run
- **THEN** After 1 h of sleep, guest clock skew is fixed within 5 s of resume; 0 VM crashes; sleep blocking works on all 3 OSes.

### Requirement: Coexistence with WSL2, Docker and other VMMs (FR-PL03-012)
Virtax SHALL provide "Coexistence with WSL2, Docker and other VMMs" as specified by FR-PL03-012 (BRD-12, priority Should, phase P1; Vietnamese: "Chạy chung với WSL2, Docker và VMM khác").

#### Scenario: Specified behaviour of FR-PL03-012
- **WHEN** the behaviour FR-PL03-012 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Windows with Hyper-V on: use WHP (Engine Pack in P1, `virtax-vmm` in P2), never ask to turn Hyper-V off. Linux: never unload kvm modules; detect VMware/VirtualBox modules holding VT-x and explain. macOS: multiple HVF VMs alongside other virtualisation apps.
- **AND** the VirtualBox reference for the current behaviour is `NEMR3.cpp:238-250; SUPDrv-linux.c:93-115`

#### Scenario: Acceptance for FR-PL03-012
- **WHEN** the FR-PL03-012 acceptance check is run
- **THEN** An Virtax VM runs concurrently with WSL2 + Docker Desktop on Windows 11 and with libvirt/QEMU VMs on Linux; no prompt to disable Hyper-V.
