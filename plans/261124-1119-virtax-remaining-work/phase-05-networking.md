---
phase: 5
title: "Networking"
status: pending
priority: P1
effort: "6 person-weeks (estimate)"
dependencies: [2]
---

# Phase 5: Networking

## Goal

Machines on `virtax-vmm` have virtio-net with NAT, port forwarding, host-only and bridged networks.

## Scope

- EN-06 network backends (`engine/network-backends`): virtio-net multi-queue (FR-EN06-001), user-mode NAT
  (002/003), port forwarding (004), TAP + vhost-net (005), DHCP/DNS (009), MTU (010), unprivileged backends with a
  narrow helper (013).
- CP-06 network management (`control-plane/network-management`): NICs as a list, NAT with live port forwarding,
  conflict detection, host-only, bridged, internal networks (FR-CP06-001 to 011).
- PL-03 host integration, helper part (`platform/host-integration-and-power`): `virtax-helper` with polkit
  activation on Linux (FR-PL03-001/002), the only elevated process (NFR-00-08).

## Gates

- **Q-07-01** default Linux NAT backend: passt or libslirp.
- **Q-07-04** port forwards bind 127.0.0.1 by default or all interfaces (security-relevant).
- **Q-07-05** MAC prefix for new VMs.
- **Q-07-03** legacy and `virtax-vmm` VMs on the same network.

## Tasks & Steps

Outline only: scout this phase's area before proposing its first OpenSpec change (deep mode), then plan each
change in detail with `openspec-propose`.

Split into OpenSpec changes, each a PR:
- [ ] 1. `add-virtio-net-nat`: virtio-net single queue with user-mode NAT (default per Q-07-01) and port forwarding.
- [ ] 2. `add-network-management`: CP-06 NIC list, live port forwards with conflict detection, `virtaxctl net`.
- [ ] 3. `add-virtax-helper`: the privileged helper (≤ 3k LOC, narrow API, audit log, fuzzed request parser), polkit
      activation, TAP creation.
- [ ] 4. `add-host-only-and-bridged`: TAP + vhost-net, host-only and bridged networks through the helper.
- [ ] 5. `add-virtio-net-multiqueue`: multi-queue and offloads.

## Verification

- `kvm_*` tests: guest gets a DHCP lease, reaches the host through a forwarded port, and two guests talk on an
  internal network; iperf3 numbers recorded.
- The helper's API is covered by unit tests and a fuzz target; it runs only through polkit.
- Every change in this phase merged with `cargo xtask ci`, `cargo xtask kvm-test` (when the engine changed) and
  `openspec validate --all --strict` green, and archived into the main specs.

## Risks

- An elevated helper is the largest new attack surface → narrow typed API, fuzzed parser, audit log, external
  review before GA (Q-11-03).
- CI runners cannot create bridges → host-only/bridged tests run in the KVM job with `sudo`, or on a self-hosted
  runner.
