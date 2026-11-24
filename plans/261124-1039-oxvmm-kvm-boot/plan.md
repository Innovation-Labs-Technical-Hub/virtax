# oxvmm — first boot on KVM

Status: done (2026-11-24): all four acceptance criteria verified (`cargo xtask ci`, `cargo xtask kvm-test`) · Branch `feat/oxvmm-kvm` (stacked on `feat/control-plane-foundation`)

## Outcome

`oxvmm`, a Rust VMM process (one per VM), boots a Linux kernel on KVM x86_64 with an initramfs, shows the
guest serial console in the VM's `logs/serial.log`, and exits when the guest powers off or reboots.
`oxboxd` drives it through a new `OxvmmEngine` behind the existing `Engine` trait, so `oxctl` can create,
start and force-stop a real guest.

## Decisions

- **Base crates:** rust-vmm (`kvm-ioctls`, `vm-memory`, `linux-loader`, `vm-superio`) instead of OpenVMM.
  OpenVMM crates are not published and are internal to its monorepo; rust-vmm is the plan's own Linux fallback
  (D4). Everything hypervisor-specific stays in `oxvmm-hv`, so HVF/WHP backends can follow.
- **Boot:** direct kernel boot (bzImage or ELF vmlinux + initramfs + cmdline) through a new optional `[boot]`
  manifest table. Firmware (OVMF) boot comes later.
- **Unsafe:** only `oxvmm-hv` may use `unsafe`, and every block needs a `SAFETY:` comment (enforced by clippy).

## Scope

| Item | In | Later |
|---|---|---|
| Guest memory with the 32-bit MMIO gap, e820 map | ✓ | |
| x86_64 long-mode boot: GDT, identity page tables, boot_params, CPUID, MSRs, LAPIC LINT | ✓ | |
| In-kernel irqchip + PIT, 16550 serial on COM1 (irqfd), i8042 reset | ✓ | |
| One vCPU | ✓ | SMP (MP table / ACPI MADT) |
| `OxvmmEngine`: start, force power-off, guest-initiated exit → powered_off, crash → aborted | ✓ | ACPI shutdown, pause, snapshots |
| virtio devices, firmware, display, arm64/HVF/WHP | | P2 roadmap |

## Acceptance criteria

1. `cargo xtask guest-kernel` fetches a pinned, sha256-verified Ubuntu kernel and builds a busybox initramfs.
2. `oxvmm --kernel … --initrd …` boots to the initramfs `/init`, whose marker line appears on serial, and the
   process exits 0 when the guest reboots (`reboot -f`).
3. `oxctl vm create --engine oxvmm --kernel … --initrd … --cpus 1` + `oxctl vm start` boots the guest; the
   machine returns to `powered_off` by itself after the guest reboots; `oxctl vm stop --force` powers off a
   running guest.
4. `cargo xtask ci` stays green on machines without KVM; the KVM tests run via `cargo xtask kvm-test`.

## Result notes

- The Ubuntu 7.0 test kernel reaches `/init` in about 2.9 s of guest time; `oxvmm` exits 0 on the guest's reboot.
- Boot-log gaps to close next: no ACPI or MP tables (single vCPU, no ACPI shutdown), no CMOS RTC, and the
  i8042 probe fails, costing about 0.5 s of boot time.
- Machines do not outlive `oxboxd` yet; their processes are killed when it exits.
