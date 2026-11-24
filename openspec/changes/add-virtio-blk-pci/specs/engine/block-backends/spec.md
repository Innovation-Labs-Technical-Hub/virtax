# Spec Delta

## ADDED Requirements

### Requirement: Raw disk images
`virtax-vmm` SHALL read and write raw disk images and SHALL refuse any other image format before the guest starts, naming the file and the format it found. Source: FR-EN05-001 (raw only; qcow2, VDI, VMDK, VHD and VHDX come later).

#### Scenario: Data persists across a power-off
- **WHEN** the guest writes a known pattern to its disk, flushes it and powers off
- **THEN** the image file on the host contains that pattern

#### Scenario: Unsupported format
- **WHEN** a VM is started with a qcow2 or VDI image
- **THEN** the start fails before the guest runs, with an error naming the file and its format

### Requirement: Guest flushes are durable
With the default writeback cache mode, `virtax-vmm` SHALL complete a guest flush request only after the data written before it is durable on the host (`fdatasync` or the platform equivalent). Source: FR-EN05-007 (writeback mode only).

#### Scenario: Flush reaches the host disk
- **WHEN** the guest writes data and issues a flush, and the flush completes
- **THEN** the written data is in the image file even if `virtax-vmm` is killed immediately afterwards

### Requirement: Image locking
`virtax-vmm` SHALL hold an exclusive lock on every read-write image while the VM runs, and a shared lock on read-only images, so that no two VMs write the same image. Source: FR-EN05-012 (locking only; dirty-bit checks and fuzzing come with qcow2).

#### Scenario: Second writer refused
- **WHEN** a second VM is started with an image another running VM has open read-write
- **THEN** the second start fails before its guest runs, with an error saying the image is in use

#### Scenario: Lock released on exit
- **WHEN** the first VM powers off or is killed
- **THEN** the image can be attached to another VM
