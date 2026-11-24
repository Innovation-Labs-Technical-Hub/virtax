# virtax-engine

The `Engine` trait — the seam between `virtaxd` and hypervisors — with two implementations: `MockEngine`
(deterministic, in memory, for development and tests) and `VirtaxVmmEngine` (one `virtax-vmm` process per VM).

## Commands

```sh
cargo test -p virtax-engine
```

## Layout

- `src/lib.rs`: the `Engine` trait, `ApplyMode`, `OpHandle` (progress and cancellation), `apply_mode_for`.
- `src/mock.rs`: `MockEngine` with scriptable latency and failures (`fail_next`).
- `src/vmm.rs`: `VirtaxVmmEngine` — builds `virtax-vmm` arguments from the manifest, waits for the ready line,
  supervises the process, and maps its exit to `powered_off` or `aborted`.

## Conventions

- `virtaxd` uses only the trait; engine-specific behaviour stays in this crate.
- State changes are broadcast as `EngineEvent::StateChanged`; the daemon keeps its read model warm from them.
- Configuration a VM cannot run with fails with `VTX-ENG-011` before any process starts.

## Gotchas

- `VirtaxVmmEngine` relies on `virtax-vmm`'s process contract: the ready line, exit codes, SIGTERM as the power
  button and re-exec on reboot. Change both sides together (`.principles/vmm.md` rule 5).
- An ACPI stop waits up to 120 s; on timeout the machine stays `running` and the error is `VTX-ENG-012`.
- Machine state lives in memory: after a daemon restart every machine starts as `powered_off`.

Rules: `../../.principles/control-plane.md`, `../../.principles/vmm.md`. Spec:
`openspec/specs/engine/engine-trait-and-legacy-adapter/`.
