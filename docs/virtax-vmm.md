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
  in-kernel local APIC, I/O APIC and PIT, and a 16550 UART on COM1.
- Hardware-reduced ACPI tables (`crates/virtax-vmm-core/src/x86/acpi.rs`): the MADT lists the CPUs and the I/O APIC;
  the FADT provides a sleep-control register for soft-off and a reset register, and declares that there is no
  PS/2 controller, VGA or CMOS clock; the DSDT holds `\_S5_` and describes COM1 (`PNP0501`, I/O 0x3F8, IRQ 4).
  On a hardware-reduced platform Linux only wires an ISA device's interrupt when the DSDT describes it.
- Built-in kernel parameters: `console=ttyS0 reboot=k panic=1 pci=off`. A guest `poweroff` (ACPI S5), reboot
  (keyboard-controller or ACPI reset) or triple fault ends the VM.

## Logs

- `<vm folder>/logs/serial.log`: the guest serial console, appended across boots.
- `<vm folder>/logs/virtax-vmm.log`: VMM diagnostics.

## Lifecycle on this engine

| Action | Behaviour |
|---|---|
| Start | Fails with `VTX-ENG-011` and a fix hint if there is no kernel, more than 254 vCPUs, or a missing file |
| Guest powers off or reboots | Machine becomes `powered_off`; queued pending changes are applied |
| VMM crash or triple fault | Machine becomes `aborted` |
| `virtaxctl vm stop --force` | Kills the VMM process; machine becomes `powered_off` |
| `virtaxctl vm stop` (ACPI) | Refused with `VTX-ENG-010`: a host-initiated shutdown needs a power-button event (ACPI GED), which is not implemented yet |

## Not yet

Host-initiated ACPI shutdown (power button), a guest reboot that restarts instead of stopping the VM, a CMOS RTC
(guests start their clock at the epoch until NTP or the kvm-clock sets it), virtio devices, firmware (UEFI)
boot, a display, pause and snapshots, machines outliving `virtaxd`, and arm64/HVF/WHP backends.

## Tests

`cargo xtask guest-kernel` fetches a pinned Ubuntu kernel package (SHA-256 checked) and builds a busybox
initramfs into `.cache/guest`. `cargo xtask kvm-test` then runs the `kvm_*` tests, which boot that guest
directly with `virtax-vmm` and through `virtaxd` + `virtaxctl`. The CI workflow runs them in a dedicated job.
