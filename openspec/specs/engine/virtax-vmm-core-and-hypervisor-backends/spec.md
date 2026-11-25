# virtax-vmm core & hypervisor backends Specification

## Purpose
Rust VMM built on OpenVMM crates; KVM/WHP/HVF; lock-free exit path; one-shot timers; ioeventfd/irqfd. Module `EN-03` (virtax-vmm core & hypervisor backends; Vietnamese: "virtax-vmm core & hypervisor backend") from BRD-10, phase P2, priority Must. Owner crates: virtax-vmm, virtax-vmm-core, virtax-vmm-hv. Depends on: `engine/engine-trait-and-legacy-adapter`. VirtualBox reference: `src/VBox/VMM (NEM backends, TM, EM, PGM concepts)`. Implementation status (2026-11-25): partial — `virtax-vmm` on KVM, Linux x86_64 only, built on rust-vmm crates (decision D4 pending), with ioeventfd/irqfd and disks from the manifest; no HVF/WHP/arm64.

## Requirements

### Requirement: Hypervisor trait over KVM, WHP and HVF (FR-EN03-001)
Virtax SHALL provide "Hypervisor trait over KVM, WHP and HVF" as specified by FR-EN03-001 (BRD-10, priority Must, phase P2; Vietnamese: "Trait hypervisor trên KVM, WHP, HVF").

#### Scenario: Specified behaviour of FR-EN03-001
- **WHEN** the behaviour FR-EN03-001 covers is exercised
- **THEN** it behaves as BRD-10 specifies: `virtax-vmm-hv` wraps OpenVMM's virt crates behind `Hypervisor`/`Vm`/`Vcpu` traits with KVM (x64, arm64), WHP (x64, arm64) and HVF (arm64) backends; each reports capabilities (CPUID policy, irqchip, dirty tracking, page size). A rust-vmm fallback exists on Linux.
- **AND** the VirtualBox reference for the current behaviour is `VMM/VMMR3/target-*/NEMR3Native-*.cpp (NEM 27,353 LOC)`

#### Scenario: Acceptance for FR-EN03-001
- **WHEN** the FR-EN03-001 acceptance check is run
- **THEN** The same `virtax-vmm-core` boots Ubuntu 24.04 on KVM x64/arm64, WHP x64/arm64 and HVF arm64; backend-specific code outside `virtax-vmm-hv` ≤ 5%.

### Requirement: Per-host backend strategy (FR-EN03-002)
Virtax SHALL provide "Per-host backend strategy" as specified by FR-EN03-002 (BRD-10, priority Must, phase P2; Vietnamese: "Chiến lược backend theo từng host").

#### Scenario: Specified behaviour of FR-EN03-002
- **WHEN** the behaviour FR-EN03-002 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Per the matrix in the additional content: KVM first-class (full irqchip, arm64 vGICv3 + ITS, PSCI in KVM); HVF requires macOS 15+ for `hv_gic`; WHP uses the Hyper-V LAPIC in x2APIC mode, exit-context registers and the emulator API for MMIO; Windows arm64 best-effort; macOS x64 not targeted. Rollout KVM → HVF → WHP.
- **AND** the VirtualBox reference for the current behaviour is `analysis 01 §7; NEMR3Native-darwin-armv8.cpp:1750-1760`

#### Scenario: Acceptance for FR-EN03-002
- **WHEN** the FR-EN03-002 acceptance check is run
- **THEN** `virtaxctl host caps --json` on each runner matches the matrix; choosing virtax-vmm on macOS 14 → VTX-HV-010 "requires macOS 15" with a legacy-engine hint.

### Requirement: vCPU loop without global locks (FR-EN03-003)
Virtax SHALL provide "vCPU loop without global locks" as specified by FR-EN03-003 (BRD-10, priority Must, phase P2; Vietnamese: "vCPU loop không global lock").

