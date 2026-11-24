# Network management Specification

## Purpose
NAT, NAT network, host-only, bridged, internal; port forwarding with conflict checks; DHCP. Module `CP-06` (Network management; Vietnamese: "Quản lý mạng") from BRD-07, phase P1, priority Must. Owner crates: virtaxd, virtax-helper. Depends on: `platform/host-integration-and-power`. VirtualBox reference: `src/VBox/Main/src-server/NATNetworkImpl.cpp, HostOnlyNetworkImpl.cpp, DHCPServerImpl.cpp, NetworkServices`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: NICs as a dynamic list, changeable while running (FR-CP06-001)
Virtax SHALL provide "NICs as a dynamic list, changeable while running" as specified by FR-CP06-001 (BRD-07, priority Must, phase P1; Vietnamese: "NIC dạng danh sách động, đổi được khi VM chạy").

#### Scenario: Specified behaviour of FR-CP06-001
- **WHEN** the behaviour FR-CP06-001 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Each VM has `[[nics]]` (slot, model, mac, attachment, cable) with no 4-NIC cap: up to the engine maximum (VBox: 8 with PIIX3, 36 with ICH9; virtax-vmm: 16). UpdateMachine returns an ApplyMode per field: attachment and cable changes are Live, model changes need Power off.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/settings/machine/UIMachineSettingsNetwork.cpp:455; Main/src-server/PlatformPropertiesImpl.cpp:230-231; NetworkAdapterImpl.cpp`

#### Scenario: Acceptance for FR-CP06-001
- **WHEN** the FR-CP06-001 acceptance check is run
- **THEN** Given a running VM, When NIC 1 changes from NAT to host-only, Then the guest gets a new DHCP address within ≤ 5 s without reboot; adding a 6th NIC on an ICH9 VM succeeds.

### Requirement: Per-VM NAT with live port forwarding (FR-CP06-002)
Virtax SHALL provide "Per-VM NAT with live port forwarding" as specified by FR-CP06-002 (BRD-07, priority Must, phase P1; Vietnamese: "NAT theo VM và port forwarding live").

#### Scenario: Specified behaviour of FR-CP06-002
- **WHEN** the behaviour FR-CP06-002 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Rules have name, proto (tcp/udp), host_ip, host_port, guest_ip, guest_port, and can be added/removed live on a running VM. New rules bind 127.0.0.1 by default; exposing to the LAN requires explicitly choosing 'All interfaces' with a warning. One-click presets: SSH (22), RDP (3389), HTTP (80/8080).
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/NATEngineImpl.cpp:278-330 (addRedirect); Devices/Network/DrvNAT.cpp:1822-1824 (INADDR_ANY default)`

#### Scenario: Acceptance for FR-CP06-002
- **WHEN** the FR-CP06-002 acceptance check is run
- **THEN** Adding the SSH preset to a running VM makes `ssh -p 2222 localhost` work within ≤ 1 s; host `ss -ltn` shows the rule bound to 127.0.0.1, not 0.0.0.0.

### Requirement: Cross-VM port-forward conflict detection (FR-CP06-003)
Virtax SHALL provide "Cross-VM port-forward conflict detection" as specified by FR-CP06-003 (BRD-07, priority Must, phase P1; Vietnamese: "Phát hiện xung đột port forward giữa các VM").

