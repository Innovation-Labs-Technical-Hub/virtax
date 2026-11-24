# virtaxctl CLI Specification

## Purpose
VBoxManage-equivalent commands, JSON output, shell completion, VBoxManage translation aliases. Module `CP-15` (virtaxctl CLI; Vietnamese: "CLI virtaxctl") from BRD-09, phase P1, priority Must. Owner crates: virtaxctl. Depends on: `control-plane/control-plane-api-and-ipc`. VirtualBox reference: `src/VBox/Frontends/VBoxManage (≈54 commands)`. Implementation status (2026-11-24): partial — `virtaxctl vm|ops|events|version`, `--json`, documented exit codes, wait/`--no-wait`; no completions, VBoxManage shim or bulk selectors yet.

## Requirements

### Requirement: Command structure and VBoxManage parity (FR-CP15-001)
Virtax SHALL provide "Command structure and VBoxManage parity" as specified by FR-CP15-001 (BRD-09, priority Must, phase P1; Vietnamese: "Cấu trúc lệnh và tương đương VBoxManage").

#### Scenario: Specified behaviour of FR-CP15-001
- **WHEN** the behaviour FR-CP15-001 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `virtaxctl <noun> <verb>` (vm, snapshot, media, net, appliance, unattended, guest, metrics, events, ops, logs, diag, host) covers ≈54 VBoxManage commands per the mapping table; unsupported commands are documented with a reason.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManage.cpp:197-278`

#### Scenario: Acceptance for FR-CP15-001
- **WHEN** the FR-CP15-001 acceptance check is run
- **THEN** The CI parity matrix shows 100% of P1 VBoxManage commands with an `virtaxctl` equivalent or a reasoned "not supported" entry.

### Requirement: Machine-readable output (FR-CP15-002)
Virtax SHALL provide "Machine-readable output" as specified by FR-CP15-002 (BRD-09, priority Must, phase P1; Vietnamese: "Output cho máy đọc").

#### Scenario: Specified behaviour of FR-CP15-002
- **WHEN** the behaviour FR-CP15-002 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `--json` (schema = the proto JSON mapping, versioned), `--output table|json|yaml|name`, `-q` for names only; JSON mode prints no human text on stdout and sends warnings to stderr; `events watch` uses NDJSON.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManageInfo.cpp (--machinereadable)`

#### Scenario: Acceptance for FR-CP15-002
- **WHEN** the FR-CP15-002 acceptance check is run
- **THEN** `virtaxctl vm list --json` validates against the published JSON schema; e2e tests confirm stdout holds only JSON.

### Requirement: Exit codes and errors (FR-CP15-003)
Virtax SHALL provide "Exit codes and errors" as specified by FR-CP15-003 (BRD-09, priority Must, phase P1; Vietnamese: "Exit code và lỗi").

#### Scenario: Specified behaviour of FR-CP15-003
- **WHEN** the behaviour FR-CP15-003 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Documented exit codes: 0 ok, 1 generic, 2 usage, 3 not found, 4 conflict/etag, 5 state/precondition, 6 timeout, 7 cancelled, 10 engine error; errors print the VTX code + a localized sentence + a hint; with `--json` the error is an object.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManage.cpp (RTEXITCODE_*)`

#### Scenario: Acceptance for FR-CP15-003
- **WHEN** the FR-CP15-003 acceptance check is run
- **THEN** Every exit code has an e2e test; starting a running VM with `--strict` exits 5, without `--strict` 0 (idempotent).

### Requirement: Long-running operation handling (FR-CP15-004)
Virtax SHALL provide "Long-running operation handling" as specified by FR-CP15-004 (BRD-09, priority Must, phase P1; Vietnamese: "Xử lý thao tác dài").

#### Scenario: Specified behaviour of FR-CP15-004
- **WHEN** the behaviour FR-CP15-004 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Long commands wait by default with a TTY progress bar driven by `Operations.Watch`; `--no-wait` returns the Operation name; `virtaxctl ops wait|cancel|list`; Ctrl-C asks whether to cancel the Operation or just detach.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManage.cpp:400-478`

