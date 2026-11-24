# Virtax

**Virtax** is a Rust rebuild of desktop virtualization (GPL-3.0-or-later), following the
"Strangler + OpenVMM" plan: a new Rust control plane, CLI and UI first on the VirtualBox 7.2 engine,
then a Rust VMM (`virtax-vmm`). The design set (C4 architecture, BRDs, module index) lives outside this repo; it uses
the earlier provisional codename "OxBox", which Virtax replaces (decision D5).

## Status

**P0 control-plane foundation.** What works today:

- `virtaxd`, a per-user daemon serving the `virtax.v1` gRPC API over a Unix socket (mode 0600 in a 0700
  directory). It accepts only connections from its own UID, and only one daemon runs per data directory.
- A SQLite (WAL) registry with versioned migrations, plus one portable `machine.toml` per VM. Manifest writes
  are atomic and keep a `.prev` copy; each VM folder is locked; edits use etag optimistic concurrency; the
  daemon reconciles the registry with the manifests at startup and restores damaged manifests from `.prev`.
- The `Engine` trait (the strangler seam) and a deterministic `MockEngine`.
- Event streaming: every change is journaled with a gap-free `seq` and served by `Events.Watch`, which
  replays from a cursor and then follows live events. `virtaxctl events watch` resumes after daemon restarts.
  See [docs/events.md](docs/events.md).
- Pending changes: on a running VM, changes that take effect at the next boot are queued, and changes that
  need power-off are queued on request (`virtaxctl vm set --at-shutdown`). The queue is applied when the VM
  powers off, at daemon startup, and before every start. `virtaxctl vm pending list|cancel` manages it.
- `virtaxctl`, which can create, list, show, set, start, stop and delete VMs, plus `ops`, `events` and `version` commands.
  It supports `--json`, documented exit codes, and local `vm validate` / `vm schema`.

- `virtax-vmm`, the Rust VMM (one process per VM, KVM on Linux x86_64, built on rust-vmm crates): boots a Linux
  kernel + initramfs directly to a serial console in `<vm folder>/logs/serial.log`. `virtaxd` runs it through
  `VirtaxVmmEngine` (the default engine when `/dev/kvm` is usable). See [docs/virtax-vmm.md](docs/virtax-vmm.md).

Not yet: the VirtualBox legacy engine (VBoxCAPI FFI), the console/framebuffer data plane, the Manager UI, snapshots, media, networking, and Windows named pipes. **With only `MockEngine`, no guest runs.**

## Layout

| Path | Purpose |
|---|---|
| `crates/virtax-model` | Domain types, `machine.toml` (`virtax.machine/1`) + JSON Schema, VM lifecycle, VTX error catalogue |
| `crates/virtax-store` | Registry (SQLite), atomic manifest I/O, folder locks |
| `crates/virtax-engine` | `Engine` trait, `MockEngine` |
| `crates/virtax-vmm-hv` | Hypervisor backends (KVM); the only crate allowed `unsafe`, each block justified |
| `crates/virtax-vmm-core` | Guest memory layout, x86_64 direct boot, serial/i8042 devices, vCPU run loop |
| `apps/virtax-vmm` | The VMM process, one per VM |
| `crates/virtax-api` | `proto/virtax/v1/virtax.proto`, generated stubs, UDS client connector |
| `apps/virtaxd` | Daemon |
| `apps/virtaxctl` | CLI (`tests/e2e.rs` drives the real binaries) |
| `xtask` | `cargo xtask ci`, `cargo xtask schema` |
| `schemas/` | Generated JSON Schema for `machine.toml` |

## Build and try it

Only the pinned Rust toolchain is required (`protoc` is vendored).

```sh
cargo xtask ci                       # fmt check, lint policy, clippy -D warnings, all tests

cargo run -p virtaxd -- --mock-engine &   # simulated engine; data in ~/.local/share/virtax
cargo run -p virtaxctl -- vm create --name web --os-type Ubuntu_64 --cpus 2 --memory 2048
cargo run -p virtaxctl -- vm start web
cargo run -p virtaxctl -- --json vm list
cargo run -p virtaxctl -- vm set web --memory 4096 --dry-run   # shows when each change applies
```

Boot a real guest on KVM (Linux x86_64, user in the `kvm` group, `busybox-static` installed):

```sh
cargo xtask guest-kernel                 # pinned Ubuntu kernel + busybox initramfs in .cache/guest
cargo build -p virtax-vmm && cargo run -p virtaxd &
cargo run -p virtaxctl -- vm create --name g --cpus 1 --memory 512 \
  --kernel .cache/guest/vmlinuz --initrd .cache/guest/initramfs.cpio
cargo run -p virtaxctl -- vm start g         # the test guest prints a marker and reboots, ending the VM
cargo xtask kvm-test                     # the KVM boot tests
```

`VIRTAX_SOCKET` and `VIRTAX_DATA_DIR` override the socket and data locations. Unix socket paths must stay
under 104 bytes.

## End-to-end tests

Run from the workspace root on Linux or macOS with the pinned Rust toolchain.

```sh
cargo xtask e2e                                # run the E2E suite
cargo xtask e2e machine_lifecycle_through_cli  # filter by test name
```

The suite drives the real `virtaxctl` and `virtaxd` binaries through a private Unix socket. Each test daemon
uses a temporary data directory and the simulated engine. VirtualBox, a guest image, and a separately
running daemon are unnecessary. Coverage includes the VM lifecycle, stale etags, manifest recovery,
single-instance enforcement, offline validation, and event replay across daemon restarts.
`cargo xtask ci` also runs the suite on the existing CI matrix.

## virtaxctl exit codes

`0` ok · `1` generic · `2` usage · `3` not found · `4` conflict/etag · `5` state/precondition ·
`6` timeout · `7` cancelled · `10` engine error. With `--json`, stdout carries exactly one JSON document,
including for errors.

## Conventions

- Every crate inherits the workspace lints, which forbid `unsafe_code`. FFI crates will be explicit exceptions.
- `virtax.v1` only takes additive changes; never reuse a proto field number.
- Manifest top-level keys starting with `x-` are preserved across round trips. Engine-specific knobs go under
  `[engine_overrides.<engine>]`.
