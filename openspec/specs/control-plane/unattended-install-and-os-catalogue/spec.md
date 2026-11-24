# Unattended install & OS catalogue Specification

## Purpose
Async ISO detection, cloud-init/autoinstall/preseed/kickstart/autounattend templates, GuestOSType defaults. Module `CP-10` (Unattended install & OS catalogue; Vietnamese: "Cài đặt unattended & catalog OS") from BRD-02, phase P1, priority Must. Owner crates: virtaxd. Depends on: `control-plane/storage-and-media-management`, `control-plane/vm-lifecycle-and-runtime-orchestration`. VirtualBox reference: `src/VBox/Main/src-server/UnattendedImpl.cpp, UnattendedInstaller.cpp, Main/UnattendedTemplates, src-all/Global.cpp`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: DetectInstallMedia as an asynchronous Operation (FR-CP10-001)
Virtax SHALL provide "DetectInstallMedia as an asynchronous Operation" as specified by FR-CP10-001 (BRD-02, priority Must, phase P1; Vietnamese: "DetectInstallMedia là Operation bất đồng bộ").

#### Scenario: Specified behaviour of FR-CP10-001
- **WHEN** the behaviour FR-CP10-001 covers is exercised
- **THEN** it behaves as BRD-02 specifies: `Media.DetectInstallMedia(path)` returns an Operation at once; a worker reads ISO9660/UDF/El Torito headers and WIM metadata and returns `DetectedMedia{os_type_id, version, flavor, arch, images[], unattended{supported, reason}}`. Phase 1 calls `IUnattended::detectIsoOS` on a dedicated FFI pool (60 s deadline); results are cached by (path, size, mtime, first-MiB hash).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/UnattendedImpl.cpp:575-625`

#### Scenario: Acceptance for FR-CP10-001
- **WHEN** the FR-CP10-001 acceptance check is run
- **THEN** Cancel takes effect within 1 s; tracing shows 0 FFI calls on a UI thread; re-detecting the same ISO returns from cache within 50 ms.

### Requirement: Native Rust ISO detector (FR-CP10-002)
Virtax SHALL provide "Native Rust ISO detector" as specified by FR-CP10-002 (BRD-02, priority Should, phase P2; Vietnamese: "Bộ detect ISO viết bằng Rust").

#### Scenario: Specified behaviour of FR-CP10-002
- **WHEN** the behaviour FR-CP10-002 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Re-implement detection (Windows: `idwbinfo.txt` + WIM XML; Linux: `.treeinfo`, `.disk/info`, volume ID; FreeBSD) in Rust so it works without VBoxSVC (`virtax-vmm` VMs, hosts without VirtualBox); reads ≤ 4 MiB per ISO.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/UnattendedImpl.cpp:762-2900`

#### Scenario: Acceptance for FR-CP10-002
- **WHEN** the FR-CP10-002 acceptance check is run
- **THEN** On ≥ 60 ISOs, results match VirtualBox 7.2 for ≥ 98% (os_type, version, arch) and improve on VBox failures; ≤ 300 ms per ISO on SSD.

### Requirement: OS catalogue with a modern default device set (FR-CP10-003)
Virtax SHALL provide "OS catalogue with a modern default device set" as specified by FR-CP10-003 (BRD-02, priority Must, phase P1; Vietnamese: "Catalog OS với bộ thiết bị mặc định hiện đại").

#### Scenario: Specified behaviour of FR-CP10-003
- **WHEN** the behaviour FR-CP10-003 covers is exercised
- **THEN** it behaves as BRD-02 specifies: `guestOsTypes/{id}` takes the 79 `Global.cpp` entries and applies Virtax overrides: NVMe disk (DVD on AHCI), ICH9, xHCI, HDA, EFI for modern 64-bit OSes, TPM 2.0 + Secure Boot for Windows 11, virtio-net for Linux (Windows keeps E1000 until Phase 2), host cache off; arm64 keeps virtio-scsi in Phase 1. Overrides are versioned TOML data.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-all/Global.cpp:56-719`

#### Scenario: Acceptance for FR-CP10-003
- **WHEN** the FR-CP10-003 acceptance check is run
- **THEN** A Win11 VM gets nvme0 + EFI + Secure Boot + TPM 2.0 + xHCI + HDA; an Ubuntu 24.04 VM gets NVMe + virtio-net + EFI + ICH9; both install on the VirtualBox 7.2 engine on all 6 host combinations.

### Requirement: Host-aware recommendation API (FR-CP10-004)
Virtax SHALL provide "Host-aware recommendation API" as specified by FR-CP10-004 (BRD-02, priority Must, phase P1; Vietnamese: "API khuyến nghị tài nguyên theo host").

#### Scenario: Specified behaviour of FR-CP10-004
- **WHEN** the behaviour FR-CP10-004 covers is exercised
- **THEN** it behaves as BRD-02 specifies: `GuestOsTypes.Recommend(os_type, folder)` derives from the GuestOSType and `HostInfo`: vCPU = clamp(⌊logical CPUs/3⌋, max(rec, 2), 8); RAM = clamp(25% of host RAM, rec, 50%); disk = max(rec, 64 GB) when free space ≥ 2× that. It returns bilingual `reasons[]`; `virtaxctl` uses the same API.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/GuestOSTypeImpl.cpp`

