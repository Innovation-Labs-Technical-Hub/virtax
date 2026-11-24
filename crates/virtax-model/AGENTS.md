# virtax-model

Domain types shared by every other crate: the `machine.toml` manifest (`virtax.machine/1`) and its JSON Schema,
the collapsed VM lifecycle, the `MachineSummary` read-model DTO and the `VTX-*` error catalogue.
No I/O and no async: pure types, parsing and validation.

## Commands

```sh
cargo test -p virtax-model
cargo xtask schema        # regenerate schemas/virtax.machine.v1.json after changing manifest types
```

## Layout

- `src/manifest.rs`: `MachineManifest` and its tables, `parse_manifest` (line/column errors), `validate`,
  `to_toml`, `json_schema`.
- `src/state.rs`: `MachineState`, `Action`, which actions each state allows.
- `src/summary.rs`: `MachineSummary`, cached in the registry as `summary_json`.
- `src/errors.rs`: the `VtxCode` catalogue (code, English, Vietnamese, hint) and `VtxError`.

## Conventions

- Manifest compatibility: unknown top-level keys starting with `x-` round-trip; every other unknown key is an
  error. Nested tables use `deny_unknown_fields`.
- New user-facing errors get a `VtxCode` here with both languages and a hint; never reuse a retired code.
- A manifest type change needs `cargo xtask schema` and a commit of the regenerated schema.

## Gotchas

- Engine-specific keys are `[engine_overrides.<engine>]`, not `[engine.*]`: TOML cannot hold both
  `engine = "<kind>"` and an `[engine]` table.
- `toml` uses `preserve_order`, so serialized manifests keep field order; tests compare round trips.

Rules: `../../.principles/rust.md`, `../../.principles/control-plane.md`. Spec:
`openspec/specs/control-plane/registry-and-configuration-store/`.
