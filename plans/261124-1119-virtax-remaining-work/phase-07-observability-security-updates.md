---
phase: 7
title: "Observability, security, updates and extensions"
status: pending
priority: P2
effort: "8 person-weeks (estimate)"
dependencies: [1]
---

# Phase 7: Observability, security, updates and extensions

## Goal

Users and support can see metrics, logs and diagnostics; disks and state can be encrypted; the product updates itself safely; plugins and USB have their first slices.

## Scope

- CP-11 metrics, logs and diagnostics (`control-plane/metrics-logs-and-diagnostics`): Metrics.Watch, structured
  logs with rotation, diagnostics bundle with redaction, Prometheus exporter, health page / `virtaxctl doctor`.
- CP-12 security and encryption (`control-plane/security-and-encryption`), remaining FRs: capability tokens
  (FR-CP12-003), encryption with OS keystores (008), VirtualBox AES-XTS compatibility (009), TPM and Secure Boot
  policy (010), secrets hygiene (015); EN-05 encryption (FR-EN05-011).
- CP-14 updates and release channel (`control-plane/updates-and-release-channel`).
- CP-13 plugins and extensions (P2, Could) and CP-08 USB and passthrough (P2, Could/Should): first slices only.

## Gates

- **D6** telemetry (proposed: off, opt-in crash reports) — gates crash reports (FR-CP11-009).
- **Q-09-03** Prometheus exporter port and default; **Q-09-04** crash-report endpoint.
- **Q-11-01** signature scheme (minisign or Sigstore); **Q-11-04** automatic updates by default.
- **Q-11-02** Secure Boot PK per VM or project-wide; **Q-11-05** central plugin index.

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

Split into OpenSpec changes, each a PR, ordered by dependency:
- [ ] 1. `add-metrics-and-logs` (CP-11 core), then `add-diagnostics-bundle`.
- [ ] 2. `add-capability-tokens` (needed by the console data plane in phase 9).
- [ ] 3. `add-disk-encryption` with OS keystore integration.
- [ ] 4. `add-update-channel` (manifest format, signature check, staged apply, rollback; publishing waits for phase 13).
- [ ] 5. `add-plugin-host` and `add-usb-passthrough-linux`, scoped to their Should/Must P2 items.

## Verification

- Each change's acceptance scenarios pass; the diagnostics bundle redaction test covers passwords, keys and home
  paths; idle-wakeup budgets (NFR-00-02) are measured with no subscribers.
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- Key management mistakes lose user data → keys only in the OS keystore, recovery tested, external review.
