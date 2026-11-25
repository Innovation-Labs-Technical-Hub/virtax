# Block backends Specification

## Purpose
raw/qcow2/VDI, io_uring (Linux) · IOCP (Windows) · kqueue/dispatch_io (macOS), zero-copy, explicit cache modes, discard → hole punch. Module `EN-05` (Block backends; Vietnamese: "Block backend") from BRD-06, phase P2, priority Must. Owner crates: virtax-vmm-block (planned; today `crates/virtax-vmm-core/src/disk.rs` and `src/virtio/block.rs`). Depends on: `engine/virtax-vmm-core-and-hypervisor-backends`. VirtualBox reference: `src/VBox/Storage, Devices/Storage/DrvVD.cpp, VMM PDMAsyncCompletion*`. Implementation status (2026-11-25): partial — raw images with writeback flush (`fdatasync`) and image locking, blocking I/O on one worker thread per disk; no other formats, async backends, cache modes or discard.

## Requirements

### Requirement: virtax-vmm-block format matrix (FR-EN05-001)
Virtax SHALL provide "virtax-vmm-block format matrix" as specified by FR-EN05-001 (BRD-06, priority Must, phase P2; Vietnamese: "Ma trận định dạng của virtax-vmm-block").

#### Scenario: Specified behaviour of FR-EN05-001
- **WHEN** the behaviour FR-EN05-001 covers is exercised
- **THEN** it behaves as BRD-06 specifies: virtax-vmm reads/writes raw, qcow2 (v3, 64 KiB default clusters, backing files) and VDI (to run VMs moved from legacy); VMDK (monolithic sparse/flat, streamOptimized), VHD and VHDX are read-only. Opening a read-only format for writing automatically creates a qcow2 overlay or asks for Convert.
- **AND** the VirtualBox reference for the current behaviour is `Storage/QCOW.cpp:165-183 (no v3 features); Storage/VHDX.cpp:2007-2013; Storage/VDI.cpp`

#### Scenario: Acceptance for FR-EN05-001
- **WHEN** the FR-EN05-001 acceptance check is run
- **THEN** Compatibility suite: images created by qemu-img (qcow2/raw/vmdk/vhdx) and by VBox (VDI/VMDK/VHD) all boot; after writes, `qemu-img check` on qcow2 reports 0 errors.

### Requirement: io_uring backend on Linux (FR-EN05-002)
Virtax SHALL provide "io_uring backend on Linux" as specified by FR-EN05-002 (BRD-06, priority Must, phase P2; Vietnamese: "Backend io_uring trên Linux").

#### Scenario: Specified behaviour of FR-EN05-002
- **WHEN** the behaviour FR-EN05-002 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Each device queue has its own io_uring with registered files and fixed buffers pointing into guest RAM; O_DIRECT by default, buffered when the cache mode is writeback; flush → IORING_OP_FSYNC (DATASYNC); optional SQPOLL for NVMe-class disks. Automatic fallback to a preadv/pwritev thread pool when io_uring is disabled (sysctl, seccomp).
- **AND** the VirtualBox reference for the current behaviour is `Runtime/r3/linux/fileaio-linux.cpp:242-299; Runtime/r3/linux/ioqueue-iouringfile-provider.cpp (unused)`

#### Scenario: Acceptance for FR-EN05-002
- **WHEN** the FR-EN05-002 acceptance check is run
- **THEN** On an Ubuntu 24.04 host, guest fio 4k randread QD32 ≥ 1.5× VBox 7.2 on the same disk; with `kernel.io_uring_disabled=2` the VM still runs and the log states the fallback backend.

### Requirement: IOCP/IoRing backend on Windows (FR-EN05-003)
Virtax SHALL provide "IOCP/IoRing backend on Windows" as specified by FR-EN05-003 (BRD-06, priority Must, phase P2; Vietnamese: "Backend IOCP/IoRing trên Windows").

#### Scenario: Specified behaviour of FR-EN05-003
- **WHEN** the behaviour FR-EN05-003 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Uses overlapped ReadFile/WriteFile (scatter/gather when page-aligned) with FILE_FLAG_NO_BUFFERING for mode none, one completion port and worker per queue; uses IoRing on Windows 11 when available. Flush → FlushFileBuffers. No global lock on the submission path.
- **AND** the VirtualBox reference for the current behaviour is `Runtime/r3/win/fileaio-win.cpp:291-345,381-387; Runtime/r3/win/fileio-win.cpp:307-312,797`

