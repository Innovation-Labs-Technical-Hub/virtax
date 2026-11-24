# virtax-vmm — SMP and ACPI

Status: done (2026-11-24) · Branch `feat/control-plane-and-virtax-vmm` (same PR as the foundation)

## Outcome

Guests boot with more than one vCPU and power themselves off through ACPI; `virtax-vmm` reports a guest power-off
separately from a reboot.

## What was built

- `x86/acpi.rs`: RSDP, XSDT, hardware-reduced FADT (sleep control/status and reset registers on I/O ports
  0x600–0x602; no PS/2, VGA or CMOS RTC), DSDT, MADT. `x86/aml.rs`: the AML subset the DSDT needs.
- One thread per vCPU; APs are started by the guest through INIT/SIPI handled by the in-kernel LAPIC.
- `VmExit::PowerOff` (exit 0, "guest powered off"); the engine's vCPU limit is 254.

## Findings

- Hardware-reduced ACPI removes the legacy PIC, so Linux preallocates no ISA IRQs; the built-in 8250 port's
  "IRQ 4" then maps to nothing and userspace console output is lost. Describing COM1 in the DSDT (`PNP0501`)
  fixes it; a MADT interrupt source override alone does not, and was removed.
- Telling the guest there is no PS/2 controller removed a failing i8042 probe: boot to `/init` dropped from
  about 2.9 s to about 0.2 s.

## Acceptance (verified)

1. `virtax-vmm --cpus 4 --cmdline virtax.poweroff`: the guest reports 4 CPUs, powers down through ACPI, exit 0.
2. The virtaxctl KVM e2e test runs a 2-vCPU machine.
3. `cargo xtask ci` and `cargo xtask kvm-test` are green.

## Next

Host-initiated ACPI shutdown via a Generic Event Device (power button), guest reboot that restarts the VM,
CMOS RTC, then virtio-blk and virtio-net over virtio-mmio.
