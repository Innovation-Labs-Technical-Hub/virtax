# Global tools (Media, Network, Plugins, Preferences) Specification

## Purpose
Disk/ISO manager (chains, usage, convert), host-only/NAT networks, plugins, global preferences. Module `UI-07` (Global tools (Media, Network, Plugins, Preferences); Vietnamese: "Công cụ toàn cục (Media, Network, Plugins, Preferences)") from BRD-06, phase P1, priority Should. Owner crates: virtax. Depends on: `control-plane/storage-and-media-management`, `control-plane/network-management`, `control-plane/plugins-and-extensions`. VirtualBox reference: `src/VBox/Frontends/VirtualBox/src/medium, networkmanager, extensionpackmanager, settings/global`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Media manager screen (FR-UI07-001)
Virtax SHALL provide "Media manager screen" as specified by FR-UI07-001 (BRD-06, priority Must, phase P1; Vietnamese: "Màn hình Media manager").

#### Scenario: Specified behaviour of FR-UI07-001
- **WHEN** the behaviour FR-UI07-001 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Replaces M-19: Virtual disks / ISO library / Storage usage tabs; Disks/Optical/Floppy filter chips; a Name, Attached to, Format, Virtual, On disk, Status table (OK, Multi-attach, Not attached, File missing badges); an always-visible filter field; a virtualized list; diffs/overlays nested under their base (↳, snapshot name, own size, expand/collapse with ←/→), orphan diffs with an amber badge; a 'Metadata cached in registry' caption.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/medium/UIMediumManager.cpp, UIMediumSearchWidget.cpp`

#### Scenario: Acceptance for FR-UI07-001
- **WHEN** the FR-UI07-001 acceptance check is run
- **THEN** Opening Media with 500 media takes ≤ 300 ms p95; scrolling 1,000 rows holds 60 fps; every action is keyboard operable with screen-reader labels.

### Requirement: Detail pane with Operation-based actions (FR-UI07-002)
Virtax SHALL provide "Detail pane with Operation-based actions" as specified by FR-UI07-002 (BRD-06, priority Must, phase P1; Vietnamese: "Pane chi tiết và hành động dạng Operation").

#### Scenario: Specified behaviour of FR-UI07-002
- **WHEN** the behaviour FR-UI07-002 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Selecting a disk opens a right pane: location (mono), format/variant, virtual/on-disk, reclaimable, encryption, chain; Convert, Compact/reclaim, Resize, Clone, Move and Remove buttons. Each starts an Operation shown in the Task center; disabled buttons always have a reason tooltip (e.g. 'attached to a running VM').
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/medium/UIMediumDetailsWidget.cpp; notificationcenter/UINotificationProgress*`

#### Scenario: Acceptance for FR-UI07-002
- **WHEN** the FR-UI07-002 acceptance check is run
- **THEN** Clicking Compact shows a 'Started' toast within ≤ 100 ms and the Task center shows % and ETA; the UI thread is never held > 50 ms during the Operation.

### Requirement: One-screen create-disk sheet (FR-UI07-003)
Virtax SHALL provide "One-screen create-disk sheet" as specified by FR-UI07-003 (BRD-06, priority Must, phase P1; Vietnamese: "Sheet tạo disk một màn hình").