#### Scenario: Specified behaviour of FR-EN03-003
- **WHEN** the behaviour FR-EN03-003 covers is exercised
- **THEN** it behaves as BRD-10 specifies: One thread per vCPU that only runs the guest; exits dispatch through an immutable `Arc` snapshot of MMIO/PIO ranges (arc-swap); only per-device or per-queue locks exist. vCPU kicks use `immediate_exit` + `pthread_kill`, `WHvCancelRunVirtualProcessor`, `hv_vcpus_exit`.
- **AND** the VirtualBox reference for the current behaviour is `PGMAllPhys.cpp:4271-4325; IOMAllMmioNew.cpp:787; IOMInternal.h:34`

#### Scenario: Acceptance for FR-EN03-003
- **WHEN** the FR-EN03-003 acceptance check is run
- **THEN** An 8-vCPU guest under MMIO-heavy fio: 0 VM-wide locks on the exit path; exits/s scale ≥ 6× from 1 to 8 vCPUs.

### Requirement: Minimal syscalls per exit, lazy state (FR-EN03-004)
Virtax SHALL provide "Minimal syscalls per exit, lazy state" as specified by FR-EN03-004 (BRD-10, priority Should, phase P2; Vietnamese: "Tối thiểu syscall mỗi exit, state lazy").

#### Scenario: Specified behaviour of FR-EN03-004
- **WHEN** the behaviour FR-EN03-004 covers is exercised
- **THEN** it behaves as BRD-10 specifies: vCPU state stays in the hypervisor and is read on demand: KVM uses sync regs and reads LAPIC/MP state only when needed; arm64 reads ONE_REG on demand and writes only dirty registers; HVF is lazy; WHP uses exit-context registers.
- **AND** the VirtualBox reference for the current behaviour is `NEMR3Native-linux-x86.cpp:2592-2652; NEMR3Native-linux-armv8.cpp:794-975`

#### Scenario: Acceptance for FR-EN03-004
- **WHEN** the FR-EN03-004 acceptance check is run
- **THEN** KVM x64: ≤ 1.05 syscalls per MMIO/PIO exit on average; KVM arm64: ≤ 2 ioctls besides `KVM_RUN` per exit.

### Requirement: ioeventfd/irqfd fast paths and doorbells (FR-EN03-005)
Virtax SHALL provide "ioeventfd/irqfd fast paths and doorbells" as specified by FR-EN03-005 (BRD-10, priority Must, phase P2; Vietnamese: "Fast path ioeventfd/irqfd và doorbell").

#### Scenario: Specified behaviour of FR-EN03-005
- **WHEN** the behaviour FR-EN03-005 covers is exercised
- **THEN** it behaves as BRD-10 specifies: KVM: `KVM_IOEVENTFD` for every virtio notify, `KVM_IRQFD` for MSI-X, vhost-net/vhost-user where available. WHP: `WHvRegisterPartitionDoorbellEvent` + `WHvRequestInterrupt` from I/O threads. HVF: EVENT_IDX, batching, `hv_gic_send_msi`. Devices run on their own workers (diagram 24, E4).
- **AND** the VirtualBox reference for the current behaviour is `NEMR3NativeTemplate-linux.cpp.h:547,555; VirtioCore.cpp:2390`

#### Scenario: Acceptance for FR-EN03-005
- **WHEN** the FR-EN03-005 acceptance check is run
- **THEN** virtio-blk on KVM: 0 userspace vCPU exits per kick; WHP with doorbells: notify exits to user mode −90%.

### Requirement: Tickless one-shot timers off vCPU threads (FR-EN03-006)
Virtax SHALL provide "Tickless one-shot timers off vCPU threads" as specified by FR-EN03-006 (BRD-10, priority Must, phase P2; Vietnamese: "Timer one-shot, tickless, ngoài vCPU thread").

#### Scenario: Specified behaviour of FR-EN03-006
- **WHEN** the behaviour FR-EN03-006 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Device timers run on event loops (timerfd, high-resolution waitable timers, `dispatch_source` with leeway), armed one-shot at the nearest deadline with no periodic tick. Guest timers use the in-kernel LAPIC/vtimer with TSC-deadline. The Windows system timer resolution is never changed.
- **AND** the VirtualBox reference for the current behaviour is `TMR3.cpp:227-241,2345-2373; Runtime/r3/win/timer-win.cpp:260-290`

