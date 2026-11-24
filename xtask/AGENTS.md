# xtask

Repository automation, run as `cargo xtask <command>`. Every CI job calls exactly one of these commands.

## Commands

```sh
cargo xtask ci [--quick]   # fmt check, lint policy, clippy -D warnings (skipped with --quick), all tests
cargo xtask e2e [filter]   # virtaxctl end-to-end tests
cargo xtask schema         # regenerate schemas/virtax.machine.v1.json
cargo xtask guest-kernel   # pinned Ubuntu kernel (SHA-256 checked) + busybox initramfs into .cache/guest
cargo xtask kvm-test       # every kvm_* test; needs /dev/kvm and guest-kernel
cargo xtask deny           # cargo deny check (deny.toml), then cargo vet --locked (supply-chain/)
cargo xtask audit          # cargo audit of Cargo.lock
cargo xtask spec           # OpenSpec 2.1.1 strict validation (npx; Node.js >= 22.20)
cargo xtask proto [ref]    # buf lint; with a git ref also buf breaking against it (npx)
cargo xtask multiuser-test # a second OS user is refused by socket permissions and the peer check
```

## Layout

- `src/main.rs`: commands, the lint policy (`UNSAFE_ALLOWED`) and the CI gate.
- `src/guest.rs`: test-guest download and verification, the cpio writer, the guest's `/init` script.
- `src/multiuser.rs`: the second-OS-user test (creates and removes the `virtax-other` system user).

## Conventions

- Automation lives here in Rust, not in shell scripts; `justfile` recipes only call it.
- Every download has a pinned version and SHA-256 and is cached in `.cache/` (git-ignored).
- Adding a crate that needs `unsafe` means adding it to `UNSAFE_ALLOWED` in review, never silently.

## Gotchas

- `guest-kernel` needs `curl`, `sha256sum`, `dpkg-deb`, `tar` and a static `/usr/bin/busybox` (`busybox-static`).
- `deny` and `audit` need `cargo-deny` 0.22, `cargo-vet` 0.10 and `cargo-audit` 0.23 installed; CI pins them.
- `multiuser-test` needs passwordless `sudo` and refuses to run if `virtax-other` already exists.
- Changing the test guest's `/init` requires re-running `cargo xtask guest-kernel` before the KVM tests.

Rules: `../.principles/build-and-ci.md`.
