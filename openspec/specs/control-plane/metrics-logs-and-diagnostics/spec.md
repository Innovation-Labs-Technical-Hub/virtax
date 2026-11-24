# Metrics, logs & diagnostics Specification

## Purpose
Metrics stream sampled only while subscribed, structured logs, one-click diagnostics bundle, opt-in crash reports. Module `CP-11` (Metrics, logs & diagnostics; Vietnamese: "Metrics, log & chẩn đoán") from BRD-09, phase P1, priority Should. Owner crates: virtaxd (metrics hub), tracing. Depends on: `control-plane/control-plane-api-and-ipc`. VirtualBox reference: `src/VBox/Main/src-server/Performance*.cpp, Frontends/VBoxBugReport, VMM STAM`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Metrics hub and Metrics.Watch only while subscribed (FR-CP11-001)
Virtax SHALL provide "Metrics hub and Metrics.Watch only while subscribed" as specified by FR-CP11-001 (BRD-09, priority Must, phase P1; Vietnamese: "Metrics hub và Metrics.Watch chỉ khi có subscriber").

#### Scenario: Specified behaviour of FR-CP11-001
- **WHEN** the behaviour FR-CP11-001 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `Metrics.Watch(resources, names, interval ≥ 250 ms, default 1 s)` streams samples by value; the daemon keeps a ring buffer per series (1 hour at 1 Hz); the sampler starts with the first subscriber and stops within 1 s after the last leaves; Phase 1 reads `IPerformanceCollector` on the FFI pool plus OS counters.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/PerformanceImpl.cpp:195-205,677-743`

#### Scenario: Acceptance for FR-CP11-001
- **WHEN** the FR-CP11-001 acceptance check is run
- **THEN** No subscriber → 0 sampling wakeups; the first sample arrives within 1.5 s of subscribing; 10 subscribers on one VM share 1 sampler.

### Requirement: Metric catalogue and units (FR-CP11-002)
Virtax SHALL provide "Metric catalogue and units" as specified by FR-CP11-002 (BRD-09, priority Should, phase P1; Vietnamese: "Catalog metric và đơn vị").

#### Scenario: Specified behaviour of FR-CP11-002
- **WHEN** the behaviour FR-CP11-002 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Stable names `vm.cpu.guest_pct`, `vm.cpu.vmm_pct`, `vm.mem.used_bytes`, `vm.mem.balloon_bytes`, `vm.disk.read_bps`, `vm.net.rx_bps`, `host.cpu.pct`… mapped from VBox names (`CPU/Load/User`, `Guest/RAM/Usage/Balloon`, `Net/Rate/Rx`); every metric has a unit and a bilingual description.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:12344-12418`

#### Scenario: Acceptance for FR-CP11-002
- **WHEN** the FR-CP11-002 acceptance check is run
- **THEN** `virtaxctl metrics list --json` lists 100% of metrics with units; the map covers every VBox metric the UI uses.

### Requirement: Low-overhead host sampling (FR-CP11-003)
Virtax SHALL provide "Low-overhead host sampling" as specified by FR-CP11-003 (BRD-09, priority Should, phase P1; Vietnamese: "Lấy mẫu host chi phí thấp").

#### Scenario: Specified behaviour of FR-CP11-003
- **WHEN** the behaviour FR-CP11-003 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Linux reads `/proc` with `pread` on pre-opened fds (no `fopen` per sample), macOS uses `host_statistics64`/ `proc_pid_rusage`, Windows `GetProcessTimes`/PDH; every VM is sampled in one tick to avoid scattered wakeups.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/linux/PerformanceLinux.cpp:123-160,286-346`

#### Scenario: Acceptance for FR-CP11-003
- **WHEN** the FR-CP11-003 acceptance check is run
- **THEN** Sampling 10 VMs at 1 Hz costs ≤ 0.2% of one core; sampling causes exactly 1 wakeup/s regardless of VM count.

### Requirement: Structured logging with tracing (FR-CP11-004)
Virtax SHALL provide "Structured logging with tracing" as specified by FR-CP11-004 (BRD-09, priority Must, phase P1; Vietnamese: "Log có cấu trúc với tracing").

#### Scenario: Specified behaviour of FR-CP11-004
- **WHEN** the behaviour FR-CP11-004 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `virtaxd`, Manager, Console and `virtaxctl` write JSON logs with `tracing`, with spans per RPC and per Operation (`op_id`) and a per-VM file `<vm>/Logs/virtax.jsonl`; log levels via `VIRTAX_LOG` (EnvFilter) and changeable live through the API without a restart.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Runtime/common/log/log.cpp (RTLog)`

