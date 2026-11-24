# virtax-vmm — guest lifecycle: power button, reboot, clock

Status: done (2026-11-24): all five acceptance criteria verified (`cargo xtask ci`, `cargo xtask kvm-test`) · Branch `feat/virtax-vmm-lifecycle`, stacked on `feat/virtax-control-plane-and-vmm` (PR #1)

## Outcome

`virtaxctl vm stop` (without `--force`) asks the guest to shut down through an ACPI power button and the machine
reaches `powered_off` once the guest complies; a guest reboot restarts the VM instead of ending it; guests read
the correct date and time at boot.

## Design

- **Power button:** hardware-reduced ACPI signals events through a Generic Event Device (`ACPI0013`). The DSDT
  gets `GED0` (an edge interrupt on GSI 5 whose `_EVT` notifies `\_SB.PWRB`) and `PWRB` (`PNP0C0C`). Linux's
  button driver turns the notification into a power-key input event that systemd-logind or acpid acts on.
  `virtax-vmm` presses the button when it receives SIGTERM; SIGKILL stays a hard power-off.
- **Engine:** `stop` with ACPI sends SIGTERM and waits up to 2 minutes for the guest to power off; on timeout
  the operation fails with `VTX-ENG-012` and the machine keeps running.
- **Reboot:** on a guest reset `virtax-vmm` re-executes itself with the same arguments (same PID, so the engine
  sees one continuous run); `--no-reboot` restores exit-on-reset.
- **Clock:** an MC146818 CMOS RTC on ports 0x70/0x71 in binary 24-hour mode with the host's UTC time,
  described in the DSDT as `PNP0B00`; the FADT stops declaring "no CMOS RTC".
- **Test guest:** ends with `poweroff` by default; `virtax.reboot` reboots, `virtax.hold` stays up, and it runs
  `acpid` so the power button powers it off.

## Acceptance criteria

1. KVM: SIGTERM to `virtax-vmm` running the test guest makes it power off through ACPI (exit 0).
2. KVM: with `virtax.reboot` the guest boots at least twice in one `virtax-vmm` process.
3. KVM: the guest's `date` shows the host's current year.
4. KVM e2e: `virtaxctl vm stop` (no `--force`) brings a running machine to `powered_off`.
5. `cargo xtask ci` and `cargo xtask kvm-test` green.

## Findings

- Linux on x86 always decodes the CMOS clock as BCD (`RTC_ALWAYS_BCD`), whatever status register B says. The
  first version served binary values and the guest booted into 2020-11-18 instead of 2026-11-24.
- busybox `acpid` daemonizes with its output discarded, so the test guest's power-button action writes its
  marker to `/dev/console`.
- `VTX-ENG-010` ("engine cannot do that yet") lost its only use and was removed from the catalogue.
