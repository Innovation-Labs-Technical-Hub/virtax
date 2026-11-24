---
phase: 10
title: "Manager and console UI"
status: pending
priority: P1
effort: "24 person-weeks (estimate)"
dependencies: [8, 9]
---

# Phase 10: Manager and console UI

## Goal

The desktop Manager and Console exist on the chosen UI framework, covering the eight UI modules.

## Scope

- UI-08 design system, theming, accessibility and i18n first; then UI-01 app shell, UI-02 VM library, UI-04
  settings sheet, UI-03 quick create, UI-05 console window, UI-06 VM tools, UI-07 global tools (all
  `openspec/specs/ui/`).
- Bilingual VI/EN (FR-GEN-001), WCAG 2.2 AA (NFR-00-10), the budgets in NFR-00-01 to 04.

## Gates

- **D1** UI framework (proposed: Slint after a 3-week bake-off against GPUI) — this phase does not start until
  D1 is CONFIRMED.
- **Q-04-02** keyboard auto-capture; **Q-04-04** macOS host key; **Q-04-05** recording in P1; **Q-03-01** ask or
  queue power-off changes in the settings sheet.

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

- [ ] 1. Run the D1 bake-off (two prototypes of the VM list and console on Slint and GPUI, measured against the
      architecture document's criteria) and record the decision.
- [ ] 2. `add-design-system`: tokens from the design set, components, theming, a11y and i18n infrastructure.
- [ ] 3. One OpenSpec change per UI module, each a PR, in the order above; the console window builds on phase 9.

## Verification

- UI end-to-end tests run on the MockEngine on every CI host (FR-PL01-011), with the 50 ms UI-thread budget;
  accessibility checks for every screen.
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- The largest phase; it needs a UI lead → staff it before starting, and keep the bake-off to its timebox.