#### Scenario: Specified behaviour of FR-UI07-003
- **WHEN** the behaviour FR-UI07-003 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Replaces wizard W-05 (3 pages + Expert): one sheet with the format preselected by engine, a size field with units and slider, location (VM folder by default), dynamic/fixed and a volume free-space indicator. Enter = Create; errors appear next to the field.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/wizards/newvd/*, wizards/editors/UIWizardDiskEditors.cpp; UINativeWizard.cpp:99`

#### Scenario: Acceptance for FR-UI07-003
- **WHEN** the FR-UI07-003 acceptance check is run
- **THEN** Creating a 64 GB disk with defaults takes only typing the size + Enter (≤ 2 interactions); '64gb' and '64 GiB' are both parsed correctly.

### Requirement: Medium picker without nested modals (FR-UI07-004)
Virtax SHALL provide "Medium picker without nested modals" as specified by FR-UI07-004 (BRD-06, priority Must, phase P1; Vietnamese: "Medium picker không lồng modal").

#### Scenario: Specified behaviour of FR-UI07-004
- **WHEN** the behaviour FR-UI07-004 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Replaces the Medium Selector (W-08) when attaching a disk/ISO in Settings: a searchable popover with recent items, the ISO library, and 'Create new…' and 'Browse…' inline; at most 2 levels (sheet → popover), never a stacked wizard.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/medium/UIMediumSelector.cpp; analysis 03 §2.2 D3`

#### Scenario: Acceptance for FR-UI07-004
- **WHEN** the FR-UI07-004 acceptance check is run
- **THEN** Attaching an ISO to a VM from Settings takes ≤ 3 interactions; the e2e test measures a maximum modal depth of 2.

### Requirement: Storage usage tab (FR-UI07-005)
Virtax SHALL provide "Storage usage tab" as specified by FR-UI07-005 (BRD-06, priority Should, phase P1; Vietnamese: "Tab Storage usage").

#### Scenario: Specified behaviour of FR-UI07-005
- **WHEN** the behaviour FR-UI07-005 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Horizontal bar chart per VM (base, snapshots, ISOs) with an equivalent table for screen readers; free space per host volume; a total 'reclaimable' figure; the top 5 consumers with action buttons. Never colour alone: each segment carries a label and value.
- **AND** the VirtualBox reference for the current behaviour is `new (no equivalent in VBox)`

#### Scenario: Acceptance for FR-UI07-005
- **WHEN** the FR-UI07-005 acceptance check is run
- **THEN** Figures match host `du` within ≤ 1%; the tab renders from cache within ≤ 500 ms and refreshes in the background when recomputed.

### Requirement: ISO library tab (FR-UI07-006)
Virtax SHALL provide "ISO library tab" as specified by FR-UI07-006 (BRD-06, priority Should, phase P1; Vietnamese: "Tab ISO library").

#### Scenario: Specified behaviour of FR-UI07-006
- **WHEN** the behaviour FR-UI07-006 covers is exercised
- **THEN** it behaves as BRD-06 specifies: ISO list with OS logo, name/version, architecture badge (mismatches struck through), size, checksum (Verified/Unknown/Mismatch) and VMs using it. Drag-and-drop to add; 'Create VM from ISO' opens a prefilled Quick Create (UI-03); 'Eject everywhere' detaches the ISO from all powered-off VMs. 'Create ISO from folder…' (replacing the VISO Creator W-09, Phase 2) builds a read-only ISO 9660/Joliet image to bring files into agent-less guests; the floppy creator (W-10) is not built.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/medium/UIMediumManager.cpp (Optical tab); medium/viso/UIVisoCreator.cpp; medium/UIFDCreationDialog.cpp`

#### Scenario: Acceptance for FR-UI07-006
- **WHEN** the FR-UI07-006 acceptance check is run
- **THEN** Dropping an Ubuntu ISO adds a row within ≤ 200 ms and identifies the OS within ≤ 10 s; 'Create VM from ISO' opens Quick Create with the OS preselected.

### Requirement: Plugins screen (FR-UI07-007)
Virtax SHALL provide "Plugins screen" as specified by FR-UI07-007 (BRD-06, priority Should, phase P1; Vietnamese: "Màn hình Plugins").

#### Scenario: Specified behaviour of FR-UI07-007
- **WHEN** the behaviour FR-UI07-007 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Replaces the Extensions tool (M-18). Phase 1 shows a read-only list of extension packs installed in the legacy engine (name, version, usable) and states that Virtax does not install PUEL packs itself. Phase 2 shows Virtax plugins (CP-13): publisher, signature, permission manifest, enable/disable, install from file with signature verification.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/extensionpackmanager/UIExtensionPackManager.cpp; Main/src-all/ExtPackManagerImpl.cpp`

#### Scenario: Acceptance for FR-UI07-007
- **WHEN** the FR-UI07-007 acceptance check is run
- **THEN** Phase 1: a host with an extension pack shows its name/version and no Install button; Phase 2: a plugin with a bad signature is rejected with VTX-PLG-001.

### Requirement: Searchable Global Preferences shell (FR-UI07-008)
Virtax SHALL provide "Searchable Global Preferences shell" as specified by FR-UI07-008 (BRD-06, priority Must, phase P1; Vietnamese: "Khung Global Preferences tìm kiếm được").

#### Scenario: Specified behaviour of FR-UI07-008
- **WHEN** the behaviour FR-UI07-008 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Replaces S-G0: one workspace screen (not an OK/Cancel dialog) with General, Input, Update, Language, Display, Appearance, Network proxy and Engine & health; search is the primary entry; changes apply instantly with a 10 s Undo; deep links `virtax://prefs/<section>`. No Basic/Expert mode. Preferences live in virtaxd's `ui_prefs` and are never written as `GUI/*` keys into VirtualBox.xml/.vbox; TOML Export/Import is provided.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/settings/UIAdvancedSettingsDialog.cpp:1035-1043; UIAdvancedSettingsDialogSpecific.cpp:200-275; extradata/UIExtraDataDefs.cpp (151 GUI/* keys)`

#### Scenario: Acceptance for FR-UI07-008
- **WHEN** the FR-UI07-008 acceptance check is run
- **THEN** Typing 'host key' jumps to the field within ≤ 100 ms; theme changes apply within ≤ 200 ms; Undo restores the previous value; changing 10 preferences leaves a 0-line VirtualBox.xml diff.

### Requirement: Preferences — General & storage defaults (FR-UI07-009)
Virtax SHALL provide "Preferences — General & storage defaults" as specified by FR-UI07-009 (BRD-06, priority Must, phase P1; Vietnamese: "Preferences — General & mặc định storage").

#### Scenario: Specified behaviour of FR-UI07-009
- **WHEN** the behaviour FR-UI07-009 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Replaces S-G1: default VM folder, default disk format per engine, default cache mode, ISO library folders, an 'orphan diff cleanup' policy. Inline validation: folder writable, ≥ 20 GB free, not on a network drive (warning). The VRDP Authentication Library field is dropped.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/settings/global/UIGlobalSettingsGeneral.cpp`

#### Scenario: Acceptance for FR-UI07-009
- **WHEN** the FR-UI07-009 acceptance check is run
- **THEN** Choosing a read-only folder shows an error next to the field with a suggestion; changing the default format makes the next Quick Create use it.

### Requirement: Preferences — Input and shortcuts (FR-UI07-010)
Virtax SHALL provide "Preferences — Input and shortcuts" as specified by FR-UI07-010 (BRD-06, priority Must, phase P1; Vietnamese: "Preferences — Input và phím tắt").

#### Scenario: Specified behaviour of FR-UI07-010
- **WHEN** the behaviour FR-UI07-010 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Replaces S-G2: Host key with a 'Record key' button, searchable Manager/Console shortcut tables, conflict detection (two commands on one combo, or a system shortcut), auto-capture keyboard. Reset to default per item.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/settings/global/UIGlobalSettingsInput.cpp; settings/editors/UIShortcutConfigurationEditor.cpp, UIHostComboEditor.cpp`

#### Scenario: Acceptance for FR-UI07-010
- **WHEN** the FR-UI07-010 acceptance check is run
- **THEN** Assigning Ctrl+K to a second command warns about the Command palette conflict and offers a swap; changes take effect in an open Console within ≤ 1 s.

### Requirement: Preferences — Update, Language, Display, Appearance, Proxy (FR-UI07-011)
Virtax SHALL provide "Preferences — Update, Language, Display, Appearance, Proxy" as specified by FR-UI07-011 (BRD-06, priority Should, phase P1; Vietnamese: "Preferences — Update, Language, Display, Appearance, Proxy").

#### Scenario: Specified behaviour of FR-UI07-011
- **WHEN** the behaviour FR-UI07-011 covers is exercised
- **THEN** it behaves as BRD-06 specifies: Update (S-G3): stable/beta/nightly channel, check interval, can be fully disabled. Language (S-G4): VI/EN switch without restart. Display (S-G5): maximum guest size, scale 'follow host'. Appearance (S-G7): light/dark/high-contrast/system on all 3 OSes (VBox has it on Windows only). Proxy (S-G6): auto/direct/manual.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/settings/global/UIGlobalSettings{Update,Language,Display,Interface,Proxy}.cpp; UIAdvancedSettingsDialogSpecific.cpp:266`

#### Scenario: Acceptance for FR-UI07-011
- **WHEN** the FR-UI07-011 acceptance check is run
- **THEN** Switching to high contrast on Linux and macOS gives ≥ 7:1 contrast for body text on every screen; switching language updates every screen within ≤ 1 s.

### Requirement: Preferences — Engine & health (FR-UI07-012)
Virtax SHALL provide "Preferences — Engine & health" as specified by FR-UI07-012 (BRD-06, priority Should, phase P1; Vietnamese: "Preferences — Engine & health").

#### Scenario: Specified behaviour of FR-UI07-012
- **WHEN** the behaviour FR-UI07-012 covers is exercised
- **THEN** it behaves as BRD-06 specifies: New page: active engine (BYO VirtualBox 7.2.x or Engine Pack, version), virtax-vmm preview, hypervisor status (KVM/HVF/WHP, VT-x conflicts, Secure Boot), power profile (Performance/Balanced/Battery), log paths; a 'Copy diagnostics' button. Every problem has an VTX code and a quick fix.
- **AND** the VirtualBox reference for the current behaviour is `new; Installer/linux/VBox.sh:69-90 (misleading driver warning)`

#### Scenario: Acceptance for FR-UI07-012
- **WHEN** the FR-UI07-012 acceptance check is run
- **THEN** A Linux machine without /dev/kvm access shows VTX-HV-002 with the fix command (add the user to the kvm group) and a copy button.

### Requirement: Single-model Network manager (FR-UI07-050)
Virtax SHALL provide "Single-model Network manager" as specified by FR-UI07-050 (BRD-07, priority Must, phase P1; Vietnamese: "Network manager một mô hình").

#### Scenario: Specified behaviour of FR-UI07-050
- **WHEN** the behaviour FR-UI07-050 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Replaces M-20 (Host-only / NAT / Cloud tabs that differ between macOS and Linux/Windows): one screen with NAT networks, Host-only, Internal, Bridged interfaces (host), Port forwards and DHCP leases sections; identical on 3 OSes; service status and VM count per network.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/networkmanager/UINetworkManager.cpp:54-147, UIDetailsWidget{HostNetwork,NATNetwork}.cpp`

#### Scenario: Acceptance for FR-UI07-050
- **WHEN** the FR-UI07-050 acceptance check is run
- **THEN** The Network manager opens within ≤ 300 ms from cache; the same sections appear on macOS, Linux and Windows; 100% keyboard operable.

### Requirement: Create/edit network sheet with inline validation (FR-UI07-051)
Virtax SHALL provide "Create/edit network sheet with inline validation" as specified by FR-UI07-051 (BRD-07, priority Must, phase P1; Vietnamese: "Sheet tạo/sửa mạng có validate inline").

#### Scenario: Specified behaviour of FR-UI07-051
- **WHEN** the behaviour FR-UI07-051 covers is exercised
- **THEN** it behaves as BRD-07 specifies: A right-hand sheet for NAT networks/host-only/internal: name, CIDR (free range suggested), DHCP on/off and range, IPv6. Immediate checks: CIDR overlapping host routes, VPNs or other Virtax networks; DHCP range outside the subnet; gateway equal to the host IP. Errors sit next to fields with a Fix button.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/networkmanager/UIDetailsWidgetNATNetwork.cpp, UIDetailsWidgetHostNetwork.cpp`

#### Scenario: Acceptance for FR-UI07-051
- **WHEN** the FR-UI07-051 acceptance check is run
- **THEN** Entering 192.168.1.0/24 when the host LAN is 192.168.1.0/24 shows an overlap warning + a Fix proposing 10.10.x.0/24 within ≤ 100 ms.

### Requirement: Global port-forward table (FR-UI07-052)
Virtax SHALL provide "Global port-forward table" as specified by FR-UI07-052 (BRD-07, priority Must, phase P1; Vietnamese: "Bảng port forward toàn cục").

#### Scenario: Specified behaviour of FR-UI07-052
- **WHEN** the behaviour FR-UI07-052 covers is exercised
- **THEN** it behaves as BRD-07 specifies: One table of every rule of every VM and NAT network: VM/network, proto, host IP:port, guest IP:port, status (active/inactive/conflict). Filter by VM; red conflict badges with quick fixes; inline editing without an S-X1 style sub-dialog.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/settings/machine/UIMachineSettingsPortForwardingDlg.cpp; widgets/UIPortForwardingTable.cpp`

#### Scenario: Acceptance for FR-UI07-052
- **WHEN** the FR-UI07-052 acceptance check is run
- **THEN** With 30 rules across 10 VMs, searching 'ssh' filters correctly within ≤ 100 ms; inline port edits save within ≤ 300 ms after a pre-save conflict check.

### Requirement: List-based NIC editor in the Settings sheet (FR-UI07-053)
Virtax SHALL provide "List-based NIC editor in the Settings sheet" as specified by FR-UI07-053 (BRD-07, priority Must, phase P1; Vietnamese: "Editor NIC dạng danh sách trong Settings sheet").

#### Scenario: Specified behaviour of FR-UI07-053
- **WHEN** the behaviour FR-UI07-053 covers is exercised
- **THEN** it behaves as BRD-07 specifies: A component used on the Network page of the Settings sheet (UI-04): each NIC is a card (model, attachment, MAC, cable, forwards) with Add/Remove/drag-to-reorder; Live/Next boot/Power off badges from ApplyMode; attachment choices described in plain words ('Only this computer sees the VM', 'Other LAN machines see the VM').
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/settings/machine/UIMachineSettingsNetwork.cpp:455; settings/editors/UINetworkAttachmentEditor.cpp`

#### Scenario: Acceptance for FR-UI07-053
- **WHEN** the FR-UI07-053 acceptance check is run
- **THEN** Adding a 5th NIC takes ≤ 2 interactions; a NAT card shows a '2222→22' forward summary; the screen reader announces the attachment kind and badge.

### Requirement: Bridged interface picker with status (FR-UI07-054)
Virtax SHALL provide "Bridged interface picker with status" as specified by FR-UI07-054 (BRD-07, priority Should, phase P1; Vietnamese: "Chọn interface bridged có trạng thái").

#### Scenario: Specified behaviour of FR-UI07-054
- **WHEN** the behaviour FR-UI07-054 covers is exercised
- **THEN** it behaves as BRD-07 specifies: The picker shows host interfaces with wired/Wi-Fi/VPN icons, link up/down, IP and speed; virtual interfaces (docker0, vEthernet, utun) are grouped under 'Other'; Wi-Fi carries the caveat label (FR-CP06-006). The list updates live on plug/unplug.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/settings/editors/UINetworkAttachmentEditor.cpp; Main/src-server/HostNetworkInterfaceImpl.cpp`

#### Scenario: Acceptance for FR-UI07-054
- **WHEN** the FR-UI07-054 acceptance check is run
- **THEN** Plugging in USB-Ethernet makes it appear in the picker within ≤ 2 s; docker0 is not in the main group.

### Requirement: DHCP and static lease editor (FR-UI07-055)
Virtax SHALL provide "DHCP and static lease editor" as specified by FR-UI07-055 (BRD-07, priority Should, phase P1; Vietnamese: "Editor DHCP và lease tĩnh").

#### Scenario: Specified behaviour of FR-UI07-055
- **WHEN** the behaviour FR-UI07-055 covers is exercised
- **THEN** it behaves as BRD-07 specifies: In each network's details: range, lease time, options; a current lease table (MAC, IP, VM, expiry); 'Keep this IP' turns a dynamic lease static in one click; static leases pick a VM from a list instead of typing a MAC.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/DHCPServerImpl.cpp; Main/idl/VirtualBox.xidl (IDHCPIndividualConfig)`

#### Scenario: Acceptance for FR-UI07-055
- **WHEN** the FR-UI07-055 acceptance check is run
- **THEN** Clicking 'Keep this IP' for VM 'db' creates a static lease that survives a guest reboot; no MAC typing is ever needed.

### Requirement: Network chips and quick actions (FR-UI07-056)
Virtax SHALL provide "Network chips and quick actions" as specified by FR-UI07-056 (BRD-07, priority Should, phase P1; Vietnamese: "Chip mạng và quick action").

#### Scenario: Specified behaviour of FR-UI07-056
- **WHEN** the behaviour FR-UI07-056 covers is exercised
- **THEN** it behaves as BRD-07 specifies: The Network card on Overview (MK-01) and the Console network chip show 'NIC 1 · virtio-net · NAT · 2222→22'; quick actions: Copy SSH command, open the forwarded URL, unplug/plug the cable, switch network, open Test connectivity.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/runtime/UIIndicatorsPool.cpp (network indicator); manager/details/UIDetailsElement.cpp`

#### Scenario: Acceptance for FR-UI07-056
- **WHEN** the FR-UI07-056 acceptance check is run
- **THEN** 'Copy SSH command' yields `ssh -p 2222 user@localhost` with the guest user when known in 1 interaction; unplugging the cable from the chip makes the guest see link down within ≤ 1 s.

### Requirement: Network errors with VTX-NET codes and quick fixes (FR-UI07-057)
Virtax SHALL provide "Network errors with VTX-NET codes and quick fixes" as specified by FR-UI07-057 (BRD-07, priority Must, phase P1; Vietnamese: "Lỗi mạng có mã VTX-NET và quick-fix").

#### Scenario: Specified behaviour of FR-UI07-057
- **WHEN** the behaviour FR-UI07-057 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Common failures are mapped: missing host-only backend (BYO without vboxnetadp), no /dev/net/tun access, missing vmnet entitlement, port in use, vanished bridged interface, overlapping CIDR. Each has a bilingual explanation, a fix button (e.g. the helper grants TAP access) and 'Copy technical details' with the original VERR_*.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/ConsoleImplConfigCommon.cpp:2420-2433; Devices/Network/DrvNAT.cpp:1845-1847; Frontends/VirtualBox/src/globals/UIErrorString.cpp`

#### Scenario: Acceptance for FR-UI07-057
- **WHEN** the FR-UI07-057 acceptance check is run
- **THEN** Simulating missing /dev/net/tun access shows VTX-NET-030 with a 'Grant via helper' button instead of the 'chmod 0666' text; after clicking, the VM starts.

### Requirement: VM ↔ network map (FR-UI07-058)
Virtax SHALL provide "VM ↔ network map" as specified by FR-UI07-058 (BRD-07, priority Could, phase P1; Vietnamese: "Sơ đồ VM ↔ mạng").

#### Scenario: Specified behaviour of FR-UI07-058
- **WHEN** the behaviour FR-UI07-058 covers is exercised
- **THEN** it behaves as BRD-07 specifies: An optional view: each network is a column with VMs as attached nodes, plus an equivalent table for screen readers; clicking a node opens the VM. Colour is never the only signal (network-kind labels are shown).
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-UI07-058
- **WHEN** the FR-UI07-058 acceptance check is run
- **THEN** A 12-VM lab on 3 networks renders within ≤ 300 ms; the alternative table lists all 12 VMs and their networks.

### Requirement: Network performance hints (FR-UI07-059)
Virtax SHALL provide "Network performance hints" as specified by FR-UI07-059 (BRD-07, priority Could, phase P1; Vietnamese: "Gợi ý hiệu năng mạng").

#### Scenario: Specified behaviour of FR-UI07-059
- **WHEN** the behaviour FR-UI07-059 covers is exercised
- **THEN** it behaves as BRD-07 specifies: When NAT on the legacy engine sustains > 50 MB/s for 60 s, or the NIC is E1000 on a Linux guest, Virtax shows a non-blocking hint: switch to virtio-net, use host-only/bridged for heavy traffic, or 'Move to virtax-vmm' (Phase 2). Hints can be dismissed permanently.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNAT.cpp:749-905; Main/src-all/Global.cpp:215`

#### Scenario: Acceptance for FR-UI07-059
- **WHEN** the FR-UI07-059 acceptance check is run
- **THEN** Downloading a 2 GB file over NAT on a legacy VM shows the hint once, non-modal; 'Don't remind me' stops it for good.
