# Network backends Specification

## Purpose
passt/libslirp for user NAT, TAP + vhost-net (Linux), batched vmnet (macOS), HNS/WinTun (Windows), multi-queue. Module `EN-06` (Network backends; Vietnamese: "Network backend") from BRD-07, phase P2, priority Must. Owner crates: virtax-vmm-net. Depends on: `engine/virtax-vmm-core-and-hypervisor-backends`, `platform/host-integration-and-power`. VirtualBox reference: `src/VBox/Devices/Network (DrvNAT, DrvVMNet, DevVirtioNet), HostDrivers/VBoxNetFlt, VBoxNetAdp`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Multi-queue virtio-net with offloads (FR-EN06-001)
Virtax SHALL provide "Multi-queue virtio-net with offloads" as specified by FR-EN06-001 (BRD-07, priority Must, phase P2; Vietnamese: "virtio-net multi-queue có offload").

#### Scenario: Specified behaviour of FR-EN06-001
- **WHEN** the behaviour FR-EN06-001 covers is exercised
- **THEN** it behaves as BRD-07 specifies: virtax-vmm virtio-net has queue pairs = vCPU count (max 16), MRG_RXBUF, CSUM, GUEST/HOST_TSO4/6, EVENT_IDX and a ctrl vq (MAC filter, promisc); rings are mapped via `vm-memory`, kicks use ioeventfd and interrupts irqfd on KVM; each queue has its own worker.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DevVirtioNet.cpp:139-176; Devices/VirtIO/VirtioCore.h:252; VirtioCore.cpp:180-245`

#### Scenario: Acceptance for FR-EN06-001
- **WHEN** the FR-EN06-001 acceptance check is run
- **THEN** A 4-vCPU VM running `iperf3 -P 4` guest↔host over TAP gets ≥ 3× the single-queue throughput; `ethtool -l` in the guest reports 4 combined.

### Requirement: User-mode NAT via passt on Linux (FR-EN06-002)
Virtax SHALL provide "User-mode NAT via passt on Linux" as specified by FR-EN06-002 (BRD-07, priority Must, phase P2; Vietnamese: "User NAT bằng passt trên Linux").

#### Scenario: Specified behaviour of FR-EN06-002
- **WHEN** the behaviour FR-EN06-002 covers is exercised
- **THEN** it behaves as BRD-07 specifies: On Linux, NAT runs passt by default (separate unprivileged process translating L2 ↔ L4 sockets, so TSO goes straight to host sockets), connected to virtax-vmm over a UNIX socket or vhost-user when passt supports it. virtax-vmm spawns, supervises and sandboxes passt; it falls back to libslirp when passt is missing or too old.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNAT.cpp:672-700,749-797,843-905`

#### Scenario: Acceptance for FR-EN06-002
- **WHEN** the FR-EN06-002 acceptance check is run
- **THEN** Ubuntu 24.04 host: iperf3 guest→host over NAT ≥ 2× VBox 7.2; killing passt makes virtax-vmm restart it within ≤ 1 s and new TCP connections succeed.

### Requirement: Batched libslirp for macOS/Windows (FR-EN06-003)
Virtax SHALL provide "Batched libslirp for macOS/Windows" as specified by FR-EN06-003 (BRD-07, priority Must, phase P2; Vietnamese: "libslirp batch cho macOS/Windows").

#### Scenario: Specified behaviour of FR-EN06-003
- **WHEN** the behaviour FR-EN06-003 covers is exercised
- **THEN** it behaves as BRD-07 specifies: On macOS/Windows (and as the Linux fallback), NAT uses libslirp via FFI with a rewritten data path: preallocated ring buffers, no per-frame malloc/pipe syscall, batch processing, large segments delivered to virtio-net via GUEST_TSO, and a stable poll set instead of one rebuilt each loop. A spike compares it with OpenVMM's user-mode NAT crate (`net_consomme`, MIT).
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNAT.cpp:539-605,1006-1110,2188-2205; src/libs/libslirp-4.9.5`

#### Scenario: Acceptance for FR-EN06-003
- **WHEN** the FR-EN06-003 acceptance check is run
- **THEN** macOS 15 and Windows 11: iperf3 guest→host over NAT ≥ 2× VBox 7.2; perf/Instruments shows 0 mallocs on the steady-state TX path.

### Requirement: Port forwarding in virtax-vmm NAT (FR-EN06-004)
Virtax SHALL provide "Port forwarding in virtax-vmm NAT" as specified by FR-EN06-004 (BRD-07, priority Must, phase P2; Vietnamese: "Port forwarding trong virtax-vmm NAT").

#### Scenario: Specified behaviour of FR-EN06-004
- **WHEN** the behaviour FR-EN06-004 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Same rule model as CP-06 (tcp/udp, IPv4/IPv6, host-IP binding, 127.0.0.1 by default); live add/remove through virtax-vmm's control socket; bind failures return VTX-NET-020 with the process holding the port when the OS exposes it.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNAT.cpp:1820-1882 (drvNATAddRedirect)`

