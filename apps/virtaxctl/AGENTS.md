# virtaxctl

The command-line client for `virtaxd`: `vm`, `ops`, `events` and `version` commands, human output by default
and one JSON document with `--json`.

## Commands

```sh
cargo run -p virtaxctl -- --help
cargo test -p virtaxctl --test e2e          # end-to-end tests against real virtaxd binaries
cargo xtask e2e <filter>                    # the same, filtered
just ctl vm list                            # against the dev daemon from `just dev-up`
```

## Layout

- `src/main.rs`: commands, exit codes, error rendering, `events watch` with reconnect.
- `src/output.rs`: JSON field names (part of the scripting contract) and tables.
- `tests/e2e.rs`: starts `virtaxd` on a private socket and drives this binary; `kvm_*` tests are ignored unless
  run with `cargo xtask kvm-test`.

## Conventions

- Exit codes: 0 ok, 1 generic, 2 usage, 3 not found, 4 conflict/etag, 5 state, 6 timeout, 7 cancelled,
  10 engine. Changing one breaks scripts.
- JSON field names are a stable contract: add fields, never rename or remove them.
- Make relative paths absolute before sending them to the daemon (`--kernel`, `--initrd`).

## Gotchas

- The e2e helper builds `virtaxd` and `virtax-vmm` once per test run and uses socket paths under `/tmp`.
- `just` recipe arguments are positional: `just boot 4 512`, not `just boot cpus=4`.

Rules: `../../.principles/control-plane.md`.