#### Scenario: Acceptance for FR-EN03-006
- **WHEN** the FR-EN03-006 acceptance check is run
- **THEN** Idle Ubuntu guest: VM process ≤ 10 wakeups/s and ≤ 0.5% of a core; device timers p99 ≤ 1 ms; `powercfg /energy` shows no timer request from `virtax-vmm`.

### Requirement: No userspace spinning, power-aware halting (FR-EN03-007)
Virtax SHALL provide "No userspace spinning, power-aware halting" as specified by FR-EN03-007 (BRD-10, priority Must, phase P2; Vietnamese: "Không spin ở userspace, halt theo nguồn điện").

#### Scenario: Specified behaviour of FR-EN03-007
- **WHEN** the behaviour FR-EN03-007 covers is exercised
- **THEN** it behaves as BRD-10 specifies: HLT/WFI are handled by the hypervisor; when an exit is unavoidable (HVF) the vCPU blocks until its deadline, never busy-spinning. KVM halt-polling follows the profile (0 on battery, adaptive on AC). The execution cap uses cgroup `cpu.max`, job objects or QoS instead of 5 ms sleeps.
- **AND** the VirtualBox reference for the current behaviour is `VMR3Emt.cpp:546-572,632-638; EMR3Nem.cpp:475-490`

#### Scenario: Acceptance for FR-EN03-007
- **WHEN** the FR-EN03-007 acceptance check is run
- **THEN** On battery an idle 4-vCPU VM uses ≤ 1% of a core; `perf sched` shows 0 runnable vCPUs while idle; a 50% cap measures within ±5%.

### Requirement: Guest memory: memfd + THP, free page reporting (FR-EN03-008)
Virtax SHALL provide "Guest memory: memfd + THP, free page reporting" as specified by FR-EN03-008 (BRD-10, priority Must, phase P2; Vietnamese: "Guest memory: memfd + THP, free page reporting").

#### Scenario: Specified behaviour of FR-EN03-008
- **WHEN** the behaviour FR-EN03-008 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Guest RAM uses memfd (Linux, needed for vhost-user), THP/hugetlbfs when available and optional large pages on Windows; page size is read at run time; pages fault in lazily. virtio-balloon free page reporting returns memory via `MADV_FREE`, `DiscardVirtualMemory`, `madvise` (macOS).
- **AND** the VirtualBox reference for the current behaviour is `PGMR3Phys.cpp:1909-1926,5465-5650; GMMR3.cpp:296-303`

#### Scenario: Acceptance for FR-EN03-008
- **WHEN** the FR-EN03-008 acceptance check is run
- **THEN** An idle 8 GiB VM has RSS ≤ 1.5 GiB; the guest freeing 4 GiB → host RSS −3 GiB within 30 s; THP backs ≥ 80% of guest RAM.

### Requirement: Versioned saved state, snapshots and dirty ring (FR-EN03-009)
Virtax SHALL provide "Versioned saved state, snapshots and dirty ring" as specified by FR-EN03-009 (BRD-10, priority Must, phase P2; Vietnamese: "Saved state, snapshot và dirty ring").

#### Scenario: Specified behaviour of FR-EN03-009
- **WHEN** the behaviour FR-EN03-009 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Per-device saved-state units (the SSM concept) in serde with version tags; save/restore and live snapshots using the KVM dirty ring / `WHvQueryGpaRangeDirtyBitmap` / HVF write protection (pre-copy, then pause); restore resets TSC/vtimer (`KVM_VCPU_TSC_OFFSET`, `hv_vcpu_set_vtimer_offset`, `WHvResumePartitionTime`). `.sav` compatibility is not needed.
- **AND** the VirtualBox reference for the current behaviour is `VMMR3/SSM.cpp; NEMR3NativeTemplate-linux.cpp.h:1337-1355; NEMR3Native-win-x86.cpp:3767-3805`

