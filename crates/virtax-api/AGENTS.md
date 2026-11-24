# virtax-api

The `virtax.v1` control-plane contract: `proto/virtax/v1/virtax.proto`, the tonic/prost code generated from it at
build time, the default socket path and a Unix-socket client connector.

## Commands

```sh
cargo build -p virtax-api   # regenerates the Rust code from the proto (vendored protoc, no install needed)
```

## Layout

- `proto/virtax/v1/virtax.proto`: services `Machines`, `Runtime`, `Operations`, `Events`, `Daemon`.
- `build.rs`: runs `tonic-prost-build` with the vendored `protoc`.
- `src/lib.rs`: `v1` module, `default_socket_path`, `connect`, metadata keys `vtx-code` and `vtx-etag`.

## Conventions

- Additive changes only within `virtax.v1`; a removed field's number becomes `reserved`.
- Document every message, field and RPC in the proto; `docs/events.md` documents event payloads.
- Update the proto first, then `virtaxd` and `virtaxctl` in the same change.

## Gotchas

- Generated code is in `OUT_DIR`, never in the repository; do not try to edit it.
- Unix socket paths must stay under about 104 bytes.

Rules: `../../.principles/control-plane.md`. Spec: `openspec/specs/control-plane/control-plane-api-and-ipc/`.
