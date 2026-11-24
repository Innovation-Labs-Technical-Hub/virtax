# oxvmm

`oxvmm` is OxBox's Rust VMM. Each machine runs in its own `oxvmm` process; `oxboxd` supervises it through
`OxvmmEngine` (`crates/oxbox-engine/src/oxvmm.rs`).

## Host requirements

- Linux x86_64 with `/dev/kvm` readable and writable by the user (usually membership in the `kvm` group).
- The `oxvmm` binary next to `oxboxd`, or set with `--oxvmm-bin` / `OXBOX_OXVMM_BIN`. `oxboxd` logs why the
  engine is disabled when either check fails, and `oxctl version` lists the enabled engines.

## What a guest gets today

- Direct kernel boot: bzImage or ELF vmlinux, optional initramfs, and extra kernel parameters from the
  manifest's `[boot]` table (`oxctl vm set <vm> --kernel … --initrd … --cmdline …`). Relative paths resolve
  against the machine folder.
- One vCPU in 64-bit long mode, RAM from `memory.size_mib` (split around the 3–4 GiB MMIO hole), an in-kernel
  interrupt controller and PIT, and a 16550 UART on COM1.
- Built-in kernel parameters: `console=ttyS0 reboot=k panic=1 pci=off` plus i8042 options. A guest reboot
  (keyboard-controller reset) or triple fault ends the VM.

## Logs

- `<vm folder>/logs/serial.log`: the guest serial console, appended across boots.
- `<vm folder>/logs/oxvmm.log`: VMM diagnostics.

## Lifecycle on this engine

| Action | Behaviour |
|---|---|
| Start | Fails with `OXE-ENG-011` and a fix hint if there is no kernel, more than one vCPU, or a missing file |
| Guest powers off or reboots | Machine becomes `powered_off`; queued pending changes are applied |
| VMM crash or triple fault | Machine becomes `aborted` |
| `oxctl vm stop --force` | Kills the VMM process; machine becomes `powered_off` |
| `oxctl vm stop` (ACPI) | Refused with `OXE-ENG-010` until oxvmm provides ACPI tables |

## Not yet

SMP (MP/ACPI tables), ACPI shutdown, a CMOS RTC, virtio devices, firmware (UEFI) boot, a display, pause and
snapshots, machines outliving `oxboxd`, and arm64/HVF/WHP backends.

## Tests

`cargo xtask guest-kernel` fetches a pinned Ubuntu kernel package (SHA-256 checked) and builds a busybox
initramfs into `.cache/guest`. `cargo xtask kvm-test` then runs the `kvm_*` tests, which boot that guest
directly with `oxvmm` and through `oxboxd` + `oxctl`. The CI workflow runs them in a dedicated job.