#### Scenario: Specified behaviour of FR-CP06-003
- **WHEN** the behaviour FR-CP06-003 covers is exercised
- **THEN** it behaves as BRD-07 specifies: virtaxd keeps a global (proto, host_ip, host_port) index of every rule of every VM and NAT network, including powered-off VMs, and probes the host with a bind to detect other listening services. Conflicts are reported on save and before start with a 'Use next free port' quick fix. 0.0.0.0 counts as overlapping every IP on the same port.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/NATEngineImpl.cpp:318-323; Main/src-server/NATNetworkImpl.cpp:799-804; Devices/Network/DrvNAT.cpp:1836-1848`

#### Scenario: Acceptance for FR-CP06-003
- **WHEN** the FR-CP06-003 acceptance check is run
- **THEN** VM A has 2222→22; saving VM B with 2222 shows an inline 'host port 2222 is already used by A' error plus a 'Use 2223' button within ≤ 50 ms; 0 VMs fail to start with VERR_NAT_REDIR_SETUP in a 50-VM test.

### Requirement: Supervised NAT network lifecycle (FR-CP06-004)
Virtax SHALL provide "Supervised NAT network lifecycle" as specified by FR-CP06-004 (BRD-07, priority Must, phase P1; Vietnamese: "NAT network có vòng đời được giám sát").

#### Scenario: Specified behaviour of FR-CP06-004
- **WHEN** the behaviour FR-CP06-004 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Create/edit/delete NAT networks with an IPv4 CIDR, optional IPv6, DHCP, port forwards and loopback mappings. The NAT network service only runs while a VM uses it; virtaxd watches its PID (pidfd/kqueue/WaitForSingleObject), restarts it on crash and reports Running/Stopped/Error.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/NATNetworkImpl.cpp; Main/src-server/NetworkServiceRunner.cpp:191-235; NetworkServices/NAT/VBoxNetSlirpNAT.cpp`

#### Scenario: Acceptance for FR-CP06-004
- **WHEN** the FR-CP06-004 acceptance check is run
- **THEN** Killing the NAT network process while 2 VMs use it → virtaxd restarts it within ≤ 3 s, guests keep their IPs, and the event shows in the Task center.

### Requirement: Host-only networks, one model for 3 OSes (FR-CP06-005)
Virtax SHALL provide "Host-only networks, one model for 3 OSes" as specified by FR-CP06-005 (BRD-07, priority Must, phase P1; Vietnamese: "Host-only network, một mô hình cho 3 OS").

#### Scenario: Specified behaviour of FR-CP06-005
- **WHEN** the behaviour FR-CP06-005 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Users create a 'host-only network' (name, IPv4/mask, DHCP range) the same way on every OS. Phase 1 maps it to: macOS vmnet host mode (IHostOnlyNetwork); Linux BYO vboxnetadp, or with the Engine Pack a TAP on an isolated bridge created by virtax-helper; Windows VBoxNetAdp6 from the installed VirtualBox. A missing backend reports VTX-NET-010 with guidance.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/HostOnlyNetworkImpl.cpp; Frontends/VirtualBox/src/networkmanager/UINetworkManager.cpp:54-147; HostDrivers/adpctl/VBoxNetAdpCtl.cpp`

#### Scenario: Acceptance for FR-CP06-005
- **WHEN** the FR-CP06-005 acceptance check is run
- **THEN** The same `virtaxctl network create host-only lab --cidr 10.10.0.0/24` works on 3 OSes and 2 lab VMs can ping each other and the host.

### Requirement: Bridged with an interface list and Wi-Fi caveats (FR-CP06-006)
Virtax SHALL provide "Bridged with an interface list and Wi-Fi caveats" as specified by FR-CP06-006 (BRD-07, priority Must, phase P1; Vietnamese: "Bridged có danh sách interface và cảnh báo Wi-Fi").

#### Scenario: Specified behaviour of FR-CP06-006
- **WHEN** the behaviour FR-CP06-006 covers is exercised
- **THEN** it behaves as BRD-07 specifies: A live list of host interfaces (friendly name, wired/Wi-Fi/VPN type, link state, IP). For Wi-Fi the UI explains the limits: most adapters reject foreign MACs so the engine rewrites the source MAC, only IPv4/IPv6 work, and some APs/enterprise networks block it; it suggests NAT + port forwarding. A renamed/removed interface puts the NIC into 'needs attention' instead of silently switching networks.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/HostNetworkInterfaceImpl.cpp; HostDrivers/VBoxNetFlt/win/drv/VBoxNetLwf-win.cpp; Devices/Network/DrvVMNet.m`

#### Scenario: Acceptance for FR-CP06-006
- **WHEN** the FR-CP06-006 acceptance check is run
- **THEN** Choosing Wi-Fi for bridged shows a warning banner + a 'Use NAT instead' link; unplugging a bridged USB-Ethernet adapter shows 'Interface missing' on the NIC within ≤ 2 s.

