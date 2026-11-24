# Control-plane API & IPC Specification

## Purpose
gRPC over UDS/named pipe, peer auth, Operations (LRO), by-value Events.Watch with cursor, field-mask + etag. Module `CP-01` (Control-plane API & IPC; Vietnamese: "API & IPC control plane") from BRD-09, phase P1, priority Must. Owner crates: virtaxd, virtax-api. Depends on: `control-plane/registry-and-configuration-store`. VirtualBox reference: `src/VBox/Main/idl/VirtualBox.xidl, src/libs/xpcom18a4/ipc, Main/glue, Main/webservice`. Implementation status (2026-11-24): partial — `virtax.v1` gRPC over a 0600 Unix socket with a peer-UID check, aggregated DTOs, etag updates, Operations, Events.Watch, structured VTX errors; no Windows named pipe, capability tokens, REST/mTLS gateway or client SDKs yet.

## Requirements

### Requirement: gRPC transport over UDS / named pipe (FR-CP01-001)
Virtax SHALL provide "gRPC transport over UDS / named pipe" as specified by FR-CP01-001 (BRD-09, priority Must, phase P1; Vietnamese: "Transport gRPC qua UDS / named pipe").

#### Scenario: Specified behaviour of FR-CP01-001
- **WHEN** the behaviour FR-CP01-001 covers is exercised
- **THEN** it behaves as BRD-09 specifies: tonic + prost over the UDS `$XDG_RUNTIME_DIR/virtax/virtaxd.sock` (Linux) or `~/Library/Application Support/Virtax/virtaxd.sock` (macOS), mode 0600 in a 0700 directory; Windows uses the named pipe `\\.\pipe\virtax-<SID>` with a DACL for the user's SID only and `PIPE_REJECT_REMOTE_CLIENTS`. No TCP port is opened by default.
- **AND** the VirtualBox reference for the current behaviour is `src/libs/xpcom18a4/ipc/ipcd/shared/src/ipcConfig.cpp:50-51`

#### Scenario: Acceptance for FR-CP01-001
- **WHEN** the FR-CP01-001 acceptance check is run
- **THEN** Warm connect ≤ 5 ms; `ss -lnt`/`netstat` shows 0 open TCP ports; remote clients to the pipe are rejected.

### Requirement: Peer authentication and capability tokens (FR-CP01-002)
Virtax SHALL provide "Peer authentication and capability tokens" as specified by FR-CP01-002 (BRD-09, priority Must, phase P1; Vietnamese: "Xác thực peer và capability token").

#### Scenario: Specified behaviour of FR-CP01-002
- **WHEN** the behaviour FR-CP01-002 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Check `SO_PEERCRED` (Linux), `LOCAL_PEERCRED`/`getpeereid` (macOS) and `GetNamedPipeClientProcessId` + token SID (Windows), rejecting other UIDs/SIDs; `OpenDisplay` returns a short-lived token (60 s to attach) bound to machine + screen + PID; every rejection is audit-logged.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/xpcom/server.cpp:812-887`

#### Scenario: Acceptance for FR-CP01-002
- **WHEN** the FR-CP01-002 acceptance check is run
- **THEN** A second user connecting gets `PERMISSION_DENIED` on all 3 OSes; an expired or wrong-PID token cannot attach a console.

### Requirement: AIP-style resource model (FR-CP01-003)
Virtax SHALL provide "AIP-style resource model" as specified by FR-CP01-003 (BRD-09, priority Must, phase P1; Vietnamese: "Mô hình tài nguyên kiểu AIP").

#### Scenario: Specified behaviour of FR-CP01-003
- **WHEN** the behaviour FR-CP01-003 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Resource names follow the table in the additional content (`machines/{uuid}`, `…/snapshots/{id}`, `media/{uuid}`, `operations/{id}`…); standard Get/List/Create/Update/Delete methods + custom verbs (`:start`, `:restore`); pagination `page_size ≤ 500` + `page_token`, `filter` (an AIP-160 subset: state, group, os, tag), `order_by`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/idl/VirtualBox.xidl (260 interfaces)`