#### Scenario: Acceptance for FR-CP11-004
- **WHEN** the FR-CP11-004 acceptance check is run
- **THEN** Fetching all logs for an Operation by `op_id` takes ≤ 100 ms; info-level logging costs ≤ 1% CPU under load.

### Requirement: Log rotation and retention (FR-CP11-005)
Virtax SHALL provide "Log rotation and retention" as specified by FR-CP11-005 (BRD-09, priority Should, phase P1; Vietnamese: "Rotation và retention log").

#### Scenario: Specified behaviour of FR-CP11-005
- **WHEN** the behaviour FR-CP11-005 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Size-based rolling (10 MB × 5 files per component and per VM by default), old files compressed, a 500 MB total budget; the `VBox.log` history count (`LogHistoryCount`, default 3) is configurable through the engine; settings live in Preferences.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:4896-4990`

#### Scenario: Acceptance for FR-CP11-005
- **WHEN** the FR-CP11-005 acceptance check is run
- **THEN** Writing 2 GB of synthetic logs never exceeds the 500 MB budget; the file being written is never deleted.

### Requirement: Logs API (FR-CP11-006)
Virtax SHALL provide "Logs API" as specified by FR-CP11-006 (BRD-09, priority Should, phase P1; Vietnamese: "Logs API").

#### Scenario: Specified behaviour of FR-CP11-006
- **WHEN** the behaviour FR-CP11-006 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `Logs.List/Read(range)/Tail(stream)/Search` for `VBox.log*`, the hardening log, daemon logs and per-Operation logs; the daemon keeps a line index for range reads; it backs the Logs tab (UI-06, BRD-05) and `virtaxctl logs`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:6311-6340 (readLog)`

#### Scenario: Acceptance for FR-CP11-006
- **WHEN** the FR-CP11-006 acceptance check is run
- **THEN** Reading 1,000 lines from the middle of a 256 MB file takes ≤ 20 ms; `Tail` pushes new lines within 500 ms.

### Requirement: One-click diagnostics bundle (FR-CP11-007)
Virtax SHALL provide "One-click diagnostics bundle" as specified by FR-CP11-007 (BRD-09, priority Must, phase P1; Vietnamese: "Gói chẩn đoán một click").

#### Scenario: Specified behaviour of FR-CP11-007
- **WHEN** the behaviour FR-CP11-007 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `virtaxctl diag bundle` / a Health-page button collects daemon logs, selected VM logs (`VBox.log*`, hardening), redacted `machine.toml`/`.vbox`, host info (OS, CPU features, KVM/HVF/WHP, Secure Boot), versions, the last 100 Operations and crash records into a zip.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxBugReport/VBoxBugReport.cpp:612-648`

#### Scenario: Acceptance for FR-CP11-007
- **WHEN** the FR-CP11-007 acceptance check is run
- **THEN** The bundle is created within 10 s and ≤ 50 MB by default; it contains a manifest listing every file and why it is included.

### Requirement: Redaction and preview (FR-CP11-008)
Virtax SHALL provide "Redaction and preview" as specified by FR-CP11-008 (BRD-09, priority Must, phase P1; Vietnamese: "Che dữ liệu nhạy cảm và xem trước").

#### Scenario: Specified behaviour of FR-CP11-008
- **WHEN** the behaviour FR-CP11-008 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Redact unattended passwords, product keys, disk encryption keys/passwords, home paths → `~`, usernames/hostnames → tokens, MACs (optional) and guest properties matching secret patterns; show the file list and a preview before saving or sending.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxBugReport/VBoxBugReport.cpp:619-645`