#### Scenario: Acceptance for FR-EN05-003
- **WHEN** the FR-EN05-003 acceptance check is run
- **THEN** On Windows 11 x64 and arm64 hosts: guest fio 4k randread QD32 ≥ 1.5× VBox 7.2; perf counters show ≥ 2 cores handling completions for a 4-vCPU VM.

### Requirement: macOS backend without POSIX AIO (FR-EN05-004)
Virtax SHALL provide "macOS backend without POSIX AIO" as specified by FR-EN05-004 (BRD-06, priority Must, phase P2; Vietnamese: "Backend macOS không dùng POSIX AIO").

#### Scenario: Specified behaviour of FR-EN05-004
- **WHEN** the behaviour FR-EN05-004 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Uses dispatch_io (or a preadv/pwritev thread pool if it benchmarks better) with F_NOCACHE for mode none; completions are signalled through kqueue/dispatch with no aio_suspend polling and no dependency on kern.aioprocmax. Flush follows the cache mode: F_FULLFSYNC (default) or F_BARRIERFSYNC when the user picks 'ordering only'.
- **AND** the VirtualBox reference for the current behaviour is `Runtime/r3/posix/fileaio-posix.cpp:315-330,933-940; Runtime/r3/posix/fileio-posix.cpp:861-863`

#### Scenario: Acceptance for FR-EN05-004
- **WHEN** the FR-EN05-004 acceptance check is run
- **THEN** On an M-series Mac with macOS 15, fio 4k randread QD64 is not capped at ~16-90 in-flight requests; each guest flush issues exactly one F_FULLFSYNC (checked with fs_usage).

### Requirement: Zero-copy from guest memory (FR-EN05-005)
Virtax SHALL provide "Zero-copy from guest memory" as specified by FR-EN05-005 (BRD-06, priority Must, phase P2; Vietnamese: "Zero-copy từ guest memory").

