# virtax-vmm

`virtax-vmm` is Virtax's Rust VMM. Each machine runs in its own `virtax-vmm` process; `virtaxd` supervises it through
`VirtaxVmmEngine` (`crates/virtax-engine/src/vmm.rs`).

## Host requirements

- Linux x86_64 with `/dev/kvm` readable and writable by the user (usually membership in the `kvm` group).
- The `virtax-vmm` binary next to `virtaxd`, or set with `--vmm-bin` / `VIRTAX_VMM_BIN`. `virtaxd` logs why the
  engine is disabled when either check fails, and `virtaxctl version` lists the enabled engines.

## What a guest gets today

- Direct kernel boot: bzImage or ELF vmlinux, optional initramfs, and extra kernel parameters from the
  manifest's `[boot]` table (`virtaxctl vm set <vm> --kernel … --initrd … --cmdline …`). Relative paths resolve
  against the machine folder.
- Up to 254 vCPUs (one host thread each), RAM from `memory.size_mib` (split around the 3–4 GiB MMIO hole), an
  in-kernel local APIC, I/O APIC and PIT, a 16550 UART on COM1, and a CMOS real-time clock (ports 0x70/0x71)
  that reports the host's UTC time in BCD, which Linux on x86 always expects.
- Hardware-reduced ACPI tables (`crates/virtax-vmm-core/src/x86/acpi.rs`): the MADT lists the CPUs and the I/O APIC;
  the FADT provides a sleep-control register for soft-off and a reset register, and declares that there is no
  PS/2 controller or VGA. The DSDT holds `\_S5_` and describes COM1 (`PNP0501`, I/O 0x3F8, IRQ 4), the RTC
  (`PNP0B00`), a power button (`PNP0C0C`) and the Generic Event Device (`ACPI0013`, GSI 5) that raises it. On a
  hardware-reduced platform Linux only wires an ISA device's interrupt when the DSDT describes it.
- A PCI Express root complex (`crates/virtax-vmm-core/src/pci/`): one segment, bus 0, ECAM at `0xFEB00000`
  described by the MCFG table and reserved in the e820 map and by a `PNP0C02` device; the DSDT's `PCI0` host
  bridge (`PNP0A08`) forwards the BAR window `0xC0000000–0xFDFFFFFF`. The host bridge sits at 00:00.0. Devices
  interrupt through MSI-X only (no `_PRT`, no INTx).
- Disks: see below.
- Built-in kernel parameters: `console=ttyS0 reboot=k panic=1`.

## Disks

Each `--disk` becomes a virtio 1.2 modern PCI block device (`1af4:1042`) with one request queue, in the order
given: the first is `/dev/vda` in a Linux guest, the second `/dev/vdb`, and so on.

```sh
truncate -s 64M disk.img
virtax-vmm --kernel .cache/guest/vmlinuz --initrd .cache/guest/initramfs.cpio \
    --disk disk.img --disk data.img,ro --cmdline virtax.disktest
```

- Raw images only. `virtax-vmm` reads each image's header before the guest runs and refuses qcow2, VDI, VMDK,
  VHDX and VHD, naming the file and the format (convert with `qemu-img convert -O raw`).
- `<path>,ro` attaches read-only: the guest sees a read-only device and writes fail. `<path>` and `<path>,rw`
  attach read-write; any other comma is part of the path. An empty path exits with code 1.
- Writeback caching: a guest flush completes only after `fdatasync` on the image.
- Locking: `virtax-vmm` holds an exclusive lock on each read-write image and a shared lock on each read-only one
  for as long as it runs (`flock`, released by the OS when the process ends, `SIGKILL` included). A second VM
  that wants to write an image in use, or to read one being written, fails to start with "in use by another VM".
- Requests run on one worker thread per disk with blocking reads and writes straight into guest memory; a guest
  kick reaches it through an ioeventfd without a vCPU exit.

## Power and reboot

- A guest `poweroff` (ACPI S5) ends the VM; `virtax-vmm` exits 0.
- A guest reboot (keyboard-controller or ACPI reset, including `panic=1`) restarts the VM: `virtax-vmm`
  re-executes itself with the same arguments, keeping its process ID. `--no-reboot` makes it exit instead.
- SIGTERM presses the ACPI power button. Linux reports it as a power key; the guest shuts down if something
  handles it (systemd-logind, or acpid in minimal systems). SIGKILL powers off at once.
- A triple fault ends the VM with exit code 3.

## Logs

- `<vm folder>/logs/serial.log`: the guest serial console, appended across boots.
- `<vm folder>/logs/virtax-vmm.log`: VMM diagnostics.

## Lifecycle on this engine

| Action | Behaviour |
|---|---|
| Start | Fails with `VTX-ENG-011` and a fix hint if there is no kernel, more than 254 vCPUs, or a missing file |
| Guest powers off | Machine becomes `powered_off`; queued pending changes are applied |
| Guest reboots | The VM restarts; the machine stays `running` |
| VMM crash or triple fault | Machine becomes `aborted` |
| `virtaxctl vm stop --force` | Kills the VMM process; machine becomes `powered_off` |
| `virtaxctl vm stop` | Presses the ACPI power button and waits up to 2 minutes for the guest to power off; if it does not, the operation fails with `VTX-ENG-012` and the machine keeps running |

## Not yet

Other disk formats (qcow2, VDI, …), multi-queue and io_uring for disks, discard, NVMe/AHCI/CD media, network
and other virtio devices, PCI hot-plug, firmware (UEFI) boot, a display, RTC alarms and CMOS RAM, pause and snapshots,
machines outliving `virtaxd`, and arm64/HVF/WHP backends.

## Tests

`cargo xtask guest-kernel` fetches a pinned Ubuntu kernel package (SHA-256 checked) and builds a busybox
initramfs into `.cache/guest`. `cargo xtask kvm-test` then runs the `kvm_*` tests, which boot that guest
directly with `virtax-vmm` and through `virtaxd` + `virtaxctl`. The test guest's `virtax.disktest` parameter
writes `VIRTAX-DISK-PATTERN-vda` (and `-vdb`) to the start of each disk with a flush, which the disk tests then
look for in the image files. The CI workflow runs them in a dedicated job.
