# Updates & release channel Specification

## Purpose
Stable/beta/nightly channels, minisign-signed manifests, delta updates, no driver uninstall. Module `CP-14` (Updates & release channel; Vietnamese: "Cập nhật & kênh phát hành") from BRD-11, phase P1, priority Should. Owner crates: virtax-updater. Depends on: `platform/packaging-signing-and-installers`. VirtualBox reference: `src/VBox/Main/src-server/UpdateAgentImpl.cpp`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Stable, beta and nightly channels (FR-CP14-001)
Virtax SHALL provide "Stable, beta and nightly channels" as specified by FR-CP14-001 (BRD-11, priority Must, phase P1; Vietnamese: "Kênh phát hành stable, beta, nightly").

#### Scenario: Specified behaviour of FR-CP14-001
- **WHEN** the behaviour FR-CP14-001 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Users pick a channel in Preferences › Updates (stable by default); nightly carries a warning and an "unstable" badge; enterprises can lock the channel. Replaces VBox's Stable/All/WithBetas (Testing never implemented).
- **AND** the VirtualBox reference for the current behaviour is `UpdateAgentImpl.cpp:982-987,1097-1104; UIGlobalSettingsUpdate.cpp`

#### Scenario: Acceptance for FR-CP14-001
- **WHEN** the FR-CP14-001 acceptance check is run
- **THEN** A channel change applies at the next check within 1 minute; nightly builds show a badge in About and the status bar.

### Requirement: minisign-signed update manifest (FR-CP14-002)
Virtax SHALL provide "minisign-signed update manifest" as specified by FR-CP14-002 (BRD-11, priority Must, phase P1; Vietnamese: "Manifest cập nhật có chữ ký minisign").

#### Scenario: Specified behaviour of FR-CP14-002
- **WHEN** the behaviour FR-CP14-002 covers is exercised
- **THEN** it behaves as BRD-11 specifies: A JSON manifest per channel/platform signed with minisign (ed25519) by an online key certified by an offline root, carrying version, `min_from`, artifacts (URL, sha256, size, signature), deltas, VI/EN release notes, severity and expiry. Public keys are embedded; replay is blocked by monotonic versions.
- **AND** the VirtualBox reference for the current behaviour is `UpdateAgentImpl.cpp:1186-1260`

#### Scenario: Acceptance for FR-CP14-002
- **WHEN** the FR-CP14-002 acceptance check is run
- **THEN** A tampered manifest or artifact → VTX-UPD-002, nothing installed; a manifest expired > 14 days → warning; a replayed older manifest → rejected.

### Requirement: Privacy-preserving background check (FR-CP14-003)
Virtax SHALL provide "Privacy-preserving background check" as specified by FR-CP14-003 (BRD-11, priority Must, phase P1; Vietnamese: "Kiểm tra nền, tôn trọng quyền riêng tư").

#### Scenario: Specified behaviour of FR-CP14-003
- **WHEN** the behaviour FR-CP14-003 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Checks at most once per 24 h with jitter, only on unmetered networks and (optionally) AC power; requests carry only version/OS/arch, no machine ID; fully disableable (FR-GEN-008); system or manual proxy honoured.
- **AND** the VirtualBox reference for the current behaviour is `UpdateAgentImpl.cpp:894-902,1061`

#### Scenario: Acceptance for FR-CP14-003
- **WHEN** the FR-CP14-003 acceptance check is run
- **THEN** Egress capture: one GET to the manifest URL with no identifiers; updates off → 0 requests; checking adds ≤ 1 wakeup per hour.

### Requirement: Background download and staged apply (FR-CP14-004)
Virtax SHALL provide "Background download and staged apply" as specified by FR-CP14-004 (BRD-11, priority Must, phase P1; Vietnamese: "Tải nền và áp dụng theo giai đoạn").

#### Scenario: Specified behaviour of FR-CP14-004
- **WHEN** the behaviour FR-CP14-004 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Background download with resume (HTTP range) and a bandwidth cap, sha256 + signature verified, then staged. Applied when the user clicks "Restart to update" or when no VM runs; running VMs continue and new binaries are used on the next VM start.
- **AND** the VirtualBox reference for the current behaviour is `UpdateAgentImpl.cpp:457-468; debian/preinst:65-80`

#### Scenario: Acceptance for FR-CP14-004
- **WHEN** the FR-CP14-004 acceptance check is run
- **THEN** Updating with 3 VMs running → 0 VMs stopped; Manager restarts within 3 s; an interrupted download resumes.

### Requirement: Delta updates (FR-CP14-005)
Virtax SHALL provide "Delta updates" as specified by FR-CP14-005 (BRD-11, priority Should, phase P1; Vietnamese: "Delta update").

#### Scenario: Specified behaviour of FR-CP14-005
- **WHEN** the behaviour FR-CP14-005 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Binary patches (zstd `--patch-from` or bsdiff) from N-1…N-3 to N per platform; a patch failure falls back to the full download; the result is verified by the final hash.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP14-005
- **WHEN** the FR-CP14-005 acceptance check is run
- **THEN** A typical patch release ships deltas ≤ 20% of the full size; 100% of applied patches are hash-verified.

### Requirement: No driver uninstall/reinstall on upgrade (FR-CP14-006)
Virtax SHALL provide "No driver uninstall/reinstall on upgrade" as specified by FR-CP14-006 (BRD-11, priority Must, phase P1; Vietnamese: "Không gỡ/cài lại driver khi nâng cấp").

