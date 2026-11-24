# Plugins & extensions Specification

## Purpose
Out-of-process gRPC plugins (remote display, cloud) with signing and permission manifests. Module `CP-13` (Plugins & extensions; Vietnamese: "Plugin & mở rộng") from BRD-11, phase P2, priority Could. Owner crates: virtaxd (plugin host). Depends on: `control-plane/control-plane-api-and-ipc`, `control-plane/security-and-encryption`. VirtualBox reference: `src/VBox/Main/src-all/ExtPackManagerImpl.cpp, src/VBox/ExtPacks`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Out-of-process plugin host over gRPC (FR-CP13-001)
Virtax SHALL provide "Out-of-process plugin host over gRPC" as specified by FR-CP13-001 (BRD-11, priority Must, phase P2; Vietnamese: "Plugin host ngoài process qua gRPC").

#### Scenario: Specified behaviour of FR-CP13-001
- **WHEN** the behaviour FR-CP13-001 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Plugins are separate processes launched by `virtaxd`, speaking gRPC (`plugin.proto`) over a private UDS/pipe; crashes are isolated, restarts back off, resources are capped with cgroups/job objects. No plugin loads into `virtaxd` or `virtax-vmm`.
- **AND** the VirtualBox reference for the current behaviour is `ExtPackManagerImpl.cpp:884-1109`

#### Scenario: Acceptance for FR-CP13-001
- **WHEN** the FR-CP13-001 acceptance check is run
- **THEN** kill -9 of a plugin → no impact on the daemon or VMs, the plugin restarts within 5 s; idle plugins use ≤ 0.1% CPU.

### Requirement: Signed plugin packages with a manifest (FR-CP13-002)
Virtax SHALL provide "Signed plugin packages with a manifest" as specified by FR-CP13-002 (BRD-11, priority Must, phase P2; Vietnamese: "Gói plugin có chữ ký và manifest").

#### Scenario: Specified behaviour of FR-CP13-002
- **WHEN** the behaviour FR-CP13-002 covers is exercised
- **THEN** it behaves as BRD-11 specifies: An `.virtax-plugin` package holds `manifest.toml` (id, version, publisher, per-target binaries, scopes, minimum Virtax version) and a minisign signature, verified at install and at every load. Unsigned plugins run only in developer mode with a persistent badge.
- **AND** the VirtualBox reference for the current behaviour is `ExtPackUtil.cpp:837-840`

#### Scenario: Acceptance for FR-CP13-002
- **WHEN** the FR-CP13-002 acceptance check is run
- **THEN** Tampering with one binary in the package → refused with VTX-PLG-002; developer mode shows a status-bar badge.

### Requirement: Permission scopes and user consent (FR-CP13-003)
Virtax SHALL provide "Permission scopes and user consent" as specified by FR-CP13-003 (BRD-11, priority Must, phase P2; Vietnamese: "Permission scope và đồng ý của người dùng").

#### Scenario: Specified behaviour of FR-CP13-003
- **WHEN** the behaviour FR-CP13-003 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Plugins declare scopes (`machines.read`, `machines.write`, `display.read:<vm>`, `input.write:<vm>`, `net.listen:<port>`, `net.outbound:<host>`, `files.read:<path>`); a bilingual consent dialog at install; `virtaxd` issues scoped capability tokens, enforces them at run time and can revoke them.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP13-003
- **WHEN** the FR-CP13-003 acceptance check is run
- **THEN** A plugin calling outside its scope → PERMISSION_DENIED + audit; revoking a scope takes effect within 1 s.

### Requirement: Remote-display plugin (VNC, SPICE, RDP) (FR-CP13-004)
Virtax SHALL provide "Remote-display plugin (VNC, SPICE, RDP)" as specified by FR-CP13-004 (BRD-11, priority Should, phase P2; Vietnamese: "Plugin remote display (VNC, SPICE, RDP)").

