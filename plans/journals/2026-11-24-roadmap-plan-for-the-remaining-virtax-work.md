---
title: Roadmap plan for the remaining Virtax work
date: 2026-11-24
summary: "13-phase gated plan covering the 26 unstarted and 8 partial modules, CI gaps and housekeeping"
---

# Roadmap plan for the remaining Virtax work

## What happened

The owner asked to fulfil the remaining-work checklist. An autonomous `/ak-loop` did not fit (no numeric metric;
several items are owner decisions), and the owner declined the clarifying questions, so the work became a plan:
`plans/261124-1119-virtax-remaining-work/` with 13 phases and 50 tracked tasks.

- Phases 1–2 are detailed: CI hardening (cargo-deny/cargo-audit per FR-CP12-011, OpenSpec validation, buf
  lint/breaking, a second-OS-user socket test) and the already-specified `add-virtio-blk-pci` change.
- Phases 3–13 are outlines (deep mode) to be scouted and split into OpenSpec changes when they start:
  VMM hardening, storage/snapshots, networking, guest integration/install/import, observability/security/updates,
  control-plane completion, display data plane, UI, legacy VirtualBox engine, cross-platform hypervisors, release.
- Effort is a rough 97 person-weeks for one engineer.

## Decisions

- Scope held in full; nothing cut. Owner decisions (D1–D6, phase-level Q-…) are gates with proposed defaults
  rather than blockers: phases 2–9 proceed on rust-vmm while D4 is open; the UI phase does not start before D1.
- Every phase runs as OpenSpec change → PR with CI green → archive, so the specs stay the source of truth.

## Findings

- `ak plan add-phase` does not update the phases table in `plan.md`; it was written by hand and `ak plan
  validate` accepted it.
- `ak plan status` counts only checkbox tasks; numbered steps showed "0/0 tasks" until converted to checkboxes.
- A consistency sweep confirmed every plan link resolves and all 94 cited requirement/question IDs and 23
  capability paths exist in `openspec/specs/`.

## Next steps

- Owner: decide D1, D2, D3 (start certificate purchase), D4; confirm remote branch deletion and owner handles.
- Implementation starts with phase 1 (`/ak-cook plans/261124-1119-virtax-remaining-work/plan.md`) once approved.

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