#### Scenario: Acceptance for FR-CP15-004
- **WHEN** the FR-CP15-004 acceptance check is run
- **THEN** Server logs confirm 0 polls; the progress bar updates at least once per second; Ctrl-C → detach keeps the Operation running.

### Requirement: Shell completions, help and man pages (FR-CP15-005)
Virtax SHALL provide "Shell completions, help and man pages" as specified by FR-CP15-005 (BRD-09, priority Must, phase P1; Vietnamese: "Shell completion, help và man page").

#### Scenario: Specified behaviour of FR-CP15-005
- **WHEN** the behaviour FR-CP15-005 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Completions for bash, zsh, fish and PowerShell generated with `clap_complete`, dynamic VM/snapshot names from the daemon (`virtaxctl __complete`); bilingual `--help` per locale with examples; man pages generated with `clap_mangen`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManageHelp.cpp`

#### Scenario: Acceptance for FR-CP15-005
- **WHEN** the FR-CP15-005 acceptance check is run
- **THEN** Dynamic completion answers within 100 ms with 500 VMs; .deb/.rpm/brew packages install completions; 100% of commands have help examples.

### Requirement: virtaxctl vboxmanage translation shim (FR-CP15-006)
Virtax SHALL provide "virtaxctl vboxmanage translation shim" as specified by FR-CP15-006 (BRD-09, priority Should, phase P1; Vietnamese: "Shim dịch virtaxctl vboxmanage").

#### Scenario: Specified behaviour of FR-CP15-006
- **WHEN** the behaviour FR-CP15-006 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `virtaxctl vboxmanage <args…>` parses VBoxManage syntax for supported commands (list vms, showvminfo --machinereadable, startvm --type headless, controlvm, snapshot, common modifyvm flags, createvm/registervm, import/export, clonevm, storageattach, natnetwork…), runs them via the API, prints compatible output plus the equivalent `virtaxctl` command on stderr (`--explain`); it never installs a `VBoxManage` alias that would shadow the real one.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManageModifyVM.cpp, VBoxManageControlVM.cpp`

#### Scenario: Acceptance for FR-CP15-006
- **WHEN** the FR-CP15-006 acceptance check is run
- **THEN** Across 50 public scripts ≥ 90% of calls translate; an unsupported flag exits 2 naming the flag with a hint.

### Requirement: Fast startup, no cold start (FR-CP15-007)
Virtax SHALL provide "Fast startup, no cold start" as specified by FR-CP15-007 (BRD-09, priority Must, phase P1; Vietnamese: "Khởi động nhanh, không cold start").

