# Design

## Context

See proposal.md for motivation. Today `virtax-vmm` has only port-I/O devices (COM1, CMOS RTC, i8042 reset,
ACPI registers), a hardware-reduced ACPI platform with a GED power button, and a kernel command line with
`pci=off`. MMIO exits are answered with all-ones. The 32-bit MMIO hole is `[3 GiB, 4 GiB)`; the I/O APIC is at
`0xFEC0_0000` and the local APIC at `0xFEE0_0000`. All hypervisor calls go through `crates/virtax-vmm-hv`, the
only crate allowed `unsafe`.

## Goals / Non-Goals

**Goals:**
- A PCI bus that later virtio devices (net, vsock, gpu) reuse without changes to the bus or ACPI code.
- Device I/O off the vCPU threads, so a slow disk does not stall the guest's CPUs.
- No new `unsafe` outside `virtax-vmm-hv`.

**Non-Goals:**
- Throughput targets (fio numbers belong to FR-EN05-002/006 and the performance gates of FR-PL01-007).
- PCI hot-plug, multiple PCI segments, legacy INTx.

## Decisions

**1. Virtio over modern PCI, not virtio-mmio.** FR-EN04-001 specifies modern PCI; Windows guests only have
virtio drivers for PCI; and NVMe, xHCI and VFIO (later) need a PCI bus anyway. virtio-mmio would be a smaller
first step but a dead end, and its devices would need re-plumbing later.

**2. One PCI segment, bus 0 only, ECAM inside the 32-bit MMIO hole.** ECAM for bus 0 (1 MiB) sits at the top of
the hole below the I/O APIC, and BARs are allocated from a fixed 32-bit window below it. The MCFG table and the
DSDT host bridge (`PNP0A08`/`PNP0A03`, with `_CRS` for the bus range and the BAR window) describe both. Memory
BARs only; no I/O BARs. Alternative: a 64-bit BAR window above RAM, which is deferred until devices need
large BARs.

**3. MSI-X only.** Each device gets an MSI-X table; the guest's programmed vectors become KVM GSI routes, each
fed by an irqfd. The DSDT declares no `_PRT`, so there is no INTx emulation. Linux's virtio-pci prefers MSI-X;
a guest that cannot use MSI is out of scope. The MSI routing ioctl is added to `virtax-vmm-hv` behind a safe
API.

**4. Notifications through ioeventfd, requests on a worker thread.** The queue-notify register is registered as
an ioeventfd, so a guest kick does not exit to userspace. One worker thread per device waits on it, walks the
descriptor chain with rust-vmm `virtio-queue`, does blocking `pread`/`pwrite`/`fdatasync` straight into guest
memory through `vm-memory` (no bounce buffer), then signals completion through the device's MSI-X irqfd.
io_uring (FR-EN05-002) replaces the blocking calls later without changing the device. Alternative: handle
requests on the vCPU thread at the MMIO exit, which is simpler but stalls the guest on every I/O.

**5. Format detection by magic bytes.** Before attaching, `virtax-vmm` reads each image's header and refuses
qcow2 (`QFI\xfb`), VDI (`0xbeda107f` at offset 64), VMDK (`KDMV`, or a text descriptor), VHDX (`vhdxfile`) and
VHD (a `conectix` footer). Anything else is raw. The engine also checks the file exists and maps the bus,
because those errors need `VTX-ENG-011` and the machine folder context.

**6. Locks with the standard library.** `File::try_lock` (exclusive) for read-write images and
`File::try_lock_shared` for read-only ones, held for the process lifetime; the OS releases them when the process
dies, including on `SIGKILL`.

**7. CLI.** `virtax-vmm --disk <path>[,ro|,rw]`, repeatable, in attachment order; the engine builds these from
the manifest and always writes the mode, so an image path that contains a comma stays unambiguous (a trailing
`,ro` or `,rw` is the mode; any other comma belongs to the path).

## Risks / Trade-offs

- [MSI-X routing is new hypervisor surface] → Keep it in `virtax-vmm-hv` with a `SAFETY:`-documented wrapper,
  and cover it with a KVM test that boots with two disks (two devices, separate vectors).
- [Removing `pci=off` changes every guest's boot] → The KVM tests already cover boot, power-off, reboot and the
  power button; they run with PCI enabled, with and without disks.
- [`virtio-queue` API changes between releases] → Pin the version; the queue handling stays in one module.
- [Blocking I/O on one worker limits throughput] → Accepted for this slice; FR-EN05-002/006 replace it.

## Migration Plan

No data migration. Existing machines without disks behave as before apart from PCI enumeration. Rollback is
reverting the change; manifests are unchanged by it.