#### Scenario: Acceptance for FR-CP10-004
- **WHEN** the FR-CP10-004 acceptance check is run
- **THEN** A 12-CPU/32 GB host + Ubuntu yields 4 vCPUs/8 GB/64 GB (matching MK-03); a 12-host × 6-OS table matches 100%; an 8 GB host never gets > 4 GB.

### Requirement: Template engine with context-aware escaping and custom templates (FR-CP10-005)
Virtax SHALL provide "Template engine with context-aware escaping and custom templates" as specified by FR-CP10-005 (BRD-02, priority Must, phase P1; Vietnamese: "Engine template có escape theo ngữ cảnh và template tuỳ chỉnh").

#### Scenario: Specified behaviour of FR-CP10-005
- **WHEN** the behaviour FR-CP10-005 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Port the `UnattendedScript.cpp` engine (`@@VBOX_INSERT_*@@`, `@@VBOX_COND_*@@`) to Rust to reuse existing templates, adding context-aware escaping (XML, shell, preseed, YAML). Each template has a descriptor (matching OS, installer, outputs, delivery) and golden tests; custom templates live in `<config>/virtax/unattended/`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/UnattendedScript.cpp`

#### Scenario: Acceptance for FR-CP10-005
- **WHEN** the FR-CP10-005 acceptance check is run
- **THEN** Output is byte-identical to VirtualBox 7.2 for every P1 template in the golden corpus; a password with ' " $ \ < > & ; installs and logs in on Windows 11, Ubuntu 24.04, Debian 12 and AlmaLinux 9; a custom Linux Mint 22 template is picked automatically.

### Requirement: Windows autounattend (FR-CP10-006)
Virtax SHALL provide "Windows autounattend" as specified by FR-CP10-006 (BRD-02, priority Must, phase P1; Vietnamese: "Windows autounattend").

#### Scenario: Specified behaviour of FR-CP10-006
- **WHEN** the behaviour FR-CP10-006 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Windows 10/11, Server 2022/2025 x64 and Windows 11 arm64: generate `autounattend.xml` on aux media (image index, language, time zone, keyboard), create a local account skipping Microsoft account/OOBE, auto-log on once for post-install; install Guest Additions (Phase 1) or virtio-win + `virtax-agent` (Phase 2).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/UnattendedTemplates/win_nt6_unattended.xml`

#### Scenario: Acceptance for FR-CP10-006
- **WHEN** the FR-CP10-006 acceptance check is run
- **THEN** Windows 11 24H2 x64 and arm64 reach the desktop within 25 minutes on the reference host (NVMe, 8 vCPUs), GA/agent online, 0 clicks in Setup.

### Requirement: Ubuntu autoinstall + cloud-init (FR-CP10-007)
Virtax SHALL provide "Ubuntu autoinstall + cloud-init" as specified by FR-CP10-007 (BRD-02, priority Must, phase P1; Vietnamese: "Ubuntu autoinstall + cloud-init").

#### Scenario: Specified behaviour of FR-CP10-007
- **WHEN** the behaviour FR-CP10-007 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Ubuntu Desktop ≥ 22.10 and Server ≥ 20.04 use autoinstall via a NoCloud seed (`user-data`, `meta-data` on a CIDATA volume/VISO) + the `autoinstall` kernel parameter; older releases use preseed. SHA-512 crypt passwords; SSH keys, extra packages, proxy and `late-commands` supported.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/UnattendedTemplates/ubuntu_autoinstall_user_data`

#### Scenario: Acceptance for FR-CP10-007
- **WHEN** the FR-CP10-007 acceptance check is run
- **THEN** Ubuntu 24.04 Desktop/Server, amd64/arm64 reach the login screen within 15 minutes and accept key-based SSH; aux media holds no plaintext password.

### Requirement: Debian, RHEL/Fedora, openSUSE and FreeBSD templates (FR-CP10-008)
Virtax SHALL provide "Debian, RHEL/Fedora, openSUSE and FreeBSD templates" as specified by FR-CP10-008 (BRD-02, priority Should, phase P1; Vietnamese: "Template Debian, họ RHEL/Fedora, openSUSE và FreeBSD").

#### Scenario: Specified behaviour of FR-CP10-008
- **WHEN** the behaviour FR-CP10-008 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Debian 11-13 via preseed (mirror, proxy, "Avoid updates over network"); RHEL/Alma/Rocky/Oracle Linux 8-10 via kickstart; **add Fedora** Server/Everything (VirtualBox blocks it: "Fedora ISOs cannot be installed at present") using `--iscrypted` SHA-512 instead of `--plaintext`; openSUSE/SLES (AutoYaST, skipped by VirtualBox) and FreeBSD 14 as best-effort.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/UnattendedImpl.cpp:4509-4536`

