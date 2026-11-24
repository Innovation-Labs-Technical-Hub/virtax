# Device model & firmware Specification

## Purpose
Minimal virtio set + NVMe/xHCI/TPM/HDA, ACPI, OVMF/AAVMF, GICv3/PL011 for arm64. Module `EN-04` (Device model & firmware; Vietnamese: "Device model & firmware") from BRD-10, phase P2, priority Must. Owner crates: virtax-vmm-devices. Depends on: `engine/virtax-vmm-core-and-hypervisor-backends`. VirtualBox reference: `src/VBox/Devices (PC, Bus, VirtIO, Storage, Network, Security, EFI)`. Implementation status (2026-11-24): partial — hardware-reduced ACPI, 16550 UART, CMOS RTC, i8042 reset and an ACPI power button; no virtio devices or firmware boot yet.

## Requirements

### Requirement: Minimal virtio device set (FR-EN04-001)
Virtax SHALL provide "Minimal virtio device set" as specified by FR-EN04-001 (BRD-10, priority Must, phase P2; Vietnamese: "Bộ thiết bị virtio tối thiểu").

#### Scenario: Specified behaviour of FR-EN04-001
- **WHEN** the behaviour FR-EN04-001 covers is exercised
- **THEN** it behaves as BRD-10 specifies: virtio 1.2 modern PCI: blk (MQ, discard, write-zeroes), net (MQ, vnet_hdr), gpu, input, snd, vsock, fs, balloon (+ free page reporting), rng, console, on `virtio-queue` with EVENT_IDX + INDIRECT_DESC. Each device implements the `VirtioDevice` trait, runs on its own worker and uses no `unsafe`.
- **AND** the VirtualBox reference for the current behaviour is `VirtioCore.h:224-252; DevVirtioNet.cpp:166-170`

#### Scenario: Acceptance for FR-EN04-001
- **WHEN** the FR-EN04-001 acceptance check is run
- **THEN** A Linux 6.x guest probes all 10 devices with in-box drivers and negotiates EVENT_IDX + INDIRECT_DESC; virtio-win installs on Windows 11.

### Requirement: NVMe so Windows boots without extra drivers (FR-EN04-002)
Virtax SHALL provide "NVMe so Windows boots without extra drivers" as specified by FR-EN04-002 (BRD-10, priority Must, phase P2; Vietnamese: "NVMe để Windows boot không cần driver").

#### Scenario: Specified behaviour of FR-EN04-002
- **WHEN** the behaviour FR-EN04-002 covers is exercised
- **THEN** it behaves as BRD-10 specifies: A Rust NVMe 1.4 controller (concepts from DevNVMe): queues = vCPUs (≤ 64), 4096 entries, per-queue MSI-X, doorbells via ioeventfd; Windows installs with in-box `stornvme`. Default for Windows guests.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Storage/DevNVMe.cpp:73-101,7181-7330`

#### Scenario: Acceptance for FR-EN04-002
- **WHEN** the FR-EN04-002 acceptance check is run
- **THEN** Unattended Windows 11 24H2 on NVMe without a driver disk succeeds on 3 host OSes; fio 4k QD32 in Windows ≥ 1.5× VBox AHCI.

### Requirement: Install-media device (FR-EN04-003)
Virtax SHALL provide "Install-media device" as specified by FR-EN04-003 (BRD-10, priority Must, phase P2; Vietnamese: "Thiết bị cho media cài đặt").

#### Scenario: Specified behaviour of FR-EN04-003
- **WHEN** the behaviour FR-EN04-003 covers is exercised
- **THEN** it behaves as BRD-10 specifies: A minimal AHCI with only an ATAPI CD (1-2 ports) for Windows Setup; Linux guests use a virtio-scsi CD or read-only virtio-blk; USB-MSD over xHCI is the alternative; media change/eject works live.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Storage/DevAHCI.cpp:87-89; DevATA.cpp:1787,8152`

#### Scenario: Acceptance for FR-EN04-003
- **WHEN** the FR-EN04-003 acceptance check is run
- **THEN** Windows 11/Server 2025 ISOs boot and Setup finds `install.wim`; Ubuntu boots from the virtio CD; live ISO swap ≤ 1 s.