#### Scenario: Acceptance for FR-EN03-009
- **WHEN** the FR-EN03-009 acceptance check is run
- **THEN** Save/restore of 8 GiB Ubuntu ≤ 5 s each way; a live snapshot under memtester pauses ≤ 200 ms; clock drift ≤ 1 ms; N-1 state restores.

### Requirement: Control server: config blob, live changes, metrics (FR-EN03-010)
Virtax SHALL provide "Control server: config blob, live changes, metrics" as specified by FR-EN03-010 (BRD-10, priority Must, phase P2; Vietnamese: "Control server: config blob, thay đổi live, metrics").

#### Scenario: Specified behaviour of FR-EN03-010
- **WHEN** the behaviour FR-EN03-010 covers is exercised
- **THEN** it behaves as BRD-10 specifies: `virtax-vmm` receives one typed, versioned config blob (from `MachineSpec`) at start instead of ≈300 getters; runtime changes (hot-plug, cable, CD, balloon, display hint) are messages returning an ApplyMode. Exit/wakeup/halt/queue counters go through shared memory, sampled only while subscribed (MK-06).
- **AND** the VirtualBox reference for the current behaviour is `ConsoleImplConfigCommon.cpp; ConsoleImpl.cpp:7246`

#### Scenario: Acceptance for FR-EN03-010
- **WHEN** the FR-EN03-010 acceptance check is run
- **THEN** Start request to first guest instruction ≤ 300 ms (firmware excluded); live CD mount ≤ 100 ms; unknown field → VTX-VMM-001; no subscriber → 0 extra wakeups.

### Requirement: Re-attach after a daemon restart (FR-EN03-011)
Virtax SHALL provide "Re-attach after a daemon restart" as specified by FR-EN03-011 (BRD-10, priority Must, phase P2; Vietnamese: "Re-attach sau khi daemon restart").

#### Scenario: Specified behaviour of FR-EN03-011
- **WHEN** the behaviour FR-EN03-011 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Each `virtax-vmm` has its own peer-checked control socket and a runtime record (pid, version, socket). A restarted `virtaxd` discovers and re-attaches running VMs and reconciles state, metrics and display channels; `virtax-vmm` survives daemon death (configurable orphan policy).
- **AND** the VirtualBox reference for the current behaviour is `VBoxHeadless.cpp:136-149; VirtualBoxClientImpl.cpp:757-814`

#### Scenario: Acceptance for FR-EN03-011
- **WHEN** the FR-EN03-011 acceptance check is run
- **THEN** Restart or kill -9 of `virtaxd` with 5 VMs running → 0 VMs stopped, re-attached within 2 s, Console back within 1 s (NFR-00-07).

### Requirement: As built — KVM backend on rust-vmm
The Phase 2 engine SHALL run guests on KVM (Linux x86_64) through rust-vmm crates (`kvm-ioctls`, `vm-memory`, `linux-loader`, `vm-superio`), with every hypervisor call behind `crates/virtax-vmm-hv`, the only crate allowed `unsafe`. This is provisional: decision D4 (OpenVMM crates by default) is TO BE CONFIRMED.

#### Scenario: Boots on KVM
- **WHEN** the pinned test guest is booted with `virtax-vmm`
- **THEN** it reaches userspace, prints the host's date and powers off with exit code 0
- **AND** this is verified by `apps/virtax-vmm/tests/boot.rs::kvm_boots_to_userspace_with_the_host_date_and_powers_off`

### Requirement: As built — One process per machine
Each machine SHALL run in its own `virtax-vmm` process that prints `virtax-vmm: guest running` on stderr when the guest starts. It SHALL exit 0 when the guest powers off, 3 on a triple fault and 1 when the VMM itself fails. A guest reboot SHALL restart the VM by re-executing the process with the same arguments and PID, unless `--no-reboot` is given. SIGTERM SHALL press the ACPI power button and SIGKILL SHALL power off at once. Source: `apps/virtax-vmm/src/main.rs`.