#### Scenario: Acceptance for FR-EN06-004
- **WHEN** the FR-EN06-004 acceptance check is run
- **THEN** Adding an IPv6 forward [::1]:8080 → guest 80 on a running virtax-vmm VM makes `curl -6` succeed within ≤ 1 s; a port held by host nginx reports 'nginx (pid …)'.

### Requirement: TAP + vhost-net on Linux (FR-EN06-005)
Virtax SHALL provide "TAP + vhost-net on Linux" as specified by FR-EN06-005 (BRD-07, priority Must, phase P2; Vietnamese: "TAP + vhost-net trên Linux").

#### Scenario: Specified behaviour of FR-EN06-005
- **WHEN** the behaviour FR-EN06-005 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Bridged and host-only on Linux use `IFF_TAP | IFF_VNET_HDR | IFF_MULTI_QUEUE` TAPs attached to a Linux bridge (or macvtap for bridged). virtax-helper creates the interfaces and passes the TAP and /dev/vhost-net fds to virtax-vmm over SCM_RIGHTS, so virtax-vmm runs unprivileged; the in-kernel vhost-net datapath uses ioeventfd/irqfd.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvTAP.cpp:273-287 (no IFF_VNET_HDR); HostDrivers/VBoxNetFlt/linux/VBoxNetFlt-linux.c:765-806,1121`

#### Scenario: Acceptance for FR-EN06-005
- **WHEN** the FR-EN06-005 acceptance check is run
- **THEN** Linux x64 and arm64: bridged iperf3 ≥ 2× VBox 7.2 bridged; the virtax-vmm process holds no capabilities (`getpcaps` empty).

### Requirement: Batched vmnet on macOS (FR-EN06-006)
Virtax SHALL provide "Batched vmnet on macOS" as specified by FR-EN06-006 (BRD-07, priority Must, phase P2; Vietnamese: "vmnet batch trên macOS").

#### Scenario: Specified behaviour of FR-EN06-006
- **WHEN** the behaviour FR-EN06-006 covers is exercised
- **THEN** it behaves as BRD-07 specifies: A Rust backend for VMNET_SHARED_MODE, VMNET_HOST_MODE and VMNET_BRIDGED_MODE: batched vmpktdesc-array reads/writes with preallocated buffers, back-pressure when the guest ring is full, and TSO/checksum offload keys where the macOS version supports them. If the build lacks the `com.apple.vm.networking` entitlement, virtax-helper (an SMAppService daemon) owns the vmnet interface and relays packets over a shared-memory ring.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvVMNet.m:205-280,435-460; Config.kmk:1847-1857 (restricted entitlement)`

#### Scenario: Acceptance for FR-EN06-006
- **WHEN** the FR-EN06-006 acceptance check is run
- **THEN** M-series Mac on macOS 15: bridged iperf3 ≥ 2× stock VBox 7.2; ≥ 1.5× even via the helper relay; 0 packet drops while the guest is busy (back-pressure).

### Requirement: Windows backends without own drivers (FR-EN06-007)
Virtax SHALL provide "Windows backends without own drivers" as specified by FR-EN06-007 (BRD-07, priority Should, phase P2; Vietnamese: "Backend Windows không driver riêng").