#### Scenario: Acceptance for FR-CP01-003
- **WHEN** the FR-CP01-003 acceptance check is run
- **THEN** `buf lint` with AIP rules reports 0 errors; filter `state=RUNNING AND group="/Dev lab"` is correct on 500 VMs within 10 ms.

### Requirement: Aggregated DTOs from the read model (FR-CP01-004)
Virtax SHALL provide "Aggregated DTOs from the read model" as specified by FR-CP01-004 (BRD-09, priority Must, phase P1; Vietnamese: "DTO tổng hợp từ read model").

#### Scenario: Specified behaviour of FR-CP01-004
- **WHEN** the behaviour FR-CP01-004 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `ListMachines` returns complete summaries in one message from the event-warmed SQLite read model; `GetMachine` returns the full configuration DTO; the read path never calls VBoxCAPI synchronously (stale-while-revalidate at startup).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxManage/VBoxManageList.cpp:2377-2420`

#### Scenario: Acceptance for FR-CP01-004
- **WHEN** the FR-CP01-004 acceptance check is run
- **THEN** 50 VMs: `ListMachines` p95 ≤ 10 ms; counters show 0 FFI calls on the read path; the list shows even while VBoxSVC is starting.

### Requirement: UpdateMachine with field_mask + etag (FR-CP01-005)
Virtax SHALL provide "UpdateMachine with field_mask + etag" as specified by FR-CP01-005 (BRD-09, priority Must, phase P1; Vietnamese: "UpdateMachine với field_mask + etag").

#### Scenario: Specified behaviour of FR-CP01-005
- **WHEN** the behaviour FR-CP01-005 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Updates use `field_mask` and check `etag` (= `config_rev`); a mismatch returns `ABORTED` with the current etag; per-field `ApplyMode` (Live / NextBoot / NeedsPowerOff) is returned; Phase 1 maps to LockMachine(Write) → set → saveSettings → unlock on the VM actor; `GUI/*` writes are rejected; no veto events.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/MachineImpl.cpp:2649-3043,4574-4594`

#### Scenario: Acceptance for FR-CP01-005
- **WHEN** the FR-CP01-005 acceptance check is run
- **THEN** Two clients editing the same VM: the second gets `ABORTED` + the new etag, nothing is silently overwritten; a 5-field update p95 ≤ 50 ms.

### Requirement: Operations service (FR-CP01-006)
Virtax SHALL provide "Operations service" as specified by FR-CP01-006 (BRD-09, priority Must, phase P1; Vietnamese: "Dịch vụ Operations").

#### Scenario: Specified behaviour of FR-CP01-006
- **WHEN** the behaviour FR-CP01-006 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Every long action returns `Operation{name, kind, target, state, percent, eta, step, cancellable, error}` with the Queued → Running → Cancelling/Succeeded/Failed/Cancelled lifecycle (diagram 31); `Get/List/Watch/Cancel/Retry`; persisted in the `operations` table with 30-day history; on daemon restart unfinished Operations become "Interrupted".
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-all/ProgressImpl.cpp:772-823`

#### Scenario: Acceptance for FR-CP01-006
- **WHEN** the FR-CP01-006 acceptance check is run
- **THEN** `Watch` sends coalesced updates at ≤ 4 Hz; `Cancel` is acknowledged within 100 ms; no client needs to poll (verified in server logs).

### Requirement: By-value Events.Watch with since_seq and coalescing (FR-CP01-007)
Virtax SHALL provide "By-value Events.Watch with since_seq and coalescing" as specified by FR-CP01-007 (BRD-09, priority Must, phase P1; Vietnamese: "Events.Watch by value, since_seq và coalescing").

#### Scenario: Specified behaviour of FR-CP01-007
- **WHEN** the behaviour FR-CP01-007 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Stream `Event{seq, ts, resource, kind}` filtered by resource prefix/kind; resume via `since_seq` from `event_journal` (a 100,000-event or 24 h ring), a too-old cursor gets `RESYNC_REQUIRED`; each subscriber has a 1,024-entry bounded queue, state events coalesce latest-wins per resource, transition/terminal events are never dropped; slow subscribers get coalesced, never unregistered.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-all/EventImpl.cpp:876-892`

#### Scenario: Acceptance for FR-CP01-007
- **WHEN** the FR-CP01-007 acceptance check is run
- **THEN** A 10,000-event burst with a client reading 10 msg/s loses 0 terminal events with ≤ 8 MB per subscriber; engine → client latency p95 ≤ 50 ms.

### Requirement: Legacy engine event bridge (FR-CP01-008)
Virtax SHALL provide "Legacy engine event bridge" as specified by FR-CP01-008 (BRD-09, priority Must, phase P1; Vietnamese: "Cầu nối event từ engine legacy").

#### Scenario: Specified behaviour of FR-CP01-008
- **WHEN** the behaviour FR-CP01-008 covers is exercised
- **THEN** it behaves as BRD-09 specifies: One passive listener per VBox event source, drained continuously on a dedicated FFI thread, translated into by-value events that also update the read model; if VBox unregisters the listener it re-registers and resyncs; no active listeners (they would need Rust-implemented COM objects).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/cbinding/tstCAPIGlue.c:871,1089-1109`

#### Scenario: Acceptance for FR-CP01-008
- **WHEN** the FR-CP01-008 acceptance check is run
- **THEN** Spike S4: a 10,000-event snapshot/medium burst never gets the listener unregistered; the read model matches VBoxSVC 100% afterwards.

### Requirement: Deadlines, cancellation and structured errors (FR-CP01-009)
Virtax SHALL provide "Deadlines, cancellation and structured errors" as specified by FR-CP01-009 (BRD-09, priority Must, phase P1; Vietnamese: "Deadline, huỷ và lỗi có cấu trúc").

#### Scenario: Specified behaviour of FR-CP01-009
- **WHEN** the behaviour FR-CP01-009 covers is exercised
- **THEN** it behaves as BRD-09 specifies: Every RPC has a deadline (5 s reads, 30 s writes by default; LROs return at once); errors use gRPC status + `ErrorInfo` (VTX code), vi/en `LocalizedMessage`, quick-fix actions and the original `VERR_*`/HRESULT; client cancellation stops server-side work.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIErrorString.cpp:107-228`

#### Scenario: Acceptance for FR-CP01-009
- **WHEN** the FR-CP01-009 acceptance check is run
- **THEN** The catalogue lint shows 100% of VTX codes with vi + en messages; no RPC outlives its deadline in a hung-VBoxSVC chaos test.

### Requirement: Versioning policy vbx.v1 → virtax.v1 (FR-CP01-010)
Virtax SHALL provide "Versioning policy vbx.v1 → virtax.v1" as specified by FR-CP01-010 (BRD-09, priority Must, phase P1; Vietnamese: "Chính sách version vbx.v1 → virtax.v1").

#### Scenario: Specified behaviour of FR-CP01-010
- **WHEN** the behaviour FR-CP01-010 covers is exercised
- **THEN** it behaves as BRD-09 specifies: `vbx.v1` was only the analysis's draft name; the first published contract is package `virtax.v1` (renamed before the public alpha). Within v1 only additive changes, field numbers are never reused (`reserved`), deprecated fields stay for ≥ 2 minor releases; breaking changes go to `virtax.v2` served side by side; `Host.GetVersion` reports API versions and capabilities.
- **AND** the VirtualBox reference for the current behaviour is `analysis 02 §6.4 (package vbx.v1)`

#### Scenario: Acceptance for FR-CP01-010
- **WHEN** the FR-CP01-010 acceptance check is run
- **THEN** `buf breaking` in CI reports 0 violations against the previous release; an old v1 client works with the new daemon in the compatibility test.

### Requirement: Daemon lifecycle, single instance, idle efficiency (FR-CP01-011)
Virtax SHALL provide "Daemon lifecycle, single instance, idle efficiency" as specified by FR-CP01-011 (BRD-09, priority Must, phase P1; Vietnamese: "Vòng đời daemon, một instance, hiệu quả khi rảnh").

#### Scenario: Specified behaviour of FR-CP01-011
- **WHEN** the behaviour FR-CP01-011 covers is exercised
- **THEN** it behaves as BRD-09 specifies: One `virtaxd` per user (lock file + socket probe), started on demand (systemd socket activation, launchd, or a Manager/`virtaxctl` spawn on Windows); it stays while VMs run or clients connect, keeps VBoxSVC resident, and has no periodic timers without subscribers — it wakes only on events (VBoxSVC, pidfd/kqueue, sockets).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/xpcom/server.cpp:121-124; ObjectsTracker.cpp:1019`

#### Scenario: Acceptance for FR-CP01-011
- **WHEN** the FR-CP01-011 acceptance check is run
- **THEN** 10 clients starting at once yield exactly 1 daemon; idle (no VMs, no subscribers) ≤ 5 wakeups/s measured with powertop/ETW/powermetrics.

### Requirement: Client SDKs and API documentation (FR-CP01-012)
Virtax SHALL provide "Client SDKs and API documentation" as specified by FR-CP01-012 (BRD-09, priority Should, phase P1; Vietnamese: "Client SDK và tài liệu API").

#### Scenario: Specified behaviour of FR-CP01-012
- **WHEN** the behaviour FR-CP01-012 covers is exercised
- **THEN** it behaves as BRD-09 specifies: A Rust `virtax-api` client crate generated from the protos, a Python (grpcio) package replacing `vboxapi.py`, and a reference generated from proto comments with examples per service.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/glue/python/vboxapi.py:208-1064`

#### Scenario: Acceptance for FR-CP01-012
- **WHEN** the FR-CP01-012 acceptance check is run
- **THEN** A "list VMs + start + watch events" example runs in Rust and Python in CI; 100% of RPCs carry doc comments.

### Requirement: Optional REST/JSON gateway and remote mTLS (FR-CP01-013)
Virtax SHALL provide "Optional REST/JSON gateway and remote mTLS" as specified by FR-CP01-013 (BRD-09, priority Could, phase P2; Vietnamese: "Gateway REST/JSON và remote mTLS tuỳ chọn").

#### Scenario: Specified behaviour of FR-CP01-013
- **WHEN** the behaviour FR-CP01-013 covers is exercised
- **THEN** it behaves as BRD-09 specifies: For headless labs: a REST/JSON gateway and a TCP + mTLS (rustls) listener, off by default and enabled only by explicit configuration, warning when bound beyond loopback; same resource model and permissions.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/webservice/vboxweb.cpp:135,911`

#### Scenario: Acceptance for FR-CP01-013
- **WHEN** the FR-CP01-013 acceptance check is run
- **THEN** Connections without a valid client certificate are refused; the default configuration opens no port (checked in CI).

### Requirement: As built — Lossless event delivery
Instead of coalescing state events (FR-CP01-007), `virtaxd` SHALL journal every event with a gap-free `seq` and SHALL let a watcher that falls behind the 1,024-event broadcast buffer catch up from the journal, so no event is lost. A cursor older than the retained journal (100,000 events or 24 h) or ahead of it SHALL fail with `OUT_OF_RANGE` and `VTX-API-410`. Source: `apps/virtaxd/src/events.rs`.

#### Scenario: Burst beyond the buffer
- **WHEN** 3,072 events are emitted while a watcher reads slowly
- **THEN** the watcher receives all of them in order with no gaps or duplicates
- **AND** this is verified by `apps/virtaxd/src/events.rs::burst_beyond_buffer_loses_nothing`

#### Scenario: Expired cursor
- **WHEN** a watcher asks for events after a trimmed or future `seq`
- **THEN** the request fails with `VTX-API-410`
- **AND** this is verified by `apps/virtaxd/src/events.rs::stale_or_future_cursor_requires_resync`

### Requirement: As built — Shutdown is not blocked by watchers
On SIGTERM `virtaxd` SHALL end every open event and operation stream and SHALL exit within 5 s even if clients stay connected. Source: `apps/virtaxd/src/main.rs`.

#### Scenario: Restart under an open watcher
- **WHEN** `virtaxd` is restarted while `virtaxctl events watch` is connected
- **THEN** the daemon exits, and the watcher reconnects and resumes without losing or repeating events
- **AND** this is verified by `apps/virtaxctl/tests/e2e.rs::events_watch_survives_daemon_restart_without_loss_or_duplicates`