### Requirement: xHCI and USB HID, no OHCI/EHCI (FR-EN04-004)
Virtax SHALL provide "xHCI and USB HID, no OHCI/EHCI" as specified by FR-EN04-004 (BRD-10, priority Must, phase P2; Vietnamese: "xHCI và USB HID, bỏ OHCI/EHCI").

#### Scenario: Specified behaviour of FR-EN04-004
- **WHEN** the behaviour FR-EN04-004 covers is exercised
- **THEN** it behaves as BRD-10 specifies: xHCI only, doorbell-driven, with none of the 1 kHz OHCI/EHCI frame timers; a USB HID tablet + keyboard for Windows without vioinput, USB-MSD, and passthrough hooks for CP-08.
- **AND** the VirtualBox reference for the current behaviour is `DevXHCI.cpp:359-383,5252; DevOHCI.cpp:4552; DevEHCI.cpp:931-932`

#### Scenario: Acceptance for FR-EN04-004
- **WHEN** the FR-EN04-004 acceptance check is run
- **THEN** An idle xHCI adds 0 periodic wakeups; the absolute pointer works in Windows 11 before virtio-win is installed.

### Requirement: TPM 2.0 CRB with libtpms (FR-EN04-005)
Virtax SHALL provide "TPM 2.0 CRB with libtpms" as specified by FR-EN04-005 (BRD-10, priority Must, phase P2; Vietnamese: "TPM 2.0 CRB với libtpms").

#### Scenario: Specified behaviour of FR-EN04-005
- **WHEN** the behaviour FR-EN04-005 covers is exercised
- **THEN** it behaves as BRD-10 specifies: TPM 2.0 with the CRB interface, libtpms in-process via FFI (Windows too), optional swtpm, plus PPI. TPM state is per VM, encrypted per CP-12 (BRD-11) and captured in snapshots.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Security/DevTpm.cpp:506,1020-1437; DrvTpmEmuTpms.cpp:509`

#### Scenario: Acceptance for FR-EN04-005
- **WHEN** the FR-EN04-005 acceptance check is run
- **THEN** Windows 11 PC Health Check passes TPM 2.0; BitLocker unlocks after reboot; TPM state is right after a snapshot restore.

### Requirement: q35-style x86_64 platform (FR-EN04-006)
Virtax SHALL provide "q35-style x86_64 platform" as specified by FR-EN04-006 (BRD-10, priority Must, phase P2; Vietnamese: "Nền tảng x86_64 kiểu q35").

#### Scenario: Specified behaviour of FR-EN04-006
- **WHEN** the behaviour FR-EN04-006 covers is exercised
- **THEN** it behaves as BRD-10 specifies: PCIe ECAM + root ports (hot-plug), MSI/MSI-X, VMM-generated ACPI (GED/PM, power button, S5), CMOS RTC, 16550, a minimal i8042, fw_cfg, pflash, SMBIOS. KVM full irqchip; WHP uses the Hyper-V LAPIC + a userspace IOAPIC. ISA DMA, FDC, SB16, LPT, the 16-bit BIOS and HPET are dropped.
- **AND** the VirtualBox reference for the current behaviour is `PC/DevACPI.cpp; PC/DevRTC.cpp; PC/DevQemuFwCfg.cpp`

#### Scenario: Acceptance for FR-EN04-006
- **WHEN** the FR-EN04-006 acceptance check is run
- **THEN** Windows 11 and Ubuntu shut down via ACPI from the Console within 10 s; `lspci` shows only listed devices.

### Requirement: arm64 virt platform (FR-EN04-007)
Virtax SHALL provide "arm64 virt platform" as specified by FR-EN04-007 (BRD-10, priority Must, phase P2; Vietnamese: "Nền tảng arm64 virt").

#### Scenario: Specified behaviour of FR-EN04-007
- **WHEN** the behaviour FR-EN04-007 covers is exercised
- **THEN** it behaves as BRD-10 specifies: GICv3 (KVM vGIC + ITS, `hv_gic`, Hyper-V GIC), PL011, PL031, PL061/GED power button, CFI pflash, PCIe ECAM, fw_cfg, PMU; PSCI in the kernel; the map matches VBox's ARM machine for easy imports.
- **AND** the VirtualBox reference for the current behaviour is `ConsoleImplConfigArmV8.cpp:473-825; DevPL011.cpp; DevPL031.cpp`

#### Scenario: Acceptance for FR-EN04-007
- **WHEN** the FR-EN04-007 acceptance check is run
- **THEN** Ubuntu 24.04 arm64 and Windows 11 arm64 boot on KVM arm64 and HVF; PSCI power-off ≤ 5 s.

### Requirement: Stock, pinned OVMF/AAVMF (FR-EN04-008)
Virtax SHALL provide "Stock, pinned OVMF/AAVMF" as specified by FR-EN04-008 (BRD-10, priority Must, phase P2; Vietnamese: "Firmware OVMF/AAVMF gốc, pin version").

#### Scenario: Specified behaviour of FR-EN04-008
- **WHEN** the behaviour FR-EN04-008 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Stock, version-pinned edk2 OVMF (x64) / AAVMF (arm64) with VirtioBlk/VirtioScsi/VirtioGpu DXE; per-VM NVRAM vars; boot order via fw_cfg `bootorder`; no PXE ROM by default.
- **AND** the VirtualBox reference for the current behaviour is `OvmfPkg/OvmfPkgX64.fdf:270-276,388-392; EFI/FirmwareBin`

#### Scenario: Acceptance for FR-EN04-008
- **WHEN** the FR-EN04-008 acceptance check is run
- **THEN** Reset to bootloader ≤ 2 s; the firmware hash is in the SBOM; the boot menu renders via GOP on virtio-gpu.

### Requirement: UEFI variable store and Secure Boot databases (FR-EN04-009)
Virtax SHALL provide "UEFI variable store and Secure Boot databases" as specified by FR-EN04-009 (BRD-10, priority Must, phase P2; Vietnamese: "Kho biến UEFI và cơ sở dữ liệu Secure Boot").

#### Scenario: Specified behaviour of FR-EN04-009
- **WHEN** the behaviour FR-EN04-009 covers is exercised
- **THEN** it behaves as BRD-10 specifies: A per-VM UEFI variable store with PK/KEK/db/dbx/MOK. Defaults: a per-VM generated PK (never Oracle's), KEK/db with the Microsoft KEK CA, UEFI CA 2011/2023 and Windows Production PCA; importing a VBox `.nvram` keeps enrolled keys. Key policy and UI belong to CP-12.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/UefiVariableStoreImpl.cpp:509-649`