#### Scenario: Reboot restarts in place
- **WHEN** the guest reboots
- **THEN** the same process boots the guest again
- **AND** this is verified by `apps/virtax-vmm/tests/boot.rs::kvm_guest_reboot_restarts_the_vm_in_the_same_process`

#### Scenario: No-reboot mode
- **WHEN** the guest reboots under `--no-reboot`
- **THEN** `virtax-vmm` exits 0
- **AND** this is verified by `apps/virtax-vmm/tests/boot.rs::kvm_no_reboot_exits_when_the_guest_reboots`

#### Scenario: Power button on SIGTERM
- **WHEN** SIGTERM is sent to a running `virtax-vmm`
- **THEN** the guest receives a power-button event and powers off through ACPI
- **AND** this is verified by `apps/virtax-vmm/tests/boot.rs::kvm_sigterm_presses_the_power_button_and_the_guest_shuts_down`

### Requirement: As built — Direct kernel boot with SMP
`virtax-vmm` SHALL boot a bzImage or ELF vmlinux with an optional initramfs and extra kernel parameters (manifest `[boot]`), with 1 to 254 vCPUs on one host thread each, and SHALL reject anything else before touching the hypervisor. Source: `crates/virtax-vmm-core/src/vmm.rs`.

#### Scenario: Four vCPUs
- **WHEN** the test guest is booted with 4 vCPUs
- **THEN** the guest reports 4 CPUs
- **AND** this is verified by `apps/virtax-vmm/tests/boot.rs::kvm_boots_four_vcpus_and_powers_off_through_acpi`

#### Scenario: Invalid input
- **WHEN** the kernel file is not a kernel or the vCPU count is 0
- **THEN** `virtax-vmm` fails with a clear error
- **AND** this is verified by `crates/virtax-vmm-core/src/x86/boot.rs::rejects_a_file_that_is_not_a_kernel`

### Requirement: Disks from the machine manifest
`VirtaxVmmEngine` SHALL attach every disk attachment of a `virtio` storage controller in `machine.toml` to the VM, in controller then port order, resolving relative paths against the machine folder. Source: FR-EN04-001 with the manifest's `[[storage.controllers]]` (FR-CP02-004).

#### Scenario: Manifest disk reaches the guest
- **WHEN** a machine has a `virtio` controller with one disk attachment and is started
- **THEN** the guest sees that image as a virtio block device
- **AND** this is verified by `kvm_manifest_disk_reaches_the_guest_through_virtaxctl` (`apps/virtaxctl/tests/e2e.rs`)

#### Scenario: Read-only attachment
- **WHEN** the attachment is a `dvd`, or the image file is not writable by the user
- **THEN** the image is attached read-only
- **AND** this is verified by `vmm::tests::disks_follow_controller_then_port_order` and `vmm::tests::an_image_the_user_cannot_write_is_attached_read_only` (`crates/virtax-engine`)

### Requirement: Disk configuration is checked before start
`VirtaxVmmEngine` SHALL refuse to start a machine whose disks it cannot attach, with `VTX-ENG-011` and a message naming the attachment, before `virtax-vmm` runs. Source: FR-EN01-011 (errors with fixes) applied to the disk configuration.

#### Scenario: Unsupported bus
- **WHEN** a machine with a disk on a `sata`, `ide`, `scsi` or `nvme` controller is started
- **THEN** the start fails with `VTX-ENG-011` naming the controller and saying only `virtio` is supported
- **AND** this is verified by `vmm::tests::disks_that_cannot_be_attached_are_refused_before_start`

#### Scenario: Missing image
- **WHEN** an attachment's image file does not exist
- **THEN** the start fails with `VTX-ENG-011` naming the path, and the machine stays `powered_off`
- **AND** this is verified by `vmm::tests::disks_that_cannot_be_attached_are_refused_before_start` and `kvm_manifest_disk_reaches_the_guest_through_virtaxctl`
