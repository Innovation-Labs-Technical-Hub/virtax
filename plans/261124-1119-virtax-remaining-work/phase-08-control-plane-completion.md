---
phase: 8
title: "Control-plane completion"
status: pending
priority: P2
effort: "5 person-weeks (estimate)"
dependencies: [1]
---

# Phase 8: Control-plane completion

## Goal

The partly built control-plane modules reach their full BRD scope.

## Scope

- CP-01 API and IPC (`control-plane/control-plane-api-and-ipc`): capability tokens (if not done in phase 7),
  REST/JSON gateway and mTLS listener off by default (FR-CP01-013), Rust and Python client SDKs (012).
- CP-02 registry (`control-plane/registry-and-configuration-store`): `.vbox` shadowing for legacy VMs
  (FR-CP02-010, with phase 11), `ui_prefs` API (013), escape-hatch allow-list (014), per-snapshot manifests (016).
- CP-03 lifecycle (`control-plane/vm-lifecycle-and-runtime-orchestration`): pause/resume, save state, supervisor,
  ACPI timeout escalation policy.
- CP-15 CLI (`control-plane/virtaxctl-cli`): completions and man pages, `vboxmanage` translation shim, selectors,
  bulk actions and dry-run (FR-CP15-005/006/009), `virtaxctl daemon start|stop|status` (011).

## Gates

- **Q-05-01** what closing the console does; **Q-05-02** ACPI timeout and escalation; **Q-05-03** host sleep
  pauses or saves VMs.
- **Q-09-01** daemon exits after long idle; **Q-09-02** ship the Python SDK in P1.

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

One OpenSpec change per module gap, each a PR: `add-pause-and-save-state` (after phase 4's device-state save),
`add-cli-completions-and-selectors`, `add-vboxmanage-shim`, `add-ui-prefs-api`, `add-remote-gateway`,
`add-client-sdks`.

## Verification

- Each change's acceptance scenarios pass; FR-CP15-006 acceptance (≥ 90% of 50 public scripts translate) is
  measured with a checked-in script corpus.
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- The REST/mTLS gateway widens the attack surface → off by default, CI asserts no TCP port opens by default.
