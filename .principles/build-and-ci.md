# Build and CI principles

Scope: `Cargo.toml`, `rust-toolchain.toml`, `xtask/`, `justfile`, `Makefile`, `.github/workflows/`, `.cache/`.

## Principles

1. **One command per job.** Every CI job runs one `cargo xtask` command, so a contributor reproduces CI locally
   with the same command (FR-PL01-002).
2. **Only Rust is required.** A clean clone builds with the pinned toolchain alone; `protoc` is vendored and no
   system library is needed (FR-PL01-001, FR-PL01-003).
3. **Reproducible inputs.** Toolchain, `Cargo.lock`, test kernel and tools are pinned, and downloads are
   checksum-verified (FR-PL01-003, FR-PL01-009).
4. **Fast feedback first, real guests always.** Unit and e2e tests run everywhere; KVM boot tests run in their own
   job on every pull request (FR-PL01-006).

## Rules

1. MUST implement automation in `xtask` (Rust), not in shell scripts; `justfile` recipes and the `Makefile` are
   thin wrappers, and the `Makefile` only forwards to `just`.
2. MUST keep `cargo xtask ci` the single gate: fmt check, lint policy, clippy `-D warnings`, all tests with
   `--locked`. A new check is added there, so local and CI runs stay identical.
3. MUST mark tests that need `/dev/kvm` and the test guest with `#[ignore = "..."]` and a `kvm_` name prefix;
   `cargo xtask kvm-test` runs exactly those.
4. MUST pin every external download with a version and SHA-256 in `xtask` and cache it under `.cache/`
   (git-ignored), never in the repository.
5. MUST pin GitHub Actions to a major version or commit, keep `permissions: contents: read` unless a job needs
   more, and run the CI matrix on `ubuntu-24.04`, `ubuntu-24.04-arm` and `macos-15`.
6. MUST track every background process a recipe starts (pidfile, fixed socket and data paths) and stop it from a
   matching recipe (`just dev-up` / `just dev-down`); a recipe MUST NOT start a second copy.
7. SHOULD keep `cargo xtask ci` under 10 minutes on a CI runner; slow suites move to their own job.

## Enforcement

| Rule | Check |
|---|---|
| 2, 5 | `.github/workflows/ci.yml` job `ci` (three hosts) |
| 3, 4 | `.github/workflows/ci.yml` job `kvm boot tests`; `cargo xtask kvm-test` refuses to run without the cached guest |
| 6 | `just dev-up` checks the pidfile; `virtaxd` refuses a second instance on the same data directory |
| 1, 7 | Code review |
| — | Not yet enforced: release pipeline, SBOM and signed artifacts (FR-PL01-012), performance gates (FR-PL01-007), OpenSpec validation in CI (run `just spec` locally) |

## Anti-patterns

- **Assuming an unrun workflow passes.** The workflow had never run before the first PR, and two of four jobs
  failed on cfg-gated code. Fix: push early, read the failing job's log, and fix what it shows rather than what you expect.
- **A runner command that hides output.** Piping test output through `grep` and `head` dropped the result line,
  and a run looked done when it was not. Fix: check the `test result:` lines or the exit status, not a filtered
  tail.

## References

- `openspec/specs/platform/build-and-ci-cd/spec.md` (FR-PL01-001 to FR-PL01-012 and the as-built requirements).
- Architecture §"CI/CD & environments".