#### Scenario: Acceptance for FR-EN04-009
- **WHEN** the FR-EN04-009 acceptance check is run
- **THEN** Windows 11 boots with Secure Boot; Ubuntu `mokutil --sb-state` = enabled; the default store has no Oracle PK; imported VMs still Secure Boot.

### Requirement: Driver bootstrap for Windows guests (FR-EN04-010)
Virtax SHALL provide "Driver bootstrap for Windows guests" as specified by FR-EN04-010 (BRD-10, priority Should, phase P2; Vietnamese: "Bootstrap driver cho guest Windows").

#### Scenario: Specified behaviour of FR-EN04-010
- **WHEN** the behaviour FR-EN04-010 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Until Windows has virtio-win: a USB-NCM/e1000e fallback NIC for network during OOBE, a USB HID tablet and an NVMe disk. Unattended install (CP-10) attaches the virtio-win + `virtax-agent` ISO; afterwards it offers to switch NIC/input to virtio at the next boot.
- **AND** the VirtualBox reference for the current behaviour is `Global.cpp:213-220; DevE1000.cpp:129-132`

#### Scenario: Acceptance for FR-EN04-010
- **WHEN** the FR-EN04-010 acceptance check is run
- **THEN** A fresh Windows 11 has network during OOBE; the virtio-net switch applies on the next boot; iperf3 improves ≥ 2×.

### Requirement: Device-compatibility report for "Move to virtax-vmm" (FR-EN04-011)
Virtax SHALL provide "Device-compatibility report for "Move to virtax-vmm"" as specified by FR-EN04-011 (BRD-10, priority Must, phase P2; Vietnamese: "Báo cáo tương thích thiết bị khi "Move to virtax-vmm"").

