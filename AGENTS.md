# Virtax — agent guide

Virtax is a Rust rebuild of desktop virtualization (GPL-3.0-or-later): a per-user control-plane daemon
(`virtaxd`), a CLI (`virtaxctl`) and a Rust VMM (`virtax-vmm`) that boots Linux guests on KVM. The design set
(BRDs, C4 architecture) lives outside the repository; its requirements are in `openspec/specs/`.

## Stack

Rust 2024 Cargo workspace (toolchain pinned in `rust-toolchain.toml`) · tokio + tonic gRPC over a Unix socket ·
protobuf `virtax.v1` · SQLite (rusqlite, WAL) + TOML manifests · rust-vmm crates on KVM (Linux x86_64) ·
`just` + `cargo xtask` · GitHub Actions (Linux x64/arm64, macOS, a KVM job) · OpenSpec 2.1.1.

## Directory map

| Path | What it is | Owner (role) |
|---|---|---|
| `crates/virtax-model` | Domain types, `machine.toml` schema, VM lifecycle, `VTX-*` error catalogue | Backend lead |
| `crates/virtax-store` | SQLite registry, atomic manifest I/O, folder locks | Backend lead |
| `crates/virtax-engine` | `Engine` trait, `MockEngine`, `VirtaxVmmEngine` | Backend lead / VMM lead |
| `crates/virtax-api` | `virtax.v1` proto and generated client/server code | Backend lead |
| `crates/virtax-vmm-core` | Guest memory, x86_64 boot, ACPI, devices, vCPU loop | VMM lead |
| `crates/virtax-vmm-hv` | KVM backend — the only crate allowed `unsafe` | VMM lead |
| `apps/virtaxd` | Control-plane daemon | Backend lead |
| `apps/virtaxctl` | CLI; `tests/e2e.rs` drives the real binaries | Backend lead |
| `apps/virtax-vmm` | One VMM process per VM | VMM lead |
| `xtask` | `cargo xtask` automation (CI gate, schema, test guest) | Platform/Release |
| `openspec/` | Specs and changes — see `openspec/AGENTS.md` | Tech lead |
| `.principles/` | Engineering rules per area | Tech lead |
| `docs/` | Evergreen docs (`events.md`, `virtax-vmm.md`) | Each area's owner |
| `plans/` | Implementation plans and reports (records, not authority) | Whoever runs the work |
| `schemas/` | Generated JSON Schema for `machine.toml` | Generated |

Each crate has its own `AGENTS.md` with its commands and gotchas.

## Golden rules

1. **Spec first.** Behaviour changes go through an OpenSpec change citing requirement IDs
   (`.principles/agentic-coding.md`). Decisions marked TO BE CONFIRMED stay open; follow the proposed default and
   say so.
2. **Contract first.** Change `crates/virtax-api/proto/virtax/v1/virtax.proto` additively only, before the code
   that uses it (`.principles/control-plane.md`).
3. **Never edit generated code.** prost/tonic output is generated at build time; regenerate
   `schemas/virtax.machine.v1.json` with `cargo xtask schema`.
4. **Module boundaries.** `virtaxd` reaches hypervisors only through the `Engine` trait; `unsafe` only in
   `crates/virtax-vmm-hv` (`.principles/rust.md`, `.principles/vmm.md`).
5. **Verify before claiming.** Run the narrowest test, then `cargo xtask ci`; add `cargo xtask kvm-test` for
   engine or VMM changes. Report failures with their output.
6. **Secrets and licences.** Never commit secrets or personal data; every dependency must be GPL-3.0-compatible
   (`.principles/security.md`).
7. **Shared workspace.** Stage explicit paths, read `git diff --cached`, and stop every process you start.

## Commands

```sh
just                 # list recipes (`make <recipe>` forwards to the same recipes)
just doctor          # what this host can run: KVM, busybox-static, test guest, dev daemon
just ci              # = cargo xtask ci: fmt check, lint policy, clippy -D warnings, all tests
just test <filter>   # cargo test --workspace --locked -- <filter>
just spec            # = cargo xtask spec: OpenSpec strict validation
just deny            # = cargo xtask deny: cargo-deny (deny.toml) then cargo-vet (supply-chain/)
just audit           # = cargo xtask audit: RustSec advisories (also runs daily in CI)
just proto [ref]     # = cargo xtask proto: buf lint, plus buf breaking against a git ref
cargo xtask multiuser-test   # second OS user is refused (Linux, passwordless sudo; CI only)
just guest           # pinned test kernel + busybox initramfs in .cache/guest
just kvm-test        # tests that boot real guests (needs /dev/kvm)
just dev-up / dev-down / ctl <args> / demo   # a dev daemon with data in .cache/dev
```

## Where to look

- Requirements: `openspec/specs/` (start with `openspec/AGENTS.md`).
- Rules: `.principles/README.md`.
- Behaviour docs: `README.md`, `docs/events.md`, `docs/virtax-vmm.md`.
- Contracts: `crates/virtax-api/proto/virtax/v1/virtax.proto`, `schemas/virtax.machine.v1.json`,
  `crates/virtax-model/src/errors.rs`.
