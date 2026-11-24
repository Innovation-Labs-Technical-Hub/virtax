# Control-plane principles

Scope: `apps/virtaxd`, `apps/virtaxctl`, `crates/virtax-api`, `crates/virtax-model`, `crates/virtax-store`,
`crates/virtax-engine`.

## Principles

1. **The contract comes first.** `virtax.v1` (`crates/virtax-api/proto/virtax/v1/virtax.proto`) and the
   `virtax.machine/1` manifest are what the Manager, Console, scripts and other teams build against; they change
   before the code, and only additively within a version (ADR-003, FR-CP01-010).
2. **The daemon owns state; clients are thin.** All mutations go through `virtaxd`, serialized per VM; clients
   read the warm read model and never touch the registry or manifests directly (ADR-003, FR-CP01-004).
3. **Never lose or corrupt a VM.** A crash at any point leaves a valid `machine.toml` and a registry that
   reconcile can repair; nothing is deleted because it could not be read (ADR-004, FR-CP02-005, FR-CP02-015).
4. **Long work is an Operation, change is an event.** Clients follow progress and state through
   `Operations.WatchOperation` and `Events.Watch`; nobody polls (FR-GEN-006, FR-CP01-006/007).
5. **Engines are interchangeable.** `virtaxd` reaches hypervisors only through the `Engine` trait, so the
   VirtualBox adapter, `virtax-vmm` and the MockEngine are swapped per VM (ADR-001, FR-EN01-001).

## Rules

1. MUST change `virtax.proto` only additively within `virtax.v1`: new messages, fields, RPCs and enum values; a
   removed field's number is `reserved`, never reused. A breaking change needs a new package (`virtax.v2`).
2. MUST NOT edit generated code: prost/tonic output is generated at build time; `schemas/virtax.machine.v1.json`
   is regenerated with `cargo xtask schema` after a manifest type changes.
3. MUST keep manifest compatibility: unknown top-level keys starting with `x-` round-trip unchanged; other unknown
   keys are errors with line and column; engine-specific keys live under `[engine_overrides.<engine>]`.
4. MUST write manifests only through `virtax_store::write_manifest` (temp file, fsync, `.prev`, rename, directory
   fsync) while holding the VM's folder lock.
5. MUST change the registry schema only by appending a migration to `MIGRATIONS` in
   `crates/virtax-store/src/registry.rs`; never edit a released migration. A daemon MUST refuse a registry newer
   than it knows (`VTX-CFG-022`). Back up the registry before migrating a user's data.
6. MUST check the etag on every `UpdateMachine` and return `ABORTED` with the current etag on a mismatch.
7. MUST run every mutation of a VM under its per-VM lock (`Daemon::vm_lock`), re-reading the registry row after
   acquiring it.
8. MUST journal every state-changing event through `Daemon::emit`, never while holding the registry lock, so
   `Events.Watch` sees each change exactly once, in `seq` order.
9. MUST return gRPC errors with an appropriate status code and the `vtx-code` metadata key (and `vtx-etag` for
   conflicts); `virtaxctl` maps them to its documented exit codes.
10. MUST keep `virtaxctl --json` output to one JSON document per invocation (NDJSON for `events watch`), with
    warnings and progress on stderr.
11. MUST NOT import a concrete engine (`virtax-vmm-*`, a future VirtualBox adapter) from `virtaxd` except where
    engines are constructed in `apps/virtaxd/src/main.rs`.
12. SHOULD cover each new RPC behaviour with a daemon unit test, and each new CLI behaviour with a test in
    `apps/virtaxctl/tests/e2e.rs` that drives the real binaries.

## Enforcement

| Rule | Check |
|---|---|
| 3, 4 | `crates/virtax-model` manifest tests, `crates/virtax-store` `manifest_io` tests (crash between steps, truncation, `.prev`) |
| 5 | `migrates_once_and_reopens`, `refuses_newer_registry` |
| 6, 9, 10 | `apps/virtaxctl/tests/e2e.rs` (`machine_lifecycle_through_cli` checks exit codes, the returned etag and JSON-only stdout) |
| 8 | `burst_beyond_buffer_loses_nothing`, `events_watch_survives_daemon_restart_without_loss_or_duplicates` |
| 1 | `cargo xtask proto origin/<base>` (`buf lint` and `buf breaking`) in CI job `proto (buf lint, breaking)` on every pull request |
| 2, 7, 11, 12 | Code review |

## Anti-patterns

- **Graceful shutdown waiting on streams.** tonic's graceful shutdown waits for open streams, and an event watch
  never ends, so `virtaxd` hung on SIGTERM. Fix: a shutdown token ends every stream, plus a 5 s hard deadline.
- **Unix socket paths too long.** Paths over about 104 bytes cannot be bound; keep sockets in
  `$XDG_RUNTIME_DIR` (or `/tmp` in tests), not in deep data directories. `virtaxd` refuses such paths up front.
- **Reading state outside the per-VM lock, then writing it.** A check followed by a write races with another
  request; re-read under the lock (rule 7).

## References

- Architecture: ADR-001 to ADR-004, §"C4 Level 3 — Component", §"Data model".
- `openspec/specs/control-plane/` (CP-01, CP-02, CP-03, CP-15) and `docs/events.md`.
