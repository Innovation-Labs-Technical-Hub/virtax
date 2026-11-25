# Spec Delta

## ADDED Requirements

### Requirement: Raw disk images
`virtax-vmm` SHALL read and write raw disk images and SHALL refuse any other image format before the guest starts, naming the file and the format it found. Source: FR-EN05-001 (raw only; qcow2, VDI, VMDK, VHD and VHDX come later).

#### Scenario: Data persists across a power-off
- **WHEN** the guest writes a known pattern to its disk, flushes it and powers off
- **THEN** the image file on the host contains that pattern
- **AND** this is verified by `kvm_two_disks_appear_in_the_order_given` (`apps/virtax-vmm/tests/boot.rs`) and `virtio::block::tests::writes_flushes_and_reads_back_through_the_image`

#### Scenario: Unsupported format
- **WHEN** a VM is started with a qcow2 or VDI image
- **THEN** the start fails before the guest runs, with an error naming the file and its format
- **AND** this is verified by `kvm_qcow2_image_is_refused_before_the_guest_runs` and `disk::tests::refuses_other_formats_by_their_headers` (qcow2, VDI, VMDK, VHDX, VHD)

### Requirement: Guest flushes are durable
With the default writeback cache mode, `virtax-vmm` SHALL complete a guest flush request only after the data written before it is durable on the host (`fdatasync` or the platform equivalent). Source: FR-EN05-007 (writeback mode only).

#### Scenario: Flush reaches the host disk
- **WHEN** the guest writes data and issues a flush, and the flush completes
- **THEN** the written data is in the image file even if `virtax-vmm` is killed immediately afterwards
- **AND** this is verified by `kvm_flushed_disk_writes_reach_the_image_even_if_the_vmm_is_killed`

### Requirement: Image locking
`virtax-vmm` SHALL hold an exclusive lock on every read-write image while the VM runs, and a shared lock on read-only images, so that no two VMs write the same image. Source: FR-EN05-012 (locking only; dirty-bit checks and fuzzing come with qcow2).

#### Scenario: Second writer refused
- **WHEN** a second VM is started with an image another running VM has open read-write
- **THEN** the second start fails before its guest runs, with an error saying the image is in use
- **AND** this is verified by `kvm_an_image_in_use_is_refused_until_its_vm_stops` and `disk::tests::a_read_write_image_is_locked_until_released`

#### Scenario: Lock released on exit
- **WHEN** the first VM powers off or is killed
- **THEN** the image can be attached to another VM
- **AND** this is verified by `kvm_an_image_in_use_is_refused_until_its_vm_stops` (SIGKILL) and `disk::tests::a_read_write_image_is_locked_until_released`