### Requirement: Internal networks with a member list (FR-CP06-007)
Virtax SHALL provide "Internal networks with a member list" as specified by FR-CP06-007 (BRD-07, priority Should, phase P1; Vietnamese: "Internal network có danh sách thành viên").

#### Scenario: Specified behaviour of FR-CP06-007
- **WHEN** the behaviour FR-CP06-007 covers is exercised
- **THEN** it behaves as BRD-07 specifies: An internal network is a named L2 segment between VMs with no host. The Network manager shows which VMs are on each network, with an optional DHCP server. Names are suggested from existing networks to avoid typos (VBox silently creates a new network on a mistyped name).
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvIntNet.cpp; Devices/Network/SrvIntNetR0.cpp; NetworkServices/IntNetSwitch/VBoxIntNetSwitch.cpp`

#### Scenario: Acceptance for FR-CP06-007
- **WHEN** the FR-CP06-007 acceptance check is run
- **THEN** 2 VMs on internal 'lab' + DHCP get addresses within ≤ 5 s and ping each other; typing 'Lab' when 'lab' exists suggests 'Did you mean lab?'.

### Requirement: DHCP server: ranges, static leases, options (FR-CP06-008)
Virtax SHALL provide "DHCP server: ranges, static leases, options" as specified by FR-CP06-008 (BRD-07, priority Should, phase P1; Vietnamese: "DHCP server: dải, lease tĩnh, option").

#### Scenario: Specified behaviour of FR-CP06-008
- **WHEN** the behaviour FR-CP06-008 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Configure IP ranges, lease time, static leases by MAC or by VM (Virtax resolves the MAC), DNS/domain/gateway options; a lease table shows MAC, IP, VM and expiry. Phase 1 uses IDHCPServer/VBoxNetDHCP; Phase 2 the Rust DHCP (FR-EN06-009). Ranges are validated to lie inside the subnet and not overlap the host IP.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/DHCPServerImpl.cpp; Main/src-server/DHCPConfigImpl.cpp; NetworkServices/Dhcpd/`

#### Scenario: Acceptance for FR-CP06-008
- **WHEN** the FR-CP06-008 acceptance check is run
- **THEN** A static lease 10.10.0.21 for VM 'db' is honoured after a guest reboot; a range 10.10.1.0-… for subnet 10.10.0.0/24 gives an inline error.

### Requirement: Guest DNS follows host network changes (FR-CP06-009)
Virtax SHALL provide "Guest DNS follows host network changes" as specified by FR-CP06-009 (BRD-07, priority Should, phase P1; Vietnamese: "DNS theo kịp thay đổi mạng host").

#### Scenario: Specified behaviour of FR-CP06-009
- **WHEN** the behaviour FR-CP06-009 covers is exercised
- **THEN** it behaves as BRD-07 specifies: By default NAT uses a DNS relay (10.0.2.3) forwarding to the host's current resolvers; when the host changes Wi-Fi, toggles a VPN or uses split DNS, virtaxd applies the new configuration without a guest renew. Host-resolver mode and static name mappings (host-resolver mappings) are supported.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNAT.cpp:2575-2585 (UseHostResolver, HostResolverMappings); Main (HostNameResolutionConfigurationChange event)`

#### Scenario: Acceptance for FR-CP06-009
- **WHEN** the FR-CP06-009 acceptance check is run
- **THEN** Turning on the corporate VPN on the host makes `getent hosts git.corp.local` succeed in the guest within ≤ 5 s without reboot or `dhclient`.

### Requirement: Validated MTU and jumbo frames (FR-CP06-010)
Virtax SHALL provide "Validated MTU and jumbo frames" as specified by FR-CP06-010 (BRD-07, priority Should, phase P1; Vietnamese: "MTU và jumbo frame có kiểm tra").

