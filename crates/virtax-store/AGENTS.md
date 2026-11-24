# virtax-store

Persistence for `virtaxd`: the per-user SQLite registry (WAL) and each VM's `machine.toml` folder.

## Commands

```sh
cargo test -p virtax-store
```

## Layout

- `src/registry.rs`: `Registry` — migrations, machines, pending changes, operations, the event journal.
- `src/manifest_io.rs`: crash-safe `write_manifest` and `read_manifest` (restores from `.prev`).
- `src/folder_lock.rs`: exclusive advisory lock on `<vm folder>/.virtax.lock`, also used for the daemon's
  single-instance lock.
- `src/lib.rs`: `default_data_dir` (`$VIRTAX_DATA_DIR` or the platform data directory).

## Conventions

- Schema changes only append to `MIGRATIONS`; a released migration is never edited. A newer registry is refused.
- Every multi-row change is one transaction (see `update_machine_with_pending`).
- Writes to a VM folder happen only while its `FolderLock` is held.
- Timestamps are RFC 3339 UTC with milliseconds.

## Gotchas

- `event_bounds` uses `sqlite_sequence`, so `seq` keeps counting after the journal is trimmed or emptied.
- `MachineRow.pending_count` is a subquery in `MACHINE_COLS`; keep it there when adding columns.
- The registry is not `Sync`; the daemon wraps it in a `std::sync::Mutex` and never holds it across `.await`.

Rules: `../../.principles/control-plane.md`. Spec: `openspec/specs/control-plane/registry-and-configuration-store/`.
