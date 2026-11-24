# virtaxd

The per-user control-plane daemon: serves `virtax.v1` over a 0600 Unix socket, owns the registry and manifests,
serializes changes per VM, runs Operations and journals events.

## Commands

```sh
cargo test -p virtaxd                       # daemon unit tests (events, pending changes, reconcile)
cargo run -p virtaxd -- --help              # flags: --socket --data-dir --mock-engine --vmm-bin --log-json
just dev-up && just ctl vm list && just dev-down   # run it locally with data in .cache/dev
```

## Layout

- `src/main.rs`: flags, single-instance lock, socket setup, engines, graceful shutdown.
- `src/daemon.rs`: machines (create, get, update, delete), start/stop Operations, reconcile.
- `src/pending.rs`: queued changes applied at power-off, at startup and before start.
- `src/events.rs`: event journal, `emit`, `watch_events` with catch-up from the journal.
- `src/grpc.rs`: tonic services and the peer-UID check; `src/error.rs`: VTX code → gRPC status.

## Conventions

- Mutations run under `Daemon::vm_lock(id)` and re-read the registry row inside it.
- Never call `emit` while holding the registry lock (it takes that lock).
- New behaviour gets a unit test here and, if users see it, an e2e test in `apps/virtaxctl/tests/e2e.rs`.

## Gotchas

- `--mock-engine` adds a simulated engine; `virtax-vmm` is added automatically when `/dev/kvm` and the binary
  are available (`--vmm-bin`, default: next to `virtaxd`).
- Shutdown ends open streams and exits after 5 s even if clients remain connected.

Rules: `../../.principles/control-plane.md`, `../../.principles/security.md`. Docs: `docs/events.md`.