#### Scenario: Specified behaviour of FR-CP06-010
- **WHEN** the behaviour FR-CP06-010 covers is exercised
- **THEN** it behaves as BRD-07 specifies: MTU is configurable per network/NIC. In Phase 1 NAT is clamped below 16 KiB (larger frames are silently dropped by the engine) and the UI explains why; bridged/host-only allow 9000-byte jumbo frames when the host interface supports them. Phase 2 raises limits per backend (FR-EN06-010).
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNAT.cpp:95,101,757-787,2642-2663`

#### Scenario: Acceptance for FR-CP06-010
- **WHEN** the FR-CP06-010 acceptance check is run
- **THEN** Entering MTU 65000 for NAT in Phase 1 gives an inline 'maximum 16,000' error with the reason; bridged MTU 9000 on a capable NIC passes `ping -M do -s 8972`.

### Requirement: Default NIC model and MAC policy (FR-CP06-011)
Virtax SHALL provide "Default NIC model and MAC policy" as specified by FR-CP06-011 (BRD-07, priority Must, phase P1; Vietnamese: "Model NIC mặc định và chính sách MAC").

#### Scenario: Specified behaviour of FR-CP06-011
- **WHEN** the behaviour FR-CP06-011 covers is exercised
- **THEN** it behaves as BRD-07 specifies: New VMs: virtio-net for Linux guests; Windows guests use E1000 until netkvm is installed (reported by the agent/virtio-win), then Virtax offers to switch to virtio-net. New MACs are unique within the registry; imported VMs keep their MACs; clones can keep or regenerate MACs.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-all/Global.cpp:215,262-266 (E1000 defaults); Devices/Network/DevE1000.cpp:129-132`

#### Scenario: Acceptance for FR-CP06-011
- **WHEN** the FR-CP06-011 acceptance check is run
- **THEN** Quick Create Ubuntu yields a virtio-net NIC; creating 100 VMs yields 0 duplicate MACs; cloning with 'keep MAC' while the source runs warns about a duplicate on the same network.

### Requirement: Engine Pack network patches (Phase 1) (FR-CP06-012)
Virtax SHALL provide "Engine Pack network patches (Phase 1)" as specified by FR-CP06-012 (BRD-07, priority Should, phase P1; Vietnamese: "Vá mạng trong Engine Pack (Pha 1)").

#### Scenario: Specified behaviour of FR-CP06-012
- **WHEN** the behaviour FR-CP06-012 covers is exercised
- **THEN** it behaves as BRD-07 specifies: The Engine Pack includes: (a) DrvVMNet batched reads/writes with a vmpktdesc[] array, preallocated buffers and pfnWaitReceiveAvail for back-pressure; (b) a Linux build without NetFlt so bridged/host-only use TAPs pre-created by virtax-helper (TUNSETOWNER to the user); (c) an evaluation of enabling the experimental ring-3 IntNet LocalIPC service for driverless internal/NAT networks. Patch (a) is sent upstream.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvVMNet.m:205-218,233-280,435-460; Main/src-client/ConsoleImplConfigCommon.cpp:2416-2440; Config.kmk:581-587`

#### Scenario: Acceptance for FR-CP06-012
- **WHEN** the FR-CP06-012 acceptance check is run
- **THEN** M-series Mac: bridged iperf3 ≥ 1.5× stock VBox 7.2 with no more 'we will fetch just one' log lines; Ubuntu Engine Pack: bridged works without vboxnetflt.ko.

### Requirement: Per-NIC network diagnostics (FR-CP06-013)
Virtax SHALL provide "Per-NIC network diagnostics" as specified by FR-CP06-013 (BRD-07, priority Could, phase P1; Vietnamese: "Chẩn đoán mạng từng NIC").

#### Scenario: Specified behaviour of FR-CP06-013
- **WHEN** the behaviour FR-CP06-013 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Each NIC shows link, guest IP (from GA/agent), rx/tx and drops; a 'Test connectivity' button checks DHCP lease, gateway, DNS and Internet in turn and says which step failed; 'Capture' writes a size-capped pcapng (100 MB default) to attach to bug reports.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNetSniffer.cpp, Pcap.cpp; HostServices/GuestProperties (/VirtualBox/GuestInfo/Net)`

#### Scenario: Acceptance for FR-CP06-013
- **WHEN** the FR-CP06-013 acceptance check is run
- **THEN** Simulating a broken DNS relay makes Test connectivity stop at the 'DNS' step with a fix hint within ≤ 5 s; the pcapng file opens in Wireshark.