#### Scenario: Specified behaviour of FR-EN04-011
- **WHEN** the behaviour FR-EN04-011 covers is exercised
- **THEN** it behaves as BRD-10 specifies: From the VBox configuration, report each device as Supported / Converted (with driver needs) / Unsupported (blocking). E.g. AHCI → NVMe, E1000 → virtio-net or temporary USB-NCM, AC'97 → virtio-snd, VMSVGA 3D → keep the legacy engine. Used by `Machines.ChangeEngine(dry_run)` and rollback.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-all/Global.cpp:213-292; include/VBox/settings.h`

#### Scenario: Acceptance for FR-EN04-011
- **WHEN** the FR-EN04-011 acceptance check is run
- **THEN** A 20-VM VBox corpus: each report ≤ 1 s with 0 "unknown" devices; VMs with 3D are blocked unless the user accepts 2D.

### Requirement: As built — Hardware-reduced ACPI tables
`virtax-vmm` SHALL provide RSDP, XSDT, a hardware-reduced FADT (sleep control/status and reset registers; no PS/2 controller or VGA), a DSDT and a MADT listing every vCPU and the I/O APIC. The DSDT SHALL describe every ISA device a guest must use, because a hardware-reduced platform has no legacy PIC and Linux only wires the interrupts the DSDT describes (COM1 is `PNP0501` at 0x3F8, IRQ 4). Source: `crates/virtax-vmm-core/src/x86/acpi.rs`.

#### Scenario: Tables are well formed
- **WHEN** the tables are generated
- **THEN** every length and checksum is valid and the DSDT declares `\_S5_`, COM1, the RTC, the power button and the GED
- **AND** this is verified by `crates/virtax-vmm-core/src/x86/acpi.rs::dsdt_declares_s5_and_com1`

#### Scenario: Guest powers off
- **WHEN** the guest runs `poweroff`
- **THEN** it enters ACPI S5 and the VM ends
- **AND** this is verified by `apps/virtax-vmm/tests/boot.rs::kvm_boots_four_vcpus_and_powers_off_through_acpi`

### Requirement: As built — CMOS real-time clock
`virtax-vmm` SHALL emulate an MC146818 CMOS clock on ports 0x70/0x71 that reports the host's UTC time in BCD, 24-hour mode, described in the DSDT as `PNP0B00`. BCD is required because Linux on x86 decodes the CMOS clock as BCD regardless of status register B. Source: `crates/virtax-vmm-core/src/rtc.rs`.

#### Scenario: Register encoding
- **WHEN** the clock registers are read for 2000-02-29 12:34:56 UTC
- **THEN** they hold BCD values in 24-hour mode
- **AND** this is verified by `crates/virtax-vmm-core/src/rtc.rs::registers_report_bcd_24h_time_and_ignore_the_nmi_bit`

#### Scenario: Guest date
- **WHEN** the test guest boots
- **THEN** its `date -u` matches the host's UTC date
- **AND** this is verified by `apps/virtax-vmm/tests/boot.rs::kvm_boots_to_userspace_with_the_host_date_and_powers_off`

### Requirement: As built — ACPI power button
`virtax-vmm` SHALL expose a power button (`PNP0C0C`) raised through a Generic Event Device (`ACPI0013`) on GSI 5 whose `_EVT` method notifies the button. Source: `crates/virtax-vmm-core/src/x86/acpi.rs`, `aml.rs`.

#### Scenario: AML encoding
- **WHEN** the GED and notification AML is generated
- **THEN** it matches the hand-assembled encoding
- **AND** this is verified by `crates/virtax-vmm-core/src/x86/aml.rs::power_button_pieces`

### Requirement: As built — Legacy port devices
`virtax-vmm` SHALL emulate a 16550 UART on COM1 (output to the machine's `logs/serial.log`) and SHALL treat an i8042 reset command or an ACPI reset-register write as a guest reboot. Source: `crates/virtax-vmm-core/src/devices.rs`.

#### Scenario: Serial output and reset
- **WHEN** the guest writes to COM1 and then pulses the reset line
- **THEN** the bytes reach the serial output and the VMM sees a reboot request
- **AND** this is verified by `crates/virtax-vmm-core/src/devices.rs::uart_output_reset_and_unclaimed_ports`
