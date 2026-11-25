# Spec Delta

## ADDED Requirements

### Requirement: PCIe root complex
`virtax-vmm` SHALL give x86_64 guests a PCIe root complex whose configuration space is reachable through ECAM, described by an ACPI MCFG table and a DSDT host bridge, with MSI-X interrupts available to devices. Source: FR-EN04-006 (partial: no root-port hot-plug, fw_cfg, pflash or SMBIOS).

#### Scenario: Guest enumerates PCI
- **WHEN** a Linux guest boots on `virtax-vmm` without `pci=off`
- **THEN** it finds the host bridge through ACPI and lists the attached devices under `/sys/bus/pci/devices`
- **AND** this is verified by `kvm_guest_enumerates_pci_and_finds_only_the_host_bridge` and `kvm_flushed_disk_writes_reach_the_image_even_if_the_vmm_is_killed` (`[1af4:1042]`)

#### Scenario: No devices attached
- **WHEN** a VM has no PCI devices
- **THEN** the guest still boots, power-off and the power button still work, and the PCI bus holds only the host bridge
- **AND** this is verified by `kvm_guest_enumerates_pci_and_finds_only_the_host_bridge`, `kvm_boots_four_vcpus_and_powers_off_through_acpi` and `kvm_sigterm_presses_the_power_button_and_the_guest_shuts_down`

### Requirement: virtio-blk device
`virtax-vmm` SHALL present each disk it is given as a virtio 1.2 modern PCI block device (virtio-blk) with one request queue, signalled through MSI-X, reporting the image size as its capacity and its read-only flag. Source: FR-EN04-001 (block only, single queue).

#### Scenario: Disk visible to the guest
- **WHEN** a VM starts with one raw image of 64 MiB attached read-write
- **THEN** the guest sees one virtio block device of 64 MiB that it can read and write
- **AND** this is verified by `kvm_flushed_disk_writes_reach_the_image_even_if_the_vmm_is_killed` (64 MiB, `sectors=131072 ro=0`)

#### Scenario: Read-only disk
- **WHEN** a disk is attached read-only and the guest tries to write to it
- **THEN** the write fails in the guest and the image file is unchanged
- **AND** this is verified by `kvm_read_only_disk_refuses_guest_writes` and `virtio::block::tests::a_read_only_disk_refuses_writes`

#### Scenario: Several disks
- **WHEN** a VM starts with two disks
- **THEN** the guest sees two block devices, in the order the disks were given
- **AND** this is verified by `kvm_two_disks_appear_in_the_order_given`