#### Scenario: Specified behaviour of FR-CP14-006
- **WHEN** the behaviour FR-CP14-006 covers is exercised
- **THEN** it behaves as BRD-11 specifies: The updater only replaces Virtax files; it never touches VBoxSup/vboxdrv or VirtualBox network adapters; elevation is requested only when the per-machine part (helper) changes. Packaging mechanics are in BRD-12 (PL-02).
- **AND** the VirtualBox reference for the current behaviour is `VirtualBox.wxs:417; VBoxInstallHelper.cpp:2092-2095,2167`

#### Scenario: Acceptance for FR-CP14-006
- **WHEN** the FR-CP14-006 acceptance check is run
- **THEN** A Windows upgrade takes ≤ 2 minutes with no reboot prompt and no INetCfg lock; existing VirtualBox drivers keep their version.

### Requirement: Automatic and manual rollback (FR-CP14-007)
Virtax SHALL provide "Automatic and manual rollback" as specified by FR-CP14-007 (BRD-11, priority Must, phase P1; Vietnamese: "Rollback tự động và thủ công").

#### Scenario: Specified behaviour of FR-CP14-007
- **WHEN** the behaviour FR-CP14-007 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Keep N-1 side by side; roll back automatically if the new build fails its health check (daemon fails 3× or crashes in the first 60 s); a "Roll back to x.y" button in Preferences; registry backup before schema migrations.
- **AND** the VirtualBox reference for the current behaviour is `Installer/win/VirtualBox.wxs:124`

#### Scenario: Acceptance for FR-CP14-007
- **WHEN** the FR-CP14-007 acceptance check is run
- **THEN** Installing a build with an injected crash → automatic rollback within 30 s with a notice; the registry is restored; VMs are intact.

### Requirement: Enterprise policy (FR-CP14-008)
Virtax SHALL provide "Enterprise policy" as specified by FR-CP14-008 (BRD-11, priority Must, phase P1; Vietnamese: "Policy doanh nghiệp").

#### Scenario: Specified behaviour of FR-CP14-008
- **WHEN** the behaviour FR-CP14-008 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Policies to disable updates, lock the channel, set the manifest URL (mirror) and disable plugins/telemetry via Group Policy (ADMX, `HKLM\Software\Policies\Virtax`), an MDM configuration profile on macOS and `/etc/virtax/policy.toml` on Linux; the UI shows "managed by your organization".
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP14-008
- **WHEN** the FR-CP14-008 acceptance check is run
- **THEN** With the policy set, the Updates section is locked with an explanation; `virtaxctl update check` returns VTX-UPD-010 "disabled by policy".

### Requirement: Platform-native update integration (FR-CP14-009)
Virtax SHALL provide "Platform-native update integration" as specified by FR-CP14-009 (BRD-11, priority Should, phase P1; Vietnamese: "Tích hợp kênh cập nhật của từng nền tảng").

#### Scenario: Specified behaviour of FR-CP14-009
- **WHEN** the behaviour FR-CP14-009 covers is exercised
- **THEN** it behaves as BRD-11 specifies: MSIX/winget on Windows; a self-updating notarised .app (with deltas) on macOS; on Linux the package manager owns updates (GPG-signed APT/DNF, Flatpak, AppImage zsync) — .deb/.rpm builds only notify and never self-replace.
- **AND** the VirtualBox reference for the current behaviour is `Installer/linux/*; Frontends/VirtualBox/src/networking/UINewVersionChecker.cpp`

#### Scenario: Acceptance for FR-CP14-009
- **WHEN** the FR-CP14-009 acceptance check is run
- **THEN** A .deb install shows "update via Software Updater/apt" and never overwrites itself; the AppImage self-updates with zsync.

### Requirement: Engine Pack and component updates (FR-CP14-010)
Virtax SHALL provide "Engine Pack and component updates" as specified by FR-CP14-010 (BRD-11, priority Should, phase P1; Vietnamese: "Cập nhật Engine Pack và thành phần").

#### Scenario: Specified behaviour of FR-CP14-010
- **WHEN** the behaviour FR-CP14-010 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Engine Pack, OVMF/AAVMF firmware, the virtio-win ISO and `virtax-agent` are separate manifest components updated independently. For a BYO engine: only notify when a VirtualBox 7.2.x security release exists (official link), never auto-install.
- **AND** the VirtualBox reference for the current behaviour is `UpdateAgentImpl.cpp:992`

#### Scenario: Acceptance for FR-CP14-010
- **WHEN** the FR-CP14-010 acceptance check is run
- **THEN** The Engine Pack applies only when no legacy VM runs; the new agent ISO appears in the ISO library; BYO shows the version gap.

### Requirement: Update UX in the Task center (FR-CP14-011)
Virtax SHALL provide "Update UX in the Task center" as specified by FR-CP14-011 (BRD-11, priority Should, phase P1; Vietnamese: "UX cập nhật trong Task center").

#### Scenario: Specified behaviour of FR-CP14-011
- **WHEN** the behaviour FR-CP14-011 covers is exercised
- **THEN** it behaves as BRD-11 specifies: A new version → a non-modal notice and a Task center item with bilingual release notes, size, a "security" badge, progress %, ETA and cancel; never modal over the Console.
- **AND** the VirtualBox reference for the current behaviour is `UINewVersionChecker.cpp; notificationcenter/UINotificationMessage.cpp`

#### Scenario: Acceptance for FR-CP14-011
- **WHEN** the FR-CP14-011 acceptance check is run
- **THEN** The update flow takes ≤ 2 clicks; the UI thread stays ≤ 50 ms; screen readers announce the notice.