#### Scenario: Specified behaviour of FR-CP15-007
- **WHEN** the behaviour FR-CP15-007 covers is exercised
- **THEN** it behaves as BRD-09 specifies: A static `virtaxctl` binary reaches its first RPC within 30 ms; the resident daemon means no call waits for VBoxSVC to start (unlike VirtualBox's 5 s auto-shutdown).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/xpcom/server.cpp:124; win/svcmain.cpp:103`

#### Scenario: Acceptance for FR-CP15-007
- **WHEN** the FR-CP15-007 acceptance check is run
- **THEN** `virtaxctl vm list` with 50 VMs has p95 ≤ 80 ms wall time; 100 calls spaced 10 s apart never exceed 200 ms.

### Requirement: Event and metric watch commands (FR-CP15-008)
Virtax SHALL provide "Event and metric watch commands" as specified by FR-CP15-008 (BRD-09, priority Should, phase P1; Vietnamese: "Lệnh watch event và metrics").

#### Scenario: Specified behaviour of FR-CP15-008
- **WHEN** the behaviour FR-CP15-008 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `virtaxctl events watch --since N --filter … --json` (NDJSON) and `virtaxctl metrics watch <vm> --interval 1s`; they reconnect and resume from the last seq when the daemon restarts.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManageMetrics.cpp`

#### Scenario: Acceptance for FR-CP15-008
- **WHEN** the FR-CP15-008 acceptance check is run
- **THEN** Restarting the daemon mid-watch reconnects within 2 s with no lost or duplicated events (checked by seq).

### Requirement: Selectors, bulk actions and dry-run (FR-CP15-009)
Virtax SHALL provide "Selectors, bulk actions and dry-run" as specified by FR-CP15-009 (BRD-09, priority Should, phase P1; Vietnamese: "Selector, thao tác hàng loạt và dry-run").

#### Scenario: Specified behaviour of FR-CP15-009
- **WHEN** the behaviour FR-CP15-009 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Select VMs by name, UUID, prefix, `--group`, `--tag` or `--state`; bulk with `--parallel N`; destructive bulk actions list exactly what is affected and need `--yes`; `--dry-run` for every mutating command.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManageMisc.cpp`

#### Scenario: Acceptance for FR-CP15-009
- **WHEN** the FR-CP15-009 acceptance check is run
- **THEN** `virtaxctl vm stop --group /lab --dry-run` lists the right VMs with 0 changes; bulk delete without `--yes` exits 2.

### Requirement: Scripting stability contract (FR-CP15-010)
Virtax SHALL provide "Scripting stability contract" as specified by FR-CP15-010 (BRD-09, priority Must, phase P1; Vietnamese: "Hợp đồng ổn định cho script").

#### Scenario: Specified behaviour of FR-CP15-010
- **WHEN** the behaviour FR-CP15-010 covers is exercised
- **THEN** it behaves as BRD-09 specifies: CLI flags and JSON fields follow semver; deprecations warn for ≥ 2 minor releases before removal; `virtaxctl version --json` includes API and schema versions; the changelog has a "scripting changes" section.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP15-010
- **WHEN** the FR-CP15-010 acceptance check is run
- **THEN** JSON output snapshot tests across minor releases show 0 removed/retyped fields; deprecated flags warn on stderr.

### Requirement: CLI package for headless servers (FR-CP15-011)
Virtax SHALL provide "CLI package for headless servers" as specified by FR-CP15-011 (BRD-09, priority Should, phase P1; Vietnamese: "Gói CLI cho server headless").

#### Scenario: Specified behaviour of FR-CP15-011
- **WHEN** the behaviour FR-CP15-011 covers is exercised
- **THEN** it behaves as BRD-09 specifies: An `virtax-cli` package (virtaxctl + virtaxd, no Manager/Console) for Linux x64/arm64 servers; `virtaxctl daemon start|stop|status`; works over SSH without a graphical session.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxHeadless/VBoxHeadless.cpp`

#### Scenario: Acceptance for FR-CP15-011
- **WHEN** the FR-CP15-011 acceptance check is run
- **THEN** On a GUI-less Ubuntu Server 24.04 arm64, install the package, then create and start a headless VM with `virtaxctl` over SSH only.

### Requirement: As built — One JSON document per invocation
With `--json`, every `virtaxctl` command except `events watch` SHALL print exactly one JSON document on stdout, including on failure, when it prints an `error` object with `exit_code`, `code`, `message`, `hint` and `etag`. `events watch` SHALL print NDJSON. Source: `apps/virtaxctl/src/main.rs`.

#### Scenario: Failure output is JSON
- **WHEN** `virtaxctl --json vm validate` reports invalid manifest content
- **THEN** stdout parses as a single JSON document
- **AND** this is verified by `apps/virtaxctl/tests/e2e.rs::validate_reports_line_and_column_without_daemon`

### Requirement: As built — Event watch resumes after a restart
`virtaxctl events watch` SHALL reconnect with a backoff from 100 ms to 500 ms and resume after the last `seq` it printed. Source: `apps/virtaxctl/src/main.rs`.

#### Scenario: Daemon restart mid-watch
- **WHEN** the daemon restarts while a watch is running
- **THEN** the printed sequence numbers stay contiguous
- **AND** this is verified by `apps/virtaxctl/tests/e2e.rs::events_watch_survives_daemon_restart_without_loss_or_duplicates`
