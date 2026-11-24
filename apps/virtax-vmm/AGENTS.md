# virtax-vmm

One VMM process per virtual machine (Linux x86_64, KVM). It boots a kernel and initramfs, writes the guest serial
console to `--serial`, and reports how the VM ended through its exit code.

## Commands

```sh
cargo run -p virtax-vmm -- --help           # --kernel --initrd --cmdline --memory-mib --cpus --serial --no-reboot
just boot 4 512                             # boot the test guest with 4 vCPUs and 512 MiB (needs `just guest`)
just kvm-test                               # all tests that boot real guests
```

## Layout

- `src/main.rs`: flags, the ready line, SIGTERM → power button, re-exec on reboot, exit codes.
- `tests/boot.rs`: `kvm_*` tests through the `Guest` helper, which kills the VM on drop.

## Process contract (shared with `VirtaxVmmEngine`)

- Prints `virtax-vmm: guest running` on stderr when the guest starts.
- Exit 0: guest powered off (or rebooted under `--no-reboot`); 3: triple fault; 1: VMM error.
- Guest reboot: re-executes itself with the same arguments and PID.
- SIGTERM presses the ACPI power button; SIGKILL stops at once.

## Gotchas

- The test guest's `/init` (built by `cargo xtask guest-kernel`) reacts to kernel parameters: `virtax.hold` stays
  up, `virtax.reboot` reboots, and by default it powers off.
- Signal the child's own PID in tests; `pkill -f` patterns also match the invoking shell.

Rules: `../../.principles/vmm.md`. Docs: `docs/virtax-vmm.md`.
