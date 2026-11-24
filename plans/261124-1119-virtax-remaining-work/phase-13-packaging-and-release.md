---
phase: 13
title: "Packaging, signing and release"
status: pending
priority: P2
effort: "6 person-weeks (estimate)"
dependencies: [1, 7]
---

# Phase 13: Packaging, signing and release

## Goal

Signed installers for every supported host are built, attested and published from a tag, and performance regressions block merges.

## Scope

- PL-02 packaging, signing and installers (`platform/packaging-signing-and-installers`): `.deb`, `.rpm`,
  Flatpak/AppImage, notarized macOS `.app`/`.dmg`, MSIX/MSI.
- PL-01 remaining FRs (`platform/build-and-ci-cd`): release pipeline and provenance (FR-PL01-012), performance
  gates with `perf-budgets.toml` (007), cross-compilation (005), Engine Pack pipeline (008), pinned firmware (009).
- CP-12 SBOM, provenance and reproducible builds (FR-CP12-012); CP-14 update manifest publishing.

## Gates

- **D3** code-signing certificates purchased (EV and attestation take 2–6 weeks).
- **D5** bundle ID (the product name Virtax is CONFIRMED).
- **Q-11-01** signature scheme; **Q-12-01** universal2 or per-arch macOS builds; **Q-12-02** Windows signing
  service; **Q-12-03** Snap support.

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

- [ ] 1. `add-perf-gates`: `perf-budgets.toml` from NFR-00-01 to 05 and the engine NFRs, a nightly job that fails on a
      breach or a > 5% regression.
- [ ] 2. `add-linux-packages`: `.deb`/`.rpm` with systemd user units and polkit policy, SBOM (CycloneDX) and checksums.
- [ ] 3. `add-release-pipeline`: tag → build → test → sign → package → attest → GitHub Release + update manifest.
- [ ] 4. `add-macos-and-windows-packages` once D3 certificates exist.

## Verification

- A test tag produces signed, attested artifacts with an SBOM in under 2 hours with no manual steps
  (FR-PL01-012 acceptance); the perf job fails on a deliberately added 1 ms timer (FR-PL01-007 acceptance).
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- Certificates arrive late → start D3 now; Linux packages and the pipeline do not depend on it.
