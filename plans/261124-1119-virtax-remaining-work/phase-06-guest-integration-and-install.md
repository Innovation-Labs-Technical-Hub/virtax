---
phase: 6
title: "Guest integration, unattended install and import"
status: pending
priority: P1
effort: "8 person-weeks (estimate)"
dependencies: [4, 5]
---

# Phase 6: Guest integration, unattended install and import

## Goal

Guests are installed unattended from an ISO or cloud image, integrate with the host through an agent, and existing VirtualBox VMs and OVF/OVA appliances can be imported.

## Scope

- EN-08 guest agent and drivers (`engine/guest-agent-and-drivers`): `virtax-agent` over virtio-vsock with a
  versioned protocol, clipboard, time sync, graceful shutdown, exec, file copy, guest info, virtio-fs mounts
  (FR-EN08-001 to 013).
- CP-07 guest integration (`control-plane/guest-integration`): FR-CP07-001 to 012.
- CP-10 unattended install and OS catalogue (`control-plane/unattended-install-and-os-catalogue`): FR-CP10-001 to 012.
- CP-09 OVF/OVA and VirtualBox migration (`control-plane/ovf-ova-appliances-and-virtualbox-migration`): including
  the direct `.vbox` reader (FR-CP09-006) shared with CP-02 (FR-CP02-011).

## Gates

- **Q-08-01** agent protocol: own protobuf or qemu-guest-agent compatible.
- **Q-08-02** exec without guest credentials; **Q-08-05** default clipboard mode.
- **Q-08-03** bundle virtio-win or download on demand; **Q-08-04** emulate VMMDev.
- **Q-02-01** install in background by default; **Q-02-03** default export format; **Q-02-05** cross-engine linked clones.

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

Split into OpenSpec changes, each a PR:
- [ ] 1. `add-virtio-vsock` (device) and `add-virtax-agent` (guest binary, protocol, packaging for Linux first).
- [ ] 2. `add-guest-integration`: clipboard, time sync, ACPI-free graceful shutdown, guest info and IPs in the read model.
- [ ] 3. `add-unattended-install`: OS catalogue, ISO detection, template engine, Ubuntu autoinstall + cloud-init first,
      then Windows autounattend; the whole pipeline as one Operation.
- [ ] 4. `add-vbox-reader-and-import`: `.vbox` 1.3–1.22 reader, discovery of existing VirtualBox VMs, device-compatibility
      report, conversion to `machine.toml`.
- [ ] 5. `add-ovf-import-export`: OVF/OVA inspect, import, export with manifests and signatures.

## Verification

- `kvm_*` tests: an Ubuntu cloud image installs unattended and reports its IP through the agent; clipboard and
  file copy round-trip; a sample `.vbox` corpus parses 100% (FR-CP02-011 acceptance).
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- Unattended installs are slow for CI → use cloud images in the per-PR job and full ISO installs nightly.
- Secrets in install templates → FR-CP10-010 handling, redaction tests, never logged.