#### Scenario: Specified behaviour of FR-CP13-004
- **WHEN** the behaviour FR-CP13-004 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Replaces VRDE: the plugin reads the display ring with a `display.read` token and injects input with `input.write`. VNC (RFB 3.8, VeNCrypt/TLS + password) first, SPICE/RDP later; listens on localhost only by default, authentication mandatory.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/ExtPacks/VNC; Main IVRDEServer (S-M3)`

#### Scenario: Acceptance for FR-CP13-004
- **WHEN** the FR-CP13-004 acceptance check is run
- **THEN** A VNC client connects over TLS + password; 1080p30 uses ≤ 15% of a core; binding beyond localhost is refused unless explicitly enabled.

### Requirement: Cloud-provider plugins (FR-CP13-005)
Virtax SHALL provide "Cloud-provider plugins" as specified by FR-CP13-005 (BRD-11, priority Could, phase P3; Vietnamese: "Plugin cloud provider").

#### Scenario: Specified behaviour of FR-CP13-005
- **WHEN** the behaviour FR-CP13-005 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Cloud providers (OCI, AWS…) are plugins implementing a `CloudProvider` service (list images, create, start/stop, console); the core contains no cloud SDK. Replaces the 13 `ICloud*` interfaces and screens M-21, W-06, W-07.
- **AND** the VirtualBox reference for the current behaviour is `Main/idl/VirtualBox.xidl (ICloud*, 78 methods)`

#### Scenario: Acceptance for FR-CP13-005
- **WHEN** the FR-CP13-005 acceptance check is run
- **THEN** A reference plugin lists/creates instances on a mock API; the core binary has 0 cloud-SDK dependencies (checked with cargo tree).

### Requirement: Plugin lifecycle and Plugins page (FR-CP13-006)
Virtax SHALL provide "Plugin lifecycle and Plugins page" as specified by FR-CP13-006 (BRD-11, priority Should, phase P2; Vietnamese: "Vòng đời plugin và màn Plugins").

#### Scenario: Specified behaviour of FR-CP13-006
- **WHEN** the behaviour FR-CP13-006 covers is exercised
- **THEN** it behaves as BRD-11 specifies: The Plugins page (UI-07, BRD-06): install from file/URL, enable/disable, update via the CP-14 manifest, uninstall, view scopes and logs; installs into a per-user folder with no admin rights (unlike the elevated `VBoxExtPackHelperApp`).
- **AND** the VirtualBox reference for the current behaviour is `UIExtensionPackManager.cpp (M-18); ExtPackManagerImpl.cpp:2677`

#### Scenario: Acceptance for FR-CP13-006
- **WHEN** the FR-CP13-006 acceptance check is run
- **THEN** Plugins install/uninstall without admin rights in ≤ 3 clicks; disabling takes effect within 1 s.

### Requirement: Plugin API versioning (FR-CP13-007)
Virtax SHALL provide "Plugin API versioning" as specified by FR-CP13-007 (BRD-11, priority Must, phase P2; Vietnamese: "Version hoá API plugin").

#### Scenario: Specified behaviour of FR-CP13-007
- **WHEN** the behaviour FR-CP13-007 covers is exercised
- **THEN** it behaves as BRD-11 specifies: `plugin.proto` follows semver with a capability-negotiating handshake; Virtax supports majors N and N-1; deprecations are announced at least one minor before removal.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-CP13-007
- **WHEN** the FR-CP13-007 acceptance check is run
- **THEN** A plugin built against API 1.x runs on Virtax with API 1.(x+2); an incompatible major → the plugin is disabled with a clear message.

### Requirement: List BYO-engine Extension Packs (preparatory, read-only) (FR-CP13-008)
Virtax SHALL provide "List BYO-engine Extension Packs (preparatory, read-only)" as specified by FR-CP13-008 (BRD-11, priority Should, phase P1; Vietnamese: "Liệt kê Extension Pack của BYO engine (chuẩn bị, chỉ đọc)").

#### Scenario: Specified behaviour of FR-CP13-008
- **WHEN** the behaviour FR-CP13-008 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Phase-1 preparation: with a BYO engine, show installed Extension Packs (name, version, usable, license) read-only, noting that PUEL terms apply to that VirtualBox install. Virtax never installs, redistributes or loads the Oracle Extension Pack.
- **AND** the VirtualBox reference for the current behaviour is `ExtPackManagerImpl.cpp; ExtPackUtil.cpp:239-268`

#### Scenario: Acceptance for FR-CP13-008
- **WHEN** the FR-CP13-008 acceptance check is run
- **THEN** The list matches `VBoxManage list extpacks`; every Virtax artifact SBOM contains 0 Oracle Extension Pack files.

### Requirement: Extension Pack feature replacement map (FR-CP13-009)
Virtax SHALL provide "Extension Pack feature replacement map" as specified by FR-CP13-009 (BRD-11, priority Must, phase P2; Vietnamese: "Bản đồ thay thế tính năng Extension Pack").

#### Scenario: Specified behaviour of FR-CP13-009
- **WHEN** the behaviour FR-CP13-009 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Every feature that lived in the Extension Pack has an open equivalent in the core or a plugin: USB 2/3 (xHCI, core), disk encryption (CP-12), VRDP → VNC/RDP plugin, PXE → iPXE in OVMF, webcam/smart card → CP-08, cloud → plugin (table in the additional content).
- **AND** the VirtualBox reference for the current behaviour is `ExtPackManagerImpl.cpp:1060-1109; Main/src-server/VirtualBoxImpl.cpp:6668`

#### Scenario: Acceptance for FR-CP13-009
- **WHEN** the FR-CP13-009 acceptance check is run
- **THEN** The import report for a VM using ExtPack features names each Virtax equivalent; 0 features lost silently.

### Requirement: Plugin SDK and sandbox (FR-CP13-010)
Virtax SHALL provide "Plugin SDK and sandbox" as specified by FR-CP13-010 (BRD-11, priority Could, phase P2; Vietnamese: "SDK plugin và sandbox").

#### Scenario: Specified behaviour of FR-CP13-010
- **WHEN** the behaviour FR-CP13-010 covers is exercised
- **THEN** it behaves as BRD-11 specifies: An `virtax-plugin-sdk` crate + template + sample plugin; plugins run sandboxed like `virtax-vmm` (seccomp/landlock, App Sandbox, AppContainer) with only their granted scopes; a conformance test suite.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/ExtPacks/Skeleton`

#### Scenario: Acceptance for FR-CP13-010
- **WHEN** the FR-CP13-010 acceptance check is run
- **THEN** A new developer creates a plugin from the template and passes conformance within 30 minutes; the sandbox blocks file reads outside scope.
