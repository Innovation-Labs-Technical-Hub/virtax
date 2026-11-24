# Packaging, signing & installers Specification

## Purpose
MSIX/MSI, notarized .dmg, .deb/.rpm/Flatpak/AppImage; Authenticode/codesign/GPG signing; upgrades without driver removal. Module `PL-02` (Packaging, signing & installers; Vietnamese: "Đóng gói, ký số & cài đặt") from BRD-12, phase P1, priority Must. Owner crates: xtask dist. Depends on: `platform/build-and-ci-cd`. VirtualBox reference: `src/VBox/Installer (win MSI, darwin pkg, linux run/deb/rpm), debian/`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Windows: per-user MSIX and enterprise MSI (FR-PL02-001)
Virtax SHALL provide "Windows: per-user MSIX and enterprise MSI" as specified by FR-PL02-001 (BRD-12, priority Must, phase P1; Vietnamese: "Windows: MSIX per-user và MSI cho doanh nghiệp").

#### Scenario: Specified behaviour of FR-PL02-001
- **WHEN** the behaviour FR-PL02-001 covers is exercised
- **THEN** it behaves as BRD-12 specifies: MSIX installs per user without admin and has a winget manifest; a per-machine WiX MSI for enterprises supports silent install and ADMX; separate x64 and arm64 packages; the helper service installs only when bridged/USB is needed.
- **AND** the VirtualBox reference for the current behaviour is `Installer/win/VirtualBox.wxs; VBoxMergeAppCA.wxi:28-35`

#### Scenario: Acceptance for FR-PL02-001
- **WHEN** the FR-PL02-001 acceptance check is run
- **THEN** MSIX installs in ≤ 60 s with no UAC prompt; `msiexec /qn` works with policies; uninstall leaves VM folders untouched.

### Requirement: macOS: notarised .app in a .dmg (FR-PL02-002)
Virtax SHALL provide "macOS: notarised .app in a .dmg" as specified by FR-PL02-002 (BRD-12, priority Must, phase P1; Vietnamese: "macOS: .app notarized trong .dmg").

#### Scenario: Specified behaviour of FR-PL02-002
- **WHEN** the behaviour FR-PL02-002 covers is exercised
- **THEN** it behaves as BRD-12 specifies: A drag-and-drop `.app` in a `.dmg`, hardened runtime, the `com.apple.security.hypervisor` entitlement for `virtax-vmm` (and `com.apple.vm.networking` if granted, otherwise an SMAppService helper); Developer ID codesign + `notarytool` + staple; installable in `~/Applications`.
- **AND** the VirtualBox reference for the current behaviour is `Installer/darwin/Makefile.kmk:51-56,189-191; Config.kmk:1838-1857`

#### Scenario: Acceptance for FR-PL02-002
- **WHEN** the FR-PL02-002 acceptance check is run
- **THEN** Opens without Gatekeeper warnings on clean macOS 14/15; `spctl -a -vv` accepts it; `codesign --verify --deep --strict` passes.

### Requirement: Linux: one .deb and one .rpm per arch (FR-PL02-003)
Virtax SHALL provide "Linux: one .deb and one .rpm per arch" as specified by FR-PL02-003 (BRD-12, priority Must, phase P1; Vietnamese: "Linux: một .deb và một .rpm cho mỗi arch").

#### Scenario: Specified behaviour of FR-PL02-003
- **WHEN** the behaviour FR-PL02-003 covers is exercised
- **THEN** it behaves as BRD-12 specifies: One `.deb` and one `.rpm` each for x64 and arm64 on a glibc 2.28 baseline (instead of 14 + 15 per-distro packages); GPG-signed APT/DNF repos; a `systemd --user` unit for `virtaxd`; polkit/udev rules for the helper; no kernel-module build.
- **AND** the VirtualBox reference for the current behaviour is `Installer/linux/distributions_deb; debian/control:18,44; preinst:65-80`

#### Scenario: Acceptance for FR-PL02-003
- **WHEN** the FR-PL02-003 acceptance check is run
- **THEN** Installs from the repo on Ubuntu 22.04/24.04, Debian 12 and Fedora 40+ (x64/arm64); `apt upgrade` with running VMs does not abort.

### Requirement: Flatpak and AppImage (FR-PL02-004)
Virtax SHALL provide "Flatpak and AppImage" as specified by FR-PL02-004 (BRD-12, priority Should, phase P1; Vietnamese: "Flatpak và AppImage").

