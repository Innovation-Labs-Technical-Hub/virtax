# virtax-vmm-core

The VMM's safe-Rust core for Linux x86_64: guest memory layout, direct kernel boot, ACPI tables, port-I/O
devices, the PCI bus with virtio-blk disks, and the vCPU run loop. Hypervisor calls go through `virtax-vmm-hv`.

## Commands

```sh
cargo test -p virtax-vmm-core
just kvm-test            # boots real guests; needs /dev/kvm and `just guest`
```

## Layout

- `src/vmm.rs`: `VmConfig`, `run` (one thread per vCPU), `VmExit`, `VmControl` (power button).
- `src/layout.rs`: guest-physical memory map and the MMIO hole.
- `src/x86/boot.rs`: bzImage/ELF loading, initramfs, command line, `boot_params`, e820.
- `src/x86/cpu.rs`: GDT, page tables, long-mode registers, CPUID, MSRs, LAPIC.
- `src/x86/acpi.rs`, `src/x86/aml.rs`: RSDP/XSDT/FADT/DSDT/MADT and the AML they need.
- `src/devices.rs`: port-I/O bus — COM1 (16550), CMOS RTC, i8042 reset, ACPI sleep/reset registers.
- `src/rtc.rs`: MC146818 clock serving host UTC time in BCD.
- `src/pci/`: the PCI bus (ECAM, configuration space, BAR assignment, host bridge), MSI-X, and the
  `Platform` trait devices use for MSI routes, irqfds and ioeventfds.
- `src/virtio/`: the virtio 1.2 modern PCI transport and the virtio-blk device with its worker thread.
- `src/disk.rs`: raw images — format detection by header and image locking.

## Conventions

- No `unsafe` here; if you need it, add a safe wrapper to `virtax-vmm-hv`.
- Every ACPI table and AML helper has a unit test with exact bytes or checksums.
- Guest-visible changes need a `kvm_*` test in `apps/virtax-vmm/tests/boot.rs`.

## Gotchas

- Hardware-reduced ACPI has no legacy PIC: a device's interrupt only works if the DSDT describes the device.
- Linux on x86 reads the CMOS clock as BCD regardless of status register B.
- `run` returns as soon as one vCPU stops the VM; the other vCPU threads end when the process exits.
- KVM replaces the whole interrupt routing table on each update: `Vm::set_msi_route` re-sends the default
  irqchip routes with every MSI route, so never call `set_gsi_routing` elsewhere.
- Linux uses ECAM only when it is reserved (e820 and `PNP0C02`); moving ECAM means moving both reservations.
- Every module here is gated on `target_os = "linux"` and `target_arch = "x86_64"`; keep new modules gated too.

Rules: `../../.principles/vmm.md`. Spec: `openspec/specs/engine/device-model-and-firmware/`,
`openspec/specs/engine/virtax-vmm-core-and-hypervisor-backends/`. Docs: `docs/virtax-vmm.md`.