#### Scenario: Acceptance for FR-CP11-008
- **WHEN** the FR-CP11-008 acceptance check is run
- **THEN** A corpus with 200 seeded secrets leaves 0 in the bundle; users can drop individual files before saving.

### Requirement: Opt-in crash reports (FR-CP11-009)
Virtax SHALL provide "Opt-in crash reports" as specified by FR-CP11-009 (BRD-09, priority Should, phase P1; Vietnamese: "Crash report opt-in").

#### Scenario: Specified behaviour of FR-CP11-009
- **WHEN** the behaviour FR-CP11-009 covers is exercised
- **THEN** it behaves as BRD-09 specifies: A panic hook + minidumps (`crash-handler`/`minidumper` crates) for `virtaxd`, Manager and Console, stored locally and sent to the project endpoint only after opt-in and preview; server-side symbolication; crash-free session rate computed.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP11-009
- **WHEN** the FR-CP11-009 acceptance check is run
- **THEN** A fresh install has 0 egress besides the update check (FR-GEN-008); a simulated crash yields a minidump within 2 s, shown at next launch.

### Requirement: Prometheus exporter for headless labs (FR-CP11-010)
Virtax SHALL provide "Prometheus exporter for headless labs" as specified by FR-CP11-010 (BRD-09, priority Should, phase P1; Vietnamese: "Prometheus exporter cho lab headless").

#### Scenario: Specified behaviour of FR-CP11-010
- **WHEN** the behaviour FR-CP11-010 covers is exercised
- **THEN** it behaves as BRD-09 specifies: An optional `/metrics` (OpenMetrics) endpoint, off by default, bound to 127.0.0.1 on a configurable port with VM/group labels; each scrape acts as a subscriber with interval = scrape period; binding beyond loopback needs an explicit flag + warning.
- **AND** the VirtualBox reference for the current behaviour is `new (VBoxBalloonCtrl/VBoxWatchdog.cpp:107-124 polls PerformanceCollector)`

#### Scenario: Acceptance for FR-CP11-010
- **WHEN** the FR-CP11-010 acceptance check is run
- **THEN** A 15 s Prometheus scrape of 20 VMs costs ≤ 0.1% CPU; disabling the exporter closes the port within 1 s.

### Requirement: Health page and virtaxctl doctor (FR-CP11-011)
Virtax SHALL provide "Health page and virtaxctl doctor" as specified by FR-CP11-011 (BRD-09, priority Should, phase P1; Vietnamese: "Trang Health và virtaxctl doctor").

#### Scenario: Specified behaviour of FR-CP11-011
- **WHEN** the behaviour FR-CP11-011 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Checks hypervisor availability and conflicts (KVM vs vboxdrv, Hyper-V/VBS, HVF entitlement), engine version, VBoxSVC status, free space, socket permissions and time sync; each item has a status + fix; `virtaxctl doctor --json` exits non-zero on failures.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/VMM/VMMR3/HMR3-x86.cpp:717`

#### Scenario: Acceptance for FR-CP11-011
- **WHEN** the FR-CP11-011 acceptance check is run
- **THEN** 15 checks run within 3 s; a host whose `kvm_intel` holds VT-x reports VTX-HV-002 with guidance.

### Requirement: OpenTelemetry trace export for developers (FR-CP11-012)
Virtax SHALL provide "OpenTelemetry trace export for developers" as specified by FR-CP11-012 (BRD-09, priority Could, phase P1; Vietnamese: "Xuất trace OpenTelemetry cho developer").

#### Scenario: Specified behaviour of FR-CP11-012
- **WHEN** the behaviour FR-CP11-012 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Optionally export spans (RPCs, Operations, VM start phases) over OTLP to a localhost collector for performance analysis; off by default.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP11-012
- **WHEN** the FR-CP11-012 acceptance check is run
- **THEN** With OTLP on, a VM start trace shows every phase in Jaeger; off → 0 outbound connections.