#### Scenario: Specified behaviour of FR-EN06-007
- **WHEN** the behaviour FR-EN06-007 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Host-only on Windows uses a WinTun adapter (L3, signed by the WinTun publisher) created by the helper, with virtax-vmm-net emulating ARP/ND and DHCP in user space so the guest sees an L2 network. Bridged: a spike evaluates HNS/HCN and the Hyper-V vSwitch for a third-party VMM; if infeasible, VMs needing bridged stay on the legacy engine (the installed VirtualBox's VBoxNetLwf driver) and the UI says so.
- **AND** the VirtualBox reference for the current behaviour is `HostDrivers/VBoxNetFlt/win/drv/VBoxNetLwf-win.cpp:1793-1800; HostDrivers/VBoxNetAdp/win; Installer/win/InstallHelper/VBoxInstallHelper.cpp:2167`

#### Scenario: Acceptance for FR-EN06-007
- **WHEN** the FR-EN06-007 acceptance check is run
- **THEN** Windows 11 x64/arm64: an virtax-vmm VM on host-only pings the host and other VMs and gets DHCP within ≤ 3 s; Virtax installs no NDIS driver (checked with `pnputil /enum-drivers`).

### Requirement: virtax-vmm-net switch for internal and NAT networks (FR-EN06-008)
Virtax SHALL provide "virtax-vmm-net switch for internal and NAT networks" as specified by FR-EN06-008 (BRD-07, priority Should, phase P2; Vietnamese: "Switch virtax-vmm-net cho internal và NAT network").

#### Scenario: Specified behaviour of FR-EN06-008
- **WHEN** the behaviour FR-EN06-008 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Each internal/NAT network has a user-space switch (`virtax-vmm-net`, spawned by virtaxd) connected to each VM via shared-memory rings with eventfd/kqueue/event signalling; MAC learning, broadcast/multicast, no per-frame syscalls in steady state; NAT networks attach a passt/libslirp uplink to the switch. On Linux vhost-user can connect two virtax-vmm processes directly.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/SrvIntNetR0.cpp:1235,2820-2831; NetworkServices/IntNetSwitch/VBoxIntNetSwitch.cpp; NetworkServices/NAT/VBoxNetSlirpNAT.cpp:2321-2326`

#### Scenario: Acceptance for FR-EN06-008
- **WHEN** the FR-EN06-008 acceptance check is run
- **THEN** 2 VMs on one host on an internal network: VM↔VM iperf3 ≥ 5 Gbit/s on an 8-core laptop; killing the switch makes virtaxd restart it within ≤ 2 s and new TCP sessions work.

### Requirement: Rust DHCP and DNS forwarder (FR-EN06-009)
Virtax SHALL provide "Rust DHCP and DNS forwarder" as specified by FR-EN06-009 (BRD-07, priority Must, phase P2; Vietnamese: "DHCP và DNS forwarder bằng Rust").

#### Scenario: Specified behaviour of FR-EN06-009
- **WHEN** the behaviour FR-EN06-009 covers is exercised
- **THEN** it behaves as BRD-07 specifies: DHCPv4 (dynamic/static leases, DNS/domain/router options) and a DNS forwarder run inside virtax-vmm-net, replacing VBoxNetDHCP (lwIP). The DNS forwarder uses the host's current resolvers and updates when the host changes network/VPN (FR-CP06-009). Leases are persisted so IPs survive restarts.
- **AND** the VirtualBox reference for the current behaviour is `NetworkServices/Dhcpd/Makefile.kmk:76 (lwIP); Main/src-server/DHCPServerImpl.cpp`

#### Scenario: Acceptance for FR-EN06-009
- **WHEN** the FR-EN06-009 acceptance check is run
- **THEN** 10 VMs get leases within ≤ 2 s; restarting virtaxd keeps every VM's IP; switching the host VPN lets internal names resolve in guests within ≤ 5 s.

### Requirement: End-to-end MTU without silent drops (FR-EN06-010)
Virtax SHALL provide "End-to-end MTU without silent drops" as specified by FR-EN06-010 (BRD-07, priority Should, phase P2; Vietnamese: "MTU đầu-cuối, không drop âm thầm").

#### Scenario: Specified behaviour of FR-EN06-010
- **WHEN** the behaviour FR-EN06-010 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Each backend declares its maximum MTU (TAP/vhost and passt up to 65520, vmnet and WinTun per OS); virtio-net advertises it via VIRTIO_NET_F_MTU. Oversized frames are rejected with a counter and an event instead of the silent drops of DrvNAT's 16 KiB limit.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNAT.cpp:95,757-764,782-787`

#### Scenario: Acceptance for FR-EN06-010
- **WHEN** the FR-EN06-010 acceptance check is run
- **THEN** MTU 65520 on TAP: iperf3 runs without errors; sending a frame above the backend MTU increments an 'oversize' counter and logs an event in the VM log.

### Requirement: Per-NIC rate limiting and capture (FR-EN06-011)
Virtax SHALL provide "Per-NIC rate limiting and capture" as specified by FR-EN06-011 (BRD-07, priority Could, phase P2; Vietnamese: "Giới hạn băng thông và capture theo NIC").

#### Scenario: Specified behaviour of FR-EN06-011
- **WHEN** the behaviour FR-EN06-011 covers is exercised
- **THEN** it behaves as BRD-07 specifies: rx/tx token buckets (Mbit/s, burst) changeable live, replacing DrvNetShaper bandwidth groups; live per-NIC pcapng capture toggling with a size-capped ring file and no data-path cost when off.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNetShaper.cpp; Devices/Network/DrvNetSniffer.cpp`

#### Scenario: Acceptance for FR-EN06-011
- **WHEN** the FR-EN06-011 acceptance check is run
- **THEN** Setting 50 Mbit/s yields 50 ± 5% in iperf3; enabling capture during iperf3 lowers throughput by ≤ 10%.

### Requirement: Per-queue network metrics, no idle wakeups (FR-EN06-012)
Virtax SHALL provide "Per-queue network metrics, no idle wakeups" as specified by FR-EN06-012 (BRD-07, priority Should, phase P2; Vietnamese: "Metrics mạng theo queue, không wakeup khi idle").

#### Scenario: Specified behaviour of FR-EN06-012
- **WHEN** the behaviour FR-EN06-012 covers is exercised
- **THEN** it behaves as BRD-07 specifies: Per-queue pps, bytes, drops, oversize and backend latency counters in shared memory; virtaxd only streams them while subscribed. With no traffic, backends and switches create no periodic wakeups (no per-loop rebuilt poll set).
- **AND** the VirtualBox reference for the current behaviour is `Devices/Network/DrvNAT.cpp:1036-1048,2406-2430 (poll rebuild)`

#### Scenario: Acceptance for FR-EN06-012
- **WHEN** the FR-EN06-012 acceptance check is run
- **THEN** An idle VM on NAT: 0 wakeups/s from network threads (powertop/Instruments/ETW) apart from guest TCP timers; the Performance tab shows rx/tx within ≤ 1 s of opening.

### Requirement: Unprivileged backends, narrow helper, fuzzed parsers (FR-EN06-013)
Virtax SHALL provide "Unprivileged backends, narrow helper, fuzzed parsers" as specified by FR-EN06-013 (BRD-07, priority Must, phase P2; Vietnamese: "Backend không đặc quyền, helper hẹp, parser được fuzz").

#### Scenario: Specified behaviour of FR-EN06-013
- **WHEN** the behaviour FR-EN06-013 covers is exercised
- **THEN** it behaves as BRD-07 specifies: virtax-vmm and virtax-vmm-net run unprivileged (seccomp on Linux, sandbox/hardened runtime on macOS, AppContainer/job objects on Windows). Only the helper performs CreateTap, CreateBridge, AttachToBridge, OpenVhostNet, CreateVmnetInterface, CreateWinTunAdapter and DeleteInterface; each command checks the caller and writes an audit log. DHCP/ARP/ND/DNS and vnet_hdr parsers run cargo-fuzz in CI.
- **AND** the VirtualBox reference for the current behaviour is `HostDrivers/adpctl/VBoxNetAdpCtl.cpp:89-90,359,732 (setuid shelling out); Installer/linux/install.sh:296-313`

#### Scenario: Acceptance for FR-EN06-013
- **WHEN** the FR-EN06-013 acceptance check is run
- **THEN** Calling the helper from another UID is denied and audited; 24 h fuzzing per parser: 0 panics; a guest sending a malformed vnet_hdr gets the frame dropped and the VM does not crash.
