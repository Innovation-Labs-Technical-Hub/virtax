# Proposal

## Why

`virtax-vmm` guests boot from an initramfs and have no disk, so nothing they do survives a power-off. Block
storage is the next step toward booting real distributions on the Phase 2 engine, and every later device
(network, GPU, vsock) needs the same PCI and virtio groundwork.

## What Changes

- Guests get a PCIe root complex: ECAM configuration space described by an ACPI MCFG table and a DSDT host bridge,
  with MSI-X interrupts. Guests no longer boot with `pci=off`. (FR-EN04-006, partial: no root-port hot-plug,
  fw_cfg, pflash or SMBIOS yet.)
- Each disk attachment on a `virtio` storage controller in `machine.toml` appears in the guest as a virtio 1.2
  modern PCI block device (FR-EN04-001, block only; one queue).
- Disks are raw image files, read-write or read-only (FR-EN05-001, raw only), with writeback caching and guest
  flushes made durable on the host (FR-EN05-007, writeback mode only).
- An image is locked while a VM uses it, so two VMs never write the same file (FR-EN05-012, locking only).
- `VirtaxVmmEngine` rejects disks it cannot attach (other buses, missing files, non-raw formats) with
  `VTX-ENG-011` before the VMM starts.

Out of scope: qcow2, VDI and the other formats (rest of FR-EN05-001), io_uring and the other async backends
(FR-EN05-002 to 004), multi-queue (FR-EN05-006), discard/write-zeroes (FR-EN05-008), NVMe, AHCI and CD media
(FR-EN04-002/003), virtio-net and the other virtio devices, disk creation and `virtaxctl` media commands (CP-05).

Decisions this change depends on:
- **D4 / Q-10-01** (OpenVMM crates or rust-vmm), TO BE CONFIRMED. This change follows the as-built path, rust-vmm
  (`virtio-queue` and friends), behind `virtax-vmm-hv` and the core crate's device traits, so that a later D4
  decision can swap the implementation without changing these specs.
- **Q-06-01** (default disk format for new VMs), TO BE CONFIRMED. Not decided here: this change only attaches
  existing raw images and creates none.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `engine/device-model-and-firmware`: adds the PCIe root complex and the virtio-blk device.
- `engine/block-backends`: adds raw images, flush durability and image locking.
- `engine/virtax-vmm-core-and-hypervisor-backends`: adds disk attachment from the manifest and its validation.

## Impact

- Code: `crates/virtax-vmm-core` (PCI bus, ECAM, MSI-X, virtio transport and block device, ACPI MCFG and DSDT
  host bridge), `crates/virtax-vmm-hv` (MSI routing), `apps/virtax-vmm` (`--disk` arguments),
  `crates/virtax-engine` (disk validation and arguments), `xtask` (a test disk image for the KVM tests).
- Dependencies: rust-vmm `virtio-queue` (and `virtio-bindings` if needed), pinned.
- Behaviour visible to existing guests: the kernel command line loses `pci=off`, so guests now enumerate PCI.
- Docs: `docs/virtax-vmm.md`.