#### Scenario: Specified behaviour of FR-PL02-004
- **WHEN** the behaviour FR-PL02-004 covers is exercised
- **THEN** it behaves as BRD-12 specifies: A Flathub Flatpak with `--device=kvm` and portals; the helper stays outside the sandbox and is reached via polkit; an AppImage with zsync. BYO-engine limits inside Flatpak are documented; the driverless Engine Pack and `virtax-vmm` are the main paths.
- **AND** the VirtualBox reference for the current behaviour is `Config.kmk:1197-1199 (/opt/VirtualBox hard-wired)`

#### Scenario: Acceptance for FR-PL02-004
- **WHEN** the FR-PL02-004 acceptance check is run
- **THEN** The Flathub build passes the linter; the AppImage runs on Ubuntu 22.04 without installing; the Flatpak boots a VM through `/dev/kvm`.

### Requirement: Per-OS code signing (FR-PL02-005)
Virtax SHALL provide "Per-OS code signing" as specified by FR-PL02-005 (BRD-12, priority Must, phase P1; Vietnamese: "Ký số theo từng OS").

#### Scenario: Specified behaviour of FR-PL02-005
- **WHEN** the behaviour FR-PL02-005 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Authenticode (OV/EV or Azure Trusted Signing) for every EXE/DLL/MSI/MSIX with RFC 3161 timestamps; codesign + hardened runtime + notarisation on macOS; GPG for repos and `.asc` files; minisign for update artifacts. Keys live in an HSM/KMS and sign only on protected tags.
- **AND** the VirtualBox reference for the current behaviour is `Installer/win/Makefile.kmk:562-601; Config.kmk:1787-1860`

#### Scenario: Acceptance for FR-PL02-005
- **WHEN** the FR-PL02-005 acceptance check is run
- **THEN** After packaging, CI verifies every binary (`signtool verify /pa`, `codesign -v`, `gpg --verify`); 0 unsigned PE files in the MSI.

### Requirement: Attestation signing for VBoxSup only if shipped (FR-PL02-006)
Virtax SHALL provide "Attestation signing for VBoxSup only if shipped" as specified by FR-PL02-006 (BRD-12, priority Could, phase P1; Vietnamese: "Attestation signing cho VBoxSup chỉ khi thật sự ship").

#### Scenario: Specified behaviour of FR-PL02-006
- **WHEN** the behaviour FR-PL02-006 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Only if the Engine Pack ships HM for Windows x64 without Hyper-V: EV cert + Microsoft Partner Center attestation and HVCI testing. Default: not shipped (such machines use official BYO). Never add certificates to TrustedPublisher/Root.
- **AND** the VirtualBox reference for the current behaviour is `Installer/win/Stub/VBoxStub.cpp:889-925; Makefile.kmk:562-601`

#### Scenario: Acceptance for FR-PL02-006
- **WHEN** the FR-PL02-006 acceptance check is run
- **THEN** The decision is recorded in an ADR; if shipped, the driver passes HVCI checks and the installer leaves trust stores untouched (`certutil` check).

### Requirement: Upgrades without uninstalling (FR-PL02-007)
Virtax SHALL provide "Upgrades without uninstalling" as specified by FR-PL02-007 (BRD-12, priority Must, phase P1; Vietnamese: "Nâng cấp không cần gỡ cài đặt").

#### Scenario: Specified behaviour of FR-PL02-007
- **WHEN** the behaviour FR-PL02-007 covers is exercised
- **THEN** it behaves as BRD-12 specifies: File-level upgrades: native MSIX, an MSI major upgrade with `RemoveExistingProducts` after `InstallFinalize`, atomic `.app` replacement, normal Linux packages. Running VMs continue (new binaries on next start), the daemon restarts gracefully (re-attach in P2), the registry migrates with a backup.
- **AND** the VirtualBox reference for the current behaviour is `VirtualBox.wxs:124,417; UserInterface.wxi:566-571`

#### Scenario: Acceptance for FR-PL02-007
- **WHEN** the FR-PL02-007 acceptance check is run
- **THEN** Upgrading N-1 → N with 3 VMs running: 0 VMs stopped, ≤ 2 min, no reboot; no blocking FilesInUse dialog.

### Requirement: Per-user install where possible (FR-PL02-008)
Virtax SHALL provide "Per-user install where possible" as specified by FR-PL02-008 (BRD-12, priority Must, phase P1; Vietnamese: "Cài per-user khi có thể").

