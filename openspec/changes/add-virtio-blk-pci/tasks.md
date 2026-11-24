# Tasks

## 1. PCI bus and ACPI description

- [x] 1.1 Add a PCI bus with type-0 configuration space, memory BAR allocation from the fixed window and an ECAM MMIO handler in `virtax-vmm-core`; verify with unit tests for config reads/writes, BAR sizing and ECAM address decoding
- [x] 1.2 Add the ACPI MCFG table and the DSDT host bridge (`PNP0A08`, bus range and BAR window `_CRS`); verify with table checksum and AML-encoding unit tests in `x86/acpi.rs` and `x86/aml.rs`
- [x] 1.3 Remove `pci=off` from the default kernel command line; verify every existing `kvm_*` test still passes with `cargo xtask kvm-test`
- [x] 1.4 Add a `kvm_` test that boots with no devices and checks the guest lists the host bridge under `/sys/bus/pci/devices`; verify it passes with `cargo xtask kvm-test`

## 2. MSI-X and the virtio transport

- [x] 2.1 Add a safe MSI routing API (GSI routes plus irqfd per vector) to `virtax-vmm-hv`, every `unsafe` block with a `SAFETY:` comment; verify `cargo xtask ci` passes its lint policy and clippy
- [x] 2.2 Add an MSI-X capability (table and PBA in a BAR) to the PCI device model; verify with unit tests for vector programming and masking
- [x] 2.3 Add the virtio 1.2 modern PCI transport (common, notify, ISR and device config capabilities; feature negotiation; device status) with notifications registered as ioeventfd; verify with unit tests driving a test device through feature negotiation and a queue kick

## 3. virtio-blk on raw images

- [x] 3.1 Add a pinned `virtio-queue` dependency and the virtio-blk device (capacity, read-only flag, `IN`, `OUT`, `FLUSH` and `GET_ID`) with one worker thread doing blocking `pread`/`pwrite`/`fdatasync` into guest memory; verify with unit tests on an in-memory queue against a temporary raw file
- [x] 3.2 Detect qcow2, VDI, VMDK, VHDX and VHD by their headers and refuse them; verify with unit tests on sample headers
- [x] 3.3 Lock images (exclusive for read-write, shared for read-only) for the process lifetime; verify with a test that a second open of a read-write image fails and succeeds after the first is dropped
- [x] 3.4 Add `virtax-vmm --disk <path>[,ro]` (repeatable); verify `virtax-vmm --help` documents it and an invalid value exits 1 with a clear message
- [x] 3.5 Extend the test guest's `/init` with a `virtax.disktest` mode that writes a known pattern to `/dev/vda`, flushes and powers off, and add `kvm_` tests for: pattern persisted in the host file, read-only disk refusing writes, two disks in order, a qcow2 image refused before boot, and a locked image refused; verify with `cargo xtask kvm-test`
- [x] 3.6 Document disks, the `--disk` flag and the locking behaviour in `docs/virtax-vmm.md`; verify the documented command boots the test guest with a disk

## 4. Engine integration

- [x] 4.1 Build `--disk` arguments from the manifest's `virtio` controllers in `VirtaxVmmEngine`, and refuse other buses, missing files and non-raw images with `VTX-ENG-011`; verify with engine unit tests for each refusal and for attachment order
- [x] 4.2 Add a `kvm_` end-to-end test that creates a machine through `virtaxctl`, adds a raw disk to its `machine.toml`, starts it in disk-test mode and checks the host file afterwards; verify with `cargo xtask kvm-test`

## 5. Integration checks

- [x] 5.1 Run `cargo xtask ci` and `cargo xtask kvm-test`; verify both pass locally and in the CI jobs, including the KVM job
- [x] 5.2 Mark the implemented scenarios in the delta specs against their tests; verify `openspec validate add-virtio-blk-pci --strict` passes

## Workflow follow-up

- Archive the change after review with `openspec archive add-virtio-blk-pci`, which merges the delta requirements into the main specs.
- Verify `openspec validate --all --strict` passes after the archive.
