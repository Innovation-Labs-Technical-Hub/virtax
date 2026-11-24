---
title: virtio-blk over PCI for virtax-vmm
date: 2026-11-24
summary: "Phase 1 merged; phase 2 (add-virtio-blk-pci, 17/17 tasks) implemented in PR #5 with all CI jobs green"
---

# virtio-blk over PCI for virtax-vmm

## What happened
- PR #4 (phase 1: supply-chain, spec, proto and multi-user gates) is merged into main.
- Phase 2 implemented the OpenSpec change `add-virtio-blk-pci` in PR #5: a PCIe root complex (ECAM, MCFG,
  PNP0A08 host bridge, PNP0C02 reservation), MSI-X routed through KVM, the virtio 1.2 modern PCI transport,
  virtio-blk on raw images with flush and locking, `virtax-vmm --disk`, and manifest disks in VirtaxVmmEngine.
- Tests went from 65 to 90, and the KVM tests from 6 to 13; every CI job passes, including the KVM job.

## Problems and fixes
- Linux ignores ECAM unless it is reserved: reserving it in e820 (and with PNP0C02) made the early MMCONFIG
  probe succeed on the first boot.
- KVM_SET_GSI_ROUTING replaces the whole table, so the hv crate re-sends the default irqchip routes with every
  MSI route.
- `--disk <path>,ro` alone made paths containing commas ambiguous; the CLI now accepts `,rw` too and the engine
  always writes the mode. design.md records it.
- macOS and arm64 CI failed on dead code in the stub `run()`; a host-scoped `allow(dead_code)` fixed it, checked
  locally with clippy for aarch64 Linux and aarch64 macOS.
- This host's DNS intermittently failed for github.com; pushes and CI polling were retried.

## Decision
- No new unsafe was needed: kvm-ioctls wraps GSI routing and ioeventfd safely.
- virtio-queue 0.18.0 and virtio-bindings 0.2.7 are cargo-vet exemptions, not audits.

## Next steps
- Merge PR #5 (needs the owner's go-ahead), then `openspec archive add-virtio-blk-pci`.
- Phase 3: VMM security hardening.

> Historical work record — not durable authority. Prefer docs/specs/ADRs for current decisions.
