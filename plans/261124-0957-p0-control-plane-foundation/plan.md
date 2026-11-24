# P0 — Control-plane foundation

Status: done (2026-11-24) — all 5 acceptance criteria verified by `cargo xtask ci` (27 tests) · Source: `~/Documents/Projects/Virtax_Plan.zip` (OxBox Rust Rebuild v1.0 draft, 2026-10-30)

## Outcome

A Cargo workspace where `oxboxd` serves the `oxbox.v1` gRPC API over a per-user Unix socket, keeps VMs in a SQLite
registry plus a portable `machine.toml` per VM, drives engines only through the `Engine` trait, and `oxctl` can
create, list, show, update, start, stop and delete VMs end to end against the deterministic `MockEngine`.

## Why this slice first

The module catalogue gives the dependency order: CP-02 (store/model), EN-01 (engine seam) and PL-01 (workspace)
have no dependencies; CP-01 (API) and CP-15 (CLI) depend only on them. Everything UI- and engine-specific
(Slint bake-off, VBoxCAPI FFI, shmfb) needs VirtualBox 7.2 or the D1 decision, neither of which exists on this host.

## Scope (FRs, partial where noted)

| Module | FRs covered here | Deferred |
|---|---|---|
| PL-01 | 001 workspace + `forbid(unsafe_code)`, 002 `cargo xtask ci`, 003 pinned toolchain + vendored protoc, 004 CI workflow (Linux job) | multi-OS matrix, perf gates, release |
| CP-02 | 001 WAL registry, 002 migrations, 003 `summary_json`, 004 `machine.toml` + JSON Schema + `validate`, 005 atomic write w/ `.prev`, 006 folder lock, 007 etag, 015 reconcile (restore from `.prev`) | `.vbox` shadowing, pending_changes, ui_prefs API |
| EN-01 | 001 `Engine` trait, PL01-011 `MockEngine` | vbox-sys, XIDL codegen, event bridge |
| CP-01 | 001 UDS 0600 in 0700 dir, 002 peer-UID check, 004 read model DTOs, 005 field_mask + etag, 006 Operations (get/list/wait), 009 OXE error codes | Windows pipe, Events.Watch, capability tokens |
| CP-15 | 002 `--json`, 003 exit codes, 004 wait/`--no-wait` | completions, vboxmanage shim, bulk selectors |

## Acceptance criteria

1. `cargo xtask ci` (fmt check, clippy `-D warnings`, tests) passes on Linux.
2. An e2e test starts `oxboxd` on a temp socket and drives `oxctl` through create → list → set (etag) → start → stop → delete.
3. A stale etag on update returns `ABORTED` and `oxctl` exits 4.
4. A truncated `machine.toml` is restored from `machine.toml.prev` at daemon startup; the VM is not dropped.
5. A second daemon on the same data dir is refused (single instance).

## Non-goals

No UI, no real hypervisor, no VirtualBox FFI, no network/USB/media.

## Result notes

- `[engine.vbox_legacy]` from the architecture doc conflicts with `engine = "<kind>"` in TOML; overrides live under `[engine_overrides.<engine>]`.
- Engine kind `mock` was added to the registry `CHECK` constraint for the MockEngine.
- Changes that need power-off are refused with `OXE-RT-002` while a VM runs; the `pending_changes` queue (FR-CP02-009) is the next step.
- The peer-UID rejection is implemented, but no test covers it, because that needs a second OS user.
- Next slices: `Events.Watch` + event journal (CP-01-007), `pending_changes`, then the `vbox-sys` / `vbox-adapter` spike once VirtualBox 7.2 is installed on a dev host.
