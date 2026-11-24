# Spec Delta

## ADDED Requirements

### Requirement: Disks from the machine manifest
`VirtaxVmmEngine` SHALL attach every disk attachment of a `virtio` storage controller in `machine.toml` to the VM, in controller then port order, resolving relative paths against the machine folder. Source: FR-EN04-001 with the manifest's `[[storage.controllers]]` (FR-CP02-004).

#### Scenario: Manifest disk reaches the guest
- **WHEN** a machine has a `virtio` controller with one disk attachment and is started
- **THEN** the guest sees that image as a virtio block device
- **AND** this is verified by `kvm_manifest_disk_reaches_the_guest_through_virtaxctl` (`apps/virtaxctl/tests/e2e.rs`)

#### Scenario: Read-only attachment
- **WHEN** the attachment is a `dvd`, or the image file is not writable by the user
- **THEN** the image is attached read-only
- **AND** this is verified by `vmm::tests::disks_follow_controller_then_port_order` and `vmm::tests::an_image_the_user_cannot_write_is_attached_read_only` (`crates/virtax-engine`)

### Requirement: Disk configuration is checked before start
`VirtaxVmmEngine` SHALL refuse to start a machine whose disks it cannot attach, with `VTX-ENG-011` and a message naming the attachment, before `virtax-vmm` runs. Source: FR-EN01-011 (errors with fixes) applied to the disk configuration.

#### Scenario: Unsupported bus
- **WHEN** a machine with a disk on a `sata`, `ide`, `scsi` or `nvme` controller is started
- **THEN** the start fails with `VTX-ENG-011` naming the controller and saying only `virtio` is supported
- **AND** this is verified by `vmm::tests::disks_that_cannot_be_attached_are_refused_before_start`

#### Scenario: Missing image
- **WHEN** an attachment's image file does not exist
- **THEN** the start fails with `VTX-ENG-011` naming the path, and the machine stays `powered_off`
- **AND** this is verified by `vmm::tests::disks_that_cannot_be_attached_are_refused_before_start` and `kvm_manifest_disk_reaches_the_guest_through_virtaxctl`