#### Scenario: Specified behaviour of FR-EN05-005
- **WHEN** the behaviour FR-EN05-005 covers is exercised
- **THEN** it behaves as BRD-06 specifies: virtio-blk descriptor chains and NVMe PRP/SGL become iovecs pointing straight into guest memory (`vm-memory` crate) and go directly to io_uring/IOCP/dispatch_io. Bounce buffers are only used for requests misaligned for O_DIRECT or for writes to encrypted disks; there is no fixed 5 MiB in-flight cap.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Storage/DrvVD.cpp:2693-2724 (#if 0 zero-copy), 5011`

#### Scenario: Acceptance for FR-EN05-005
- **WHEN** the FR-EN05-005 acceptance check is run
- **THEN** Measured with perf/ETW: 0 data memcpy on the aligned read/write path; fio 1 MiB seq QD32 reaches ≥ 90% of the host's own throughput on that file.

### Requirement: One queue per vCPU (FR-EN05-006)
Virtax SHALL provide "One queue per vCPU" as specified by FR-EN05-006 (BRD-06, priority Must, phase P2; Vietnamese: "Một queue cho mỗi vCPU").

#### Scenario: Specified behaviour of FR-EN05-006
- **WHEN** the behaviour FR-EN05-006 covers is exercised
- **THEN** it behaves as BRD-06 specifies: virtio-blk has num_queues = vCPU count (max 16), NVMe has one SQ/CQ pair per vCPU, each queue has its own worker and MSI-X vector; kicks go through ioeventfd and completions through irqfd on KVM, with EVENT_IDX to cut exits on WHP/HVF. Two disks never share one thread.
- **AND** the VirtualBox reference for the current behaviour is `VMM/VMMR3/PDMR3AsyncCompletionFile.cpp:1052-1068; PDMR3AsyncCompletionFileNormal.cpp:235-300; Devices/Storage/DevNVMe.cpp:94`

#### Scenario: Acceptance for FR-EN05-006
- **WHEN** the FR-EN05-006 acceptance check is run
- **THEN** An 8-vCPU VM running 8 fio jobs scales nearly linearly up to 8 jobs (≥ 6× one job); on KVM userspace exits per I/O ≈ 0.

### Requirement: Cache-mode and flush semantics (FR-EN05-007)
Virtax SHALL provide "Cache-mode and flush semantics" as specified by FR-EN05-007 (BRD-06, priority Must, phase P2; Vietnamese: "Ngữ nghĩa cache mode và flush").

#### Scenario: Specified behaviour of FR-EN05-007
- **WHEN** the behaviour FR-EN05-007 covers is exercised
- **THEN** it behaves as BRD-06 specifies: writeback: host page cache, FLUSH → fdatasync/FlushFileBuffers/F_FULLFSYNC; writethrough: O_DSYNC/FILE_FLAG_WRITE_THROUGH, F_FULLFSYNC after each write batch on macOS; none: O_DIRECT/NO_BUFFERING/F_NOCACHE with flushes still honoured; unsafe: flushes ignored. Devices advertise a volatile write cache (VIRTIO_BLK_F_FLUSH, NVMe VWC) according to the mode.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Storage/DrvVD.cpp:4984,4993; VMM/VMMR3/PDMR3BlkCache.cpp:1074-1086; Runtime/r3/posix/fileio-posix.cpp:861-863`

#### Scenario: Acceptance for FR-EN05-007
- **WHEN** the FR-EN05-007 acceptance check is run
- **THEN** 1,000-run crash test (kill -9 virtax-vmm + power-loss simulation via dm-flakey/nested VM): 0 qcow2 metadata errors and 0 loss of guest-fsynced data in every mode except unsafe.

### Requirement: Discard/write-zeroes → host hole punching (FR-EN05-008)
Virtax SHALL provide "Discard/write-zeroes → host hole punching" as specified by FR-EN05-008 (BRD-06, priority Must, phase P2; Vietnamese: "Discard/write-zeroes → hole punching trên host").

#### Scenario: Specified behaviour of FR-EN05-008
- **WHEN** the behaviour FR-EN05-008 covers is exercised
- **THEN** it behaves as BRD-06 specifies: virtio-blk DISCARD/WRITE_ZEROES and NVMe Dataset Management are translated to FALLOC_FL_PUNCH_HOLE/ZERO_RANGE (Linux), FSCTL_SET_SPARSE + FSCTL_SET_ZERO_DATA (Windows) and F_PUNCHHOLE (macOS); for qcow2, clusters and refcounts are freed and the matching file range punched. No more VDI-style block moving.
- **AND** the VirtualBox reference for the current behaviour is `Storage/VDI.cpp:1221-1290 (block moving); Runtime/r3/linux/RTFileSetAllocationSize-linux.cpp:60-86 (no punch-hole)`

#### Scenario: Acceptance for FR-EN05-008
- **WHEN** the FR-EN05-008 acceptance check is run
- **THEN** Writing 10 GB then deleting + `fstrim -a` in the guest shrinks the host file by ≥ 9 GB within ≤ 60 s on ext4/xfs, NTFS and APFS.

### Requirement: qcow2 overlays and background merge (FR-EN05-009)
Virtax SHALL provide "qcow2 overlays and background merge" as specified by FR-EN05-009 (BRD-06, priority Must, phase P2; Vietnamese: "Overlay qcow2 và merge nền").

#### Scenario: Specified behaviour of FR-EN05-009
- **WHEN** the behaviour FR-EN05-009 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Snapshots and linked clones use qcow2 overlays (backing chain up to 16). The allocation map is cached so chain reads do not repeat metadata I/O. Deleting a snapshot = a background block-commit while the VM runs, with %, ETA and cancel, without forcing sync I/O or a disk-wide lock. Full clones use reflinks when the filesystem supports them (FICLONE on btrfs/XFS, clonefile() on APFS, block clone on ReFS), otherwise a hole-skipping copy.
- **AND** the VirtualBox reference for the current behaviour is `Storage/VD.cpp:1613-1642,2697-2720,6755 (VDMerge); Devices/Storage/DrvVD.cpp:5049-5053`

#### Scenario: Acceptance for FR-EN05-009
- **WHEN** the FR-EN05-009 acceptance check is run
- **THEN** With an 8-layer chain, fio 4k randread drops ≤ 10% vs one layer; merging 20 GB while the VM runs fio cuts guest IOPS by ≤ 30%, cancelling leaves a consistent image; cloning 64 GB on APFS takes ≤ 1 s.

### Requirement: Live-adjustable I/O throttling (FR-EN05-010)
Virtax SHALL provide "Live-adjustable I/O throttling" as specified by FR-EN05-010 (BRD-06, priority Should, phase P2; Vietnamese: "Giới hạn I/O (throttling) đổi được khi chạy").

#### Scenario: Specified behaviour of FR-EN05-010
- **WHEN** the behaviour FR-EN05-010 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Each disk has IOPS and MB/s limits (token bucket with burst), changeable live through UpdateMachine (Live badge). Throttling lives in virtax-vmm-block and does not force the host-cache path like VBox bandwidth groups.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Storage/DrvVD.cpp:5043-5047 (bandwidth groups force host cache); Devices/Network/DrvNetShaper.cpp`

#### Scenario: Acceptance for FR-EN05-010
- **WHEN** the FR-EN05-010 acceptance check is run
- **THEN** Setting 2,000 IOPS on a VM running fio yields 2,000 ± 5% measured IOPS within ≤ 2 s; removing the limit restores the previous rate.

### Requirement: Batched disk encryption off the submit thread (FR-EN05-011)
Virtax SHALL provide "Batched disk encryption off the submit thread" as specified by FR-EN05-011 (BRD-06, priority Should, phase P2; Vietnamese: "Mã hoá disk theo batch, ngoài thread submit").

#### Scenario: Specified behaviour of FR-EN05-011
- **WHEN** the behaviour FR-EN05-011 covers is exercised
- **THEN** it behaves as BRD-06 specifies: AES-XTS-256 processes whole requests (AES-NI / ARMv8 Crypto Extensions) on the queue worker: reads decrypt in place in guest memory after the I/O completes, writes encrypt into a separate buffer. virtax-vmm reads VDFilterCrypt-encrypted VDIs so legacy VMs move without decrypting first.
- **AND** the VirtualBox reference for the current behaviour is `Storage/VDFilterCrypt.cpp:379-440,738-763 (per-sector cipher + memcpy); Devices/Storage/IOBufMgmt.cpp:367-368`

#### Scenario: Acceptance for FR-EN05-011
- **WHEN** the FR-EN05-011 acceptance check is run
- **THEN** An encrypted disk reaches ≥ 80% of the 4k randread IOPS of a plain disk on the same host; an encrypted VDI from VBox 7.2 boots on virtax-vmm with the same password.

### Requirement: Image locking, consistency checks and fuzzing (FR-EN05-012)
Virtax SHALL provide "Image locking, consistency checks and fuzzing" as specified by FR-EN05-012 (BRD-06, priority Must, phase P2; Vietnamese: "Khoá image, kiểm tra nhất quán và fuzz").

#### Scenario: Specified behaviour of FR-EN05-012
- **WHEN** the behaviour FR-EN05-012 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Images are locked on open (OFD locks / LockFileEx / flock) so two VMs never write the same file, except Shareable ones. qcow2 keeps a dirty bit: reopening after a crash runs check/repair before allowing writes. qcow2/VDI/VMDK/VHD(X) parsers run cargo-fuzz continuously in CI.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/MediumImpl.cpp (lockRead/lockWrite tokens); Storage/VMDK.cpp, Storage/VHDX.cpp (C parsers)`

#### Scenario: Acceptance for FR-EN05-012
- **WHEN** the FR-EN05-012 acceptance check is run
- **THEN** Starting a second VM on the same disk gives VTX-STO-050 naming the VM holding the lock; 24 h of fuzzing per parser: 0 panics, 0 out-of-bounds reads.

### Requirement: Per-queue block metrics, only when subscribed (FR-EN05-013)
Virtax SHALL provide "Per-queue block metrics, only when subscribed" as specified by FR-EN05-013 (BRD-06, priority Should, phase P2; Vietnamese: "Metrics block theo queue, chỉ khi có subscriber").

#### Scenario: Specified behaviour of FR-EN05-013
- **WHEN** the behaviour FR-EN05-013 covers is exercised
- **THEN** it behaves as BRD-06 specifies: virtax-vmm-block counts IOPS, bytes, queue depth, latency histograms (p50/p99) and flush counts per queue in shared memory; virtaxd only reads and streams them through Metrics.Watch while someone is watching. Backends create no wakeups when there is no I/O.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/PerformanceImpl.cpp:201 (1 Hz sampler always on); VMM STAM counters`

#### Scenario: Acceptance for FR-EN05-013
- **WHEN** the FR-EN05-013 acceptance check is run
- **THEN** An idle VM with no disk I/O: 0 wakeups/s from block threads (powertop/ETW); opening the Performance tab shows p99 latency within ≤ 1 s.

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