#### Scenario: Specified behaviour of FR-PL02-008
- **WHEN** the behaviour FR-PL02-008 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Per-user by default: MSIX (Windows), `~/Applications` (macOS), Flatpak/AppImage (Linux). Elevation only for the optional helper, requested at first use with an explanation; no setuid.
- **AND** the VirtualBox reference for the current behaviour is `Installer/linux/install.sh:296-313; Config.kmk:1197-1199`

#### Scenario: Acceptance for FR-PL02-008
- **WHEN** the FR-PL02-008 acceptance check is run
- **THEN** Install Manager + Console + `virtaxd` and run a NAT VM without admin on all 3 OSes (P1 with the driverless Engine Pack where available, P2 with `virtax-vmm`).

### Requirement: No kernel-module builds on user machines (FR-PL02-009)
Virtax SHALL provide "No kernel-module builds on user machines" as specified by FR-PL02-009 (BRD-12, priority Must, phase P1; Vietnamese: "Không build module kernel trên máy người dùng").

#### Scenario: Specified behaviour of FR-PL02-009
- **WHEN** the behaviour FR-PL02-009 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Virtax packages never compile kernel code. P1 Linux: BYO uses the vboxdrv already installed, or the driverless Engine Pack on KVM; P2 is KVM-only. The Health page detects vboxdrv building at boot and suggests going driverless.
- **AND** the VirtualBox reference for the current behaviour is `Installer/linux/vboxdrv.sh:115-136,545,770-829`

#### Scenario: Acceptance for FR-PL02-009
- **WHEN** the FR-PL02-009 acceptance check is run
- **THEN** Installing needs no gcc/make/kernel headers; installing Virtax never triggers `vboxdrv.sh setup`.

### Requirement: Coexistence with an installed VirtualBox (FR-PL02-010)
Virtax SHALL provide "Coexistence with an installed VirtualBox" as specified by FR-PL02-010 (BRD-12, priority Must, phase P1; Vietnamese: "Cùng tồn tại với VirtualBox đã cài").

#### Scenario: Specified behaviour of FR-PL02-010
- **WHEN** the behaviour FR-PL02-010 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Install folders, service names, bundle IDs and sockets differ from VirtualBox; the Engine Pack never overwrites official VirtualBox COM registrations, drivers or network adapters. On Windows with official VirtualBox present, BYO is preferred over the Engine Pack.
- **AND** the VirtualBox reference for the current behaviour is `glue/initterm.cpp:575-615; VBoxProxyStub.c:1279-1641`

#### Scenario: Acceptance for FR-PL02-010
- **WHEN** the FR-PL02-010 acceptance check is run
- **THEN** Installing Virtax (with the Engine Pack) on a machine with VirtualBox 7.2 → VirtualBox still works; uninstalling Virtax → VirtualBox still works (automated on 3 OSes).

### Requirement: Uninstall and legacy driver cleanup (FR-PL02-011)
Virtax SHALL provide "Uninstall and legacy driver cleanup" as specified by FR-PL02-011 (BRD-12, priority Should, phase P2; Vietnamese: "Gỡ cài đặt và dọn dẹp driver cũ").

#### Scenario: Specified behaviour of FR-PL02-011
- **WHEN** the behaviour FR-PL02-011 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Uninstalling Virtax removes binaries, services and the helper but never deletes VM data. A "clean up VirtualBox drivers" wizard (VBoxSup, NetLwf, USBMon, kexts, modules) for users fully on `virtax-vmm` lists exactly what it found and removes only after the user clicks to confirm.
- **AND** the VirtualBox reference for the current behaviour is `Installer/win/VirtualBox.wxs:423,426; analysis 05 §4.3`

#### Scenario: Acceptance for FR-PL02-011
- **WHEN** the FR-PL02-011 acceptance check is run
- **THEN** After uninstall, VM folders are intact; the wizard lists only components that exist and does nothing without user confirmation.

### Requirement: Distribution through common package stores (FR-PL02-012)
Virtax SHALL provide "Distribution through common package stores" as specified by FR-PL02-012 (BRD-12, priority Could, phase P1; Vietnamese: "Phân phối qua kho gói phổ biến").

#### Scenario: Specified behaviour of FR-PL02-012
- **WHEN** the behaviour FR-PL02-012 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Publish a winget manifest, a Homebrew cask, APT/DNF repos and Flathub on release day; scripts open PRs to those stores automatically.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-PL02-012
- **WHEN** the FR-PL02-012 acceptance check is run
- **THEN** `winget install Virtax` and `brew install --cask virtax` install the new release within 48 h.
