---
title: "Virtax — remaining work"
description: "Roadmap from today's control plane and KVM VMM to every module in the design set, in 13 gated phases"
status: pending
priority: P1
effort: "97 person-weeks (rough estimate)"
tags: [roadmap, openspec, virtax-vmm, control-plane, ui]
created: 2026-11-24
---

# Virtax — remaining work

## Overview

Delivers the remaining-work checklist of 2026-11-24 in full: the 26 modules not started, the 8 partly built
modules, the missing CI checks and the housekeeping. Scope is held as requested (no cuts); work that waits on an
owner decision is gated, not dropped. Requirements and acceptance criteria are the OpenSpec specs in
`openspec/specs/`; this plan only sequences them.

**How every phase runs:** scout the area → one OpenSpec change per slice (`openspec-propose`) → implement
(`openspec-apply-change`) → PR with CI green (`ci`, `kvm boot tests`, and from phase 1 on `deny`, `spec`,
`proto`, `multiuser`) → archive the change. Phases 1–2 are planned in detail; phases 3–13 are outlines that get
scouted and planned when they start.

## Phases

| # | Phase | Priority | Effort | Depends on | Status |
|---|-------|----------|--------|-----------|--------|
| 1 | [Housekeeping and CI hardening](./phase-01-start.md) | P1 | 1 pw | — | Done (remote branch deletion awaits the owner) |
| 2 | [virtio-blk over PCI](./phase-02-virtio-blk-pci.md) | P1 | 3 pw | 1 | In progress |
| 3 | [VMM security hardening](./phase-03-vmm-security-hardening.md) | P1 | 2 pw | 2 | Pending |
| 4 | [Storage, media and snapshots](./phase-04-storage-media-snapshots.md) | P1 | 8 pw | 2 | Pending |
| 5 | [Networking](./phase-05-networking.md) | P1 | 6 pw | 2 | Pending |
| 6 | [Guest integration, unattended install and import](./phase-06-guest-integration-and-install.md) | P1 | 8 pw | 4, 5 | Pending |
| 7 | [Observability, security, updates and extensions](./phase-07-observability-security-updates.md) | P2 | 8 pw | 1 | Pending |
| 8 | [Control-plane completion](./phase-08-control-plane-completion.md) | P2 | 5 pw | 1 | Pending |
| 9 | [Display and input data plane](./phase-09-display-and-input-data-plane.md) | P2 | 6 pw | 2, 7 | Pending |
| 10 | [Manager and console UI](./phase-10-ui-manager-and-console.md) | P1 | 24 pw | 8, 9 | Pending |
| 11 | [Legacy VirtualBox engine](./phase-11-legacy-virtualbox-engine.md) | P1 | 8 pw | 1 | Pending |
| 12 | [Cross-platform hosts and hypervisors](./phase-12-cross-platform-hypervisors.md) | P2 | 12 pw | 2, 3 | Pending |
| 13 | [Packaging, signing and release](./phase-13-packaging-and-release.md) | P2 | 6 pw | 1, 7 | Pending |

pw = person-weeks, a rough estimate for one experienced engineer. Phases with no dependency between them (for
example 4, 5, 7, 8 and 11) can run in parallel with separate owners.

## Owner decisions that gate phases

| Decision | Proposed default (not confirmed) | Gates |
|---|---|---|
| D1 UI framework | Slint, after a 3-week bake-off with GPUI | Phase 10 cannot start |
| D2 Phase 1 engine distribution | BYO VirtualBox 7.2 (alpha), Engine Pack (beta) | Phase 11 |
| D3 Code-signing certificates | Buy Windows OV/EV and Apple Developer ID now | Phase 13 (macOS/Windows) |
| D4 VMM base | OpenVMM crates; the code uses rust-vmm provisionally | Phases 2–9 proceed on rust-vmm; phase 12 cannot start |
| D5 Bundle ID | — (name Virtax is confirmed) | Phase 13 |
| D6 Telemetry | Off, opt-in crash reports | Phase 7 crash reports |
| Owner handles, remote branch deletion | — | Phase 1 tasks 1 and 6 |

Phase-level open questions (Q-…) are listed in each phase's Gates section; work follows their proposed defaults
and says so in its proposal, per `.principles/agentic-coding.md` rule 2.

## Success criteria

- [ ] Every requirement in `openspec/specs/` that the design set schedules for P1 or P2 is implemented and its
      scenarios pass as tests, or is explicitly deferred by the owner.
- [ ] Every "not yet enforced" item in `.principles/README.md` is enforced in CI.
- [ ] The six host combinations boot guests (NFR-00-11); signed installers ship from a tag (FR-PL01-012).

## Risks (self red-team)

- **Scale:** 97 person-weeks for one person is about two years; the design set staffs 4–6 people. Parallelize
  phases 4/5/7/8/11 or the UI phase becomes the critical path.
- **Decision latency:** D1, D2 and D4 each block a large phase; deciding them early matters more than any code.
- **Hardware for CI:** macOS/Windows hypervisor and VirtualBox runners do not exist yet (Q-12-04, phase 11).
- **Spec drift:** archiving each change keeps `openspec/specs/` current; a phase is not done until it is archived.

## Validation log

Interactive validation (owner questions) was not run: the owner declined questions in this session, so every
open decision is recorded as a gate with its proposed default instead. Run `/ak-plan validate` on this file to
resolve them.
