# virtax-vmm

`virtax-vmm` is Virtax's Rust VMM. Each machine runs in its own `virtax-vmm` process; `virtaxd` supervises it through
`VirtaxVmmEngine` (`crates/virtax-engine/src/virtax-vmm.rs`).

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
- Built-in kernel parameters: `console=ttyS0 reboot=k panic=1 pci=off`.

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

Virtio devices (disk, network), firmware (UEFI) boot, a display, RTC alarms and CMOS RAM, pause and snapshots,
machines outliving `virtaxd`, and arm64/HVF/WHP backends.

## Tests

`cargo xtask guest-kernel` fetches a pinned Ubuntu kernel package (SHA-256 checked) and builds a busybox
initramfs into `.cache/guest`. `cargo xtask kvm-test` then runs the `kvm_*` tests, which boot that guest
directly with `virtax-vmm` and through `virtaxd` + `virtaxctl`. The CI workflow runs them in a dedicated job.
