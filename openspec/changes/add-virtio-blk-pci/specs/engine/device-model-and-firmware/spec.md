# Spec Delta

## ADDED Requirements

### Requirement: PCIe root complex
`virtax-vmm` SHALL give x86_64 guests a PCIe root complex whose configuration space is reachable through ECAM, described by an ACPI MCFG table and a DSDT host bridge, with MSI-X interrupts available to devices. Source: FR-EN04-006 (partial: no root-port hot-plug, fw_cfg, pflash or SMBIOS).

#### Scenario: Guest enumerates PCI
- **WHEN** a Linux guest boots on `virtax-vmm` without `pci=off`
- **THEN** it finds the host bridge through ACPI and lists the attached devices under `/sys/bus/pci/devices`

#### Scenario: No devices attached
- **WHEN** a VM has no PCI devices
- **THEN** the guest still boots, power-off and the power button still work, and the PCI bus holds only the host bridge

### Requirement: virtio-blk device
`virtax-vmm` SHALL present each disk it is given as a virtio 1.2 modern PCI block device (virtio-blk) with one request queue, signalled through MSI-X, reporting the image size as its capacity and its read-only flag. Source: FR-EN04-001 (block only, single queue).

#### Scenario: Disk visible to the guest
- **WHEN** a VM starts with one raw image of 64 MiB attached read-write
- **THEN** the guest sees one virtio block device of 64 MiB that it can read and write

#### Scenario: Read-only disk
- **WHEN** a disk is attached read-only and the guest tries to write to it
- **THEN** the write fails in the guest and the image file is unchanged

#### Scenario: Several disks
- **WHEN** a VM starts with two disks
- **THEN** the guest sees two block devices, in the order the disks were given
