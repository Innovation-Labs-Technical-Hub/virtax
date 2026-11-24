---
phase: 9
title: "Display and input data plane"
status: pending
priority: P2
effort: "6 person-weeks (estimate)"
dependencies: [2, 7]
---

# Phase 9: Display and input data plane

## Goal

`virtax-vmm` guests have a framebuffer and input devices exposed through the shared-memory display protocol, ready for the console window.

## Scope

- EN-02 display and input data plane (`engine/display-and-input-data-plane`): DisplayChannel setup, SHM ring v1
  with the `VXFB` header, triple buffering, damage coalescing, cursor and input channels, capability tokens,
  resize hints (FR-EN02-002 to 011, 015, 016). The `vbx-shmfb` adapter inside VirtualBox (001, 012, 013) belongs to
  phase 11.
- EN-07 graphics and audio (`engine/graphics-and-audio`): virtio-gpu 2D baseline, ramfb, virtio-snd, host audio
  backends (FR-EN07-001, 002, 007, 009, 010); 3D (003–005) stays Phase 3.
- virtio-input devices for keyboard and pointer (EN-04 FR-EN04-001).

## Gates

- **Q-04-01** one console process per VM or one for all; **Q-04-03** default present mode.
- **Q-10-03** Windows-guest 3D policy (proposed: legacy engine).

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

Split into OpenSpec changes, each a PR: `add-virtio-gpu-2d`, `add-display-channel` (SHM ring, tokens,
`OpenDisplay` RPC), `add-virtio-input`, `add-virtio-snd`. Provide a headless test client that reads frames and
injects input, so the data plane is tested before any UI exists.

## Verification

- A `kvm_*` test boots a guest to a framebuffer console, reads frames through the SHM ring and types a command
  through the input channel; frame-time and idle-CPU numbers are recorded against NFR-04-*.
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- SHM protocol changes after the UI starts using it → version the header (`VXFB`, version, generation) from the
  first commit and test compatibility.
