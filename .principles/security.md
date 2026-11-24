# Security principles

Scope: the whole repository — daemon, CLI, VMM, build and release, CI and contributor workflow.

## Principles

1. **Know the trust boundaries.** User ↔ `virtaxd` (same UID) ↔ a future elevated helper ↔ VM process ↔ guest
   (untrusted). Each crossing validates its input (architecture §"Security & access control").
2. **Least privilege.** Nothing runs elevated except a future, minimal helper; the daemon and VMM run as the user
   (NFR-00-08).
3. **Local only by default.** The control plane listens on a private Unix socket; no TCP port opens unless a user
   configures one (FR-CP01-001, FR-CP01-013).
4. **Private by default.** No telemetry; diagnostics redact secrets and personal paths (FR-GEN-008).
5. **Every dependency is a decision.** Third-party code runs with the user's rights, so it is pinned, licensed
   compatibly with GPL-3.0 and reviewed (NFR-00-13).

## Rules

1. MUST create the daemon socket with mode 0600 in a 0700 directory and MUST reject any peer whose UID differs
   from the daemon's (`check_peer` in `apps/virtaxd/src/grpc.rs`), on every service.
2. MUST NOT add a network listener, outbound connection or telemetry without an explicit opt-in setting and an
   OpenSpec change that names the requirement (FR-CP01-013, FR-GEN-008, decision D6).
3. MUST treat guest data, manifests, image files and downloaded artifacts as untrusted: parse with bounds checks,
   reject what does not validate, and never execute or `eval` them.
4. MUST verify every downloaded artifact against a pinned SHA-256 before use (see `cargo xtask guest-kernel`) and
   pin tool and action versions; MUST NOT pipe a download into a shell.
5. MUST NOT log or store secrets (passwords, keys, tokens) in plain text; future disk and NVRAM keys go to the OS
   keystore (architecture §"Encryption").
6. MUST NOT commit secrets, `.env` files, private keys or personal data; review `git diff --cached` before every
   commit.
7. MUST keep every dependency's licence compatible with GPL-3.0-or-later.
8. MUST NOT kill, signal or delete processes and files the code did not create; identify a child by the PID it
   was given, never by a pattern match.
9. SHOULD keep refusal paths tested: a second user, a stale etag, a locked VM folder, an expired event cursor.

## Enforcement

| Rule | Check |
|---|---|
| 1 | Socket-mode code in `apps/virtaxd/src/main.rs`; `check_peer` on every service; e2e tests run against the 0600 socket; `cargo xtask multiuser-test` (CI job `multi-user socket test`) proves a second OS user is refused by both the socket permissions and the peer check (`VTX-API-403`) |
| 4 | `xtask/src/guest.rs` refuses a kernel package whose checksum differs |
| 9 | `etag_conflict_and_name_uniqueness`, `second_holder_is_refused_until_release`, `second_daemon_is_refused`, `stale_or_future_cursor_requires_resync` |
| 2, 3, 5, 6, 8 | Code review |
| 7 | `cargo xtask deny` (`deny.toml`: licences, advisories, bans, crates.io only; then `cargo vet` against `supply-chain/`) in CI job `supply chain (deny, vet)`; daily `cargo xtask audit` files an issue on a new advisory (`.github/workflows/audit.yml`). Not yet enforced: an SBOM per release (FR-PL01-004) |

## Anti-patterns

- **Killing by pattern.** `pkill -f` and `pgrep -f` matched the shell running the command as well as the VM, and
  the signal missed. Fix: keep the PID of the child you spawned and signal that.
- **Sweeping staging.** `git add -A` committed edits that belonged to someone else. Fix: stage explicit paths and
  read the staged diff (rule 6).

## References

- Architecture §"Security & access control", §"Data protection & privacy".
- NFR-00-08, NFR-00-13; FR-CP01-001/002/013, FR-CP12-*, FR-GEN-008; `openspec/specs/control-plane/security-and-encryption/spec.md`.
