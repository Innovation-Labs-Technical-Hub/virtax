---
phase: 1
title: "Housekeeping and CI hardening"
status: in-progress
priority: P1
effort: "1 person-week (estimate)"
dependencies: []
---

# Phase 1: Housekeeping and CI hardening

## Goal

Every check the principles mark "not yet enforced" that needs no product decision runs in CI, and the repository
has no stale branches.

## Scope

- FR-CP12-011 supply-chain gates (`cargo-deny` licences and advisories, `cargo-audit`) — Must, P1.
- FR-CP01-002 peer-credential check, proven with a second OS user.
- FR-CP01-003 / FR-CP01-010 `buf lint` and `buf breaking` for `virtax.v1`.
- OpenSpec strict validation in CI (`.principles/agentic-coding.md` rules 1–2).
- Out of scope here: SBOM and provenance (FR-CP12-012) move to phase 13 with the release pipeline; fuzzing and
  seccomp move to phase 3.

## Gates

- **Remote branch deletion** needs the owner's confirmation (outward-facing, not reversible from the CLI).
- **Owner handles** for `.principles/README.md` and `AGENTS.md` need the owner's input; until then roles stay.

## Files to Create / Modify

- Create: `deny.toml`, `buf.yaml` (repo root; module path `crates/virtax-api/proto`), `supply-chain/` (cargo-vet), `xtask/src/multiuser.rs`, `.github/workflows/audit.yml`
- Modify: `xtask/src/main.rs` (commands `deny`, `spec`, `proto`, `multiuser-test`), `.github/workflows/ci.yml`,
  `justfile`, `.principles/*.md` and `.principles/README.md` (enforcement tables), `AGENTS.md`

## Tasks & Steps

- [ ] 1. Switch to `main`, fast-forward to `origin/main`, and delete the local merged branches; list the four remote
      merged branches and delete them only after the owner confirms.
- [x] 2. Add `deny.toml`: GPL-3.0-or-later-compatible licence allowlist (MIT, Apache-2.0, BSD-2/3-Clause, ISC,
      Unicode-3.0, Zlib, MPL-2.0, GPL-3.0), advisories deny, crates.io as the only source; add `cargo xtask deny`
      (runs `cargo deny check` and `cargo audit`) and a CI job using a pinned install action. Verify: the job is
      green, and a deliberately disallowed licence in a scratch branch makes it fail.
- [x] 3. Add `cargo xtask spec` (pinned `npx @fission-ai/openspec@2.1.1 validate --all --strict --no-interactive`) and
      run it in CI; point `just spec` at it. Verify: CI fails on a scratch branch with a requirement lacking a scenario.
- [x] 4. Add `buf.yaml` with lint rules for the proto; `cargo xtask proto` runs `buf lint` and, on pull requests,
      `buf breaking --against` the base branch (pinned buf). Fix existing lint findings without breaking the wire
      format. Verify: renaming a field on a scratch branch fails the job.
- [ ] 5. Add `cargo xtask multiuser-test` (Linux CI only, uses `sudo useradd`): start `virtaxd` as the runner user,
      loosen the socket mode as root, connect as the second user, and expect `PERMISSION_DENIED` with `VTX-API-403`;
      also expect `EACCES` with the default 0700/0600 permissions. Verify: the job is green, and disabling
      `check_peer` on a scratch branch makes it fail.
- [x] 6. Update every `.principles` enforcement table and the "not yet enforced" list; update `AGENTS.md` commands.
      Verify: `cargo xtask ci` and `just spec` pass; every listed command was run.

## Verification

- CI on the phase's pull request: `ci` (3 hosts), `kvm boot tests`, and the new `deny`, `spec`, `proto` and
  `multiuser` jobs all green.
- Each new check was shown to fail on a deliberate violation (scratch branch, not merged).

## Risks

- An advisory in a transitive dependency blocks CI → fix by upgrading; an exception in `deny.toml` needs a
  comment and an expiry date.
- `buf lint` defaults clash with the existing proto style → choose rules that keep `virtax.v1` wire-compatible.