#### Scenario: Acceptance for FR-CP10-008
- **WHEN** the FR-CP10-008 acceptance check is run
- **THEN** Debian 12 netinst within 20 minutes; AlmaLinux 9, Rocky 9 and Fedora 41 Server install hands-free; openSUSE Leap 15.6 and FreeBSD 14 run in nightly CI and fall back to manual install on failure.

### Requirement: Unattended pipeline as one multi-step Operation with install monitoring (FR-CP10-009)
Virtax SHALL provide "Unattended pipeline as one multi-step Operation with install monitoring" as specified by FR-CP10-009 (BRD-02, priority Must, phase P1; Vietnamese: "Pipeline unattended là một Operation nhiều bước, có theo dõi cài đặt").

#### Scenario: Specified behaviour of FR-CP10-009
- **WHEN** the behaviour FR-CP10-009 covers is exercised
- **THEN** it behaves as BRD-02 specifies: `Machines.CreateUnattended(spec)` runs on the VM actor: validate → create disk → render templates → build aux media → configure VM → boot → monitor install (Guest Additions run level in Phase 1, the `virtax-agent` "hello" over vsock in Phase 2, reboot count) → detach media → ready. Phase 1 wraps `prepare`, `constructMedia`, `reconfigureVM` and `done` in a worker. A timeout (60 minutes default) raises VTX-UNA-010 with a guest screenshot; cancel rolls back or keeps a "draft" VM.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/UnattendedImpl.cpp:2902-3213`

#### Scenario: Acceptance for FR-CP10-009
- **WHEN** the FR-CP10-009 acceptance check is run
- **THEN** Each step reports progress at least every 5 s; Ready is set within 30 s of agent/GA online; 100 random cancellations leave 0 orphaned files/registry rows.

### Requirement: Safe handling of passwords and product keys (FR-CP10-010)
Virtax SHALL provide "Safe handling of passwords and product keys" as specified by FR-CP10-010 (BRD-02, priority Must, phase P1; Vietnamese: "Xử lý mật khẩu và product key an toàn").

#### Scenario: Specified behaviour of FR-CP10-010
- **WHEN** the behaviour FR-CP10-010 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Passwords/product keys live only in Operation memory (`secrecy`, zeroized), never in the registry, `machine.toml`, logs or events. Linux uses SHA-512 crypt; Windows (whose XML needs plaintext) uses 0600/user-ACL aux media that is detached and deleted once the first logon completes.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/UnattendedTemplates/fedora_ks.cfg:14,52`

#### Scenario: Acceptance for FR-CP10-010
- **WHEN** the FR-CP10-010 acceptance check is run
- **THEN** After install, scanning the VM folder, registry, logs and diagnostics bundle for the test password finds 0 hits; aux media is deleted within 60 s of agent/GA online.

### Requirement: Unsupported reasons and manual-install fallback (FR-CP10-011)
Virtax SHALL provide "Unsupported reasons and manual-install fallback" as specified by FR-CP10-011 (BRD-02, priority Must, phase P1; Vietnamese: "Lý do không hỗ trợ unattended và cài tay dự phòng").

#### Scenario: Specified behaviour of FR-CP10-011
- **WHEN** the behaviour FR-CP10-011 covers is exercised
- **THEN** it behaves as BRD-02 specifies: When unattended is unsupported, the API returns a reason code (`INSTALLER_CALAMARES`, `VERSION_UNKNOWN`, `ARCH_MISMATCH`, `TEMPLATE_MISSING`, `DISTRO_TOO_OLD`) instead of a silent `false`; the UI explains it bilingually and falls back to manual install: the VM is still created with the ISO attached and boots the installer.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-server/UnattendedImpl.cpp:4465-4542`

#### Scenario: Acceptance for FR-CP10-011
- **WHEN** the FR-CP10-011 acceptance check is run
- **THEN** A Lubuntu 24.04 ISO shows "The Calamares installer can't be automated" and the VM boots the ISO; 100% of `getIsUnattendedInstallSupported` branches map to a reason code.

### Requirement: Quick create from a cloud image (FR-CP10-012)
Virtax SHALL provide "Quick create from a cloud image" as specified by FR-CP10-012 (BRD-02, priority Could, phase P2; Vietnamese: "Tạo nhanh từ cloud image").

#### Scenario: Specified behaviour of FR-CP10-012
- **WHEN** the behaviour FR-CP10-012 covers is exercised
- **THEN** it behaves as BRD-02 specifies: Create VMs from cloud images (Ubuntu, Debian, Fedora, AlmaLinux) instead of ISOs: download after confirmation, verify SHA-256, resize the disk, seed cloud-init NoCloud (user, SSH key, hostname) and boot on `virtax-vmm`; aimed at DevOps labs.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP10-012
- **WHEN** the FR-CP10-012 acceptance check is run
- **THEN** `virtaxctl vm create --cloud-image ubuntu-24.04 --ssh-key …` reaches a working SSH login within 30 s (image cached) on KVM and HVF.
