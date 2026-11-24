# Rust principles

Scope: every crate in the Cargo workspace (`crates/*`, `apps/*`, `xtask`).

## Principles

1. **Safe Rust by default.** `unsafe` is confined to the crates that must talk to the hypervisor or C APIs, so a
   memory-safety review never has to read the rest (NFR-00-12, architecture §"C4 Level 4").
2. **Warnings are errors.** A warning ignored today hides the one that matters tomorrow; CI builds with
   `clippy -D warnings` on three hosts.
3. **One owner per concept.** Domain types live in `virtax-model`, persistence in `virtax-store`, the hypervisor
   seam in `virtax-engine`; binaries compose them. This keeps the strangler seam (ADR-001) real.
4. **Errors carry a VTX code.** A user never sees a raw OS error, HRESULT or `VERR_*` without a code, a bilingual
   message and a hint (FR-GEN-007, FR-CP01-009).
5. **Portable unless proven otherwise.** Code compiles on Linux x64/arm64 and macOS; platform-specific code is
   gated explicitly so the other hosts still build (NFR-00-11).

## Rules

1. MUST inherit the workspace lints (`[lints] workspace = true`), which forbid `unsafe_code`. The only exception
   is a crate listed in `UNSAFE_ALLOWED` in `xtask/src/main.rs` (today `crates/virtax-vmm-hv`), which MUST deny
   `clippy::undocumented_unsafe_blocks` and put a `SAFETY:` comment on every `unsafe` block.
2. MUST pass `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`.
   Allow a lint only locally, with `#[allow(...)]` and a comment saying why (see `apps/virtaxctl/src/main.rs`).
3. MUST use edition 2024 and the toolchain pinned in `rust-toolchain.toml`; MUST commit `Cargo.lock` and build CI
   with `--locked`.
4. MUST declare shared dependencies once in the root `[workspace.dependencies]` and reference them with
   `.workspace = true`. A dependency used by one crate only MAY be declared in that crate.
5. MUST map every user-facing failure to a `VtxCode` in `crates/virtax-model/src/errors.rs`, with English and
   Vietnamese text and a hint. New codes are added there, never inline.
6. MUST gate platform-specific items (functions, constants, tests and their imports) with the same
   `#[cfg(...)]` as the code that uses them, so non-Linux builds have no dead code.
7. MUST NOT panic on input from users, guests, files or the network; return an error instead. `expect` is for
   invariants the code itself guarantees, and its message states the invariant.
8. MUST NOT block the tokio runtime: SQLite calls hold a `std::sync::Mutex` only for the duration of one query,
   and never across an `.await`; long-running or blocking work goes to its own thread or task.
9. SHOULD name tests after the behaviour they prove (`queued_changes_apply_after_power_off`), and SHOULD put
   process-level tests in `apps/<bin>/tests/`.
10. SHOULD keep files under ~800 lines; split along an existing module boundary, not arbitrarily.

## Enforcement

| Rule | Check |
|---|---|
| 1 | `cargo xtask ci` → lint policy (`check_lint_policy`) and the `unsafe_code = "forbid"` workspace lint, in CI job `ci` |
| 2, 6 | `cargo xtask ci` → fmt check and clippy on `ubuntu-24.04`, `ubuntu-24.04-arm` and `macos-15` |
| 3 | `--locked` in every CI cargo command; `rust-toolchain.toml` pins the toolchain |
| 5 | `catalogue_codes_are_unique_and_bilingual` unit test; review for raw errors in user output |
| 4, 7–10 | Code review |

## Anti-patterns

- **A constant used only by Linux-gated code.** `READY_LINE` in `apps/virtax-vmm/src/main.rs` was unused on macOS
  and arm64, and clippy failed there. Fix: give it the same `#[cfg]` as its only user.
- **Holding a lock while calling something that takes it again.** A `std::sync::Mutex` is not reentrant; `emit()`
  in `virtaxd` takes the registry lock, so callers must release it first (see `record_state`).
- **Testing the happy path only.** Each rule above that guards an error path has a test for that path
  (`refuses_newer_registry`, `invalid_manifest_is_not_written`).

## References

- Architecture §"C4 Level 4 — Code" (crate map), ADR-001 (Engine trait seam).
- NFR-00-11 (one codebase for six host combinations), NFR-00-12 (Rust code quality), FR-GEN-007 (human errors).
- `openspec/specs/platform/build-and-ci-cd/spec.md` (FR-PL01-001, FR-PL01-003, "As built — Unsafe code policy").
