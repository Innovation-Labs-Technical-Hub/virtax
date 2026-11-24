# Guest agent & drivers Specification

## Purpose
Small Rust agent over virtio-vsock (clipboard, resize, time, exec, files); automated virtio-win for Windows. Module `EN-08` (Guest agent & drivers; Vietnamese: "Guest agent & driver") from BRD-08, phase P2, priority Should. Owner crates: virtax-agent. Depends on: `engine/device-model-and-firmware`. VirtualBox reference: `src/VBox/Additions (VBoxService, VBoxClient; 95% vendored Mesa/X.Org)`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: virtax-agent: one small, event-driven binary (FR-EN08-001)
Virtax SHALL provide "virtax-agent: one small, event-driven binary" as specified by FR-EN08-001 (BRD-08, priority Must, phase P2; Vietnamese: "virtax-agent: một binary nhỏ, event-driven").

#### Scenario: Specified behaviour of FR-EN08-001
- **WHEN** the behaviour FR-EN08-001 covers is exercised
- **THEN** it behaves as BRD-08 specifies: One static Rust binary (Linux x64/arm64 musl; Windows x64/arm64) running as a systemd service / Windows service, with per-user-session parts (clipboard, display) as child processes of the same binary. tokio, no periodic polling beyond the heartbeat; no kernel modules; every feature toggled by capability.
- **AND** the VirtualBox reference for the current behaviour is `Additions/common/VBoxService/*, Additions/x11/VBoxClient/*, Additions/win/VBoxTray (replaced)`

#### Scenario: Acceptance for FR-EN08-001
- **WHEN** the FR-EN08-001 acceptance check is run
- **THEN** Binary ≤ 6 MB, RSS ≤ 15 MB after 24 h, idle ≤ 0.2 wakeups/s (powertop/xperf) on Ubuntu 24.04 and Windows 11.

### Requirement: virtio-vsock channel and versioned protocol (FR-EN08-002)
Virtax SHALL provide "virtio-vsock channel and versioned protocol" as specified by FR-EN08-002 (BRD-08, priority Must, phase P2; Vietnamese: "Kênh virtio-vsock và protocol có version").

#### Scenario: Specified behaviour of FR-EN08-002
- **WHEN** the behaviour FR-EN08-002 covers is exercised
- **THEN** it behaves as BRD-08 specifies: The agent listens on a privileged vsock port (< 1024, bindable only by root/SYSTEM); the host connects from virtax-vmm's in-process vsock device (no host OS socket) and relays to virtaxd. Length-prefixed protobuf messages; Hello exchanges version + capabilities; the host supports agents up to N-2 versions old.
- **AND** the VirtualBox reference for the current behaviour is `Devices/VMMDev/VMMDev.cpp:3176,3437; include/VBox/VMMDev.h:78-81,183-191 (HGCM transport replaced)`

#### Scenario: Acceptance for FR-EN08-002
- **WHEN** the FR-EN08-002 acceptance check is run
- **THEN** An agent 2 versions old still runs clipboard/exec; a non-root guest process cannot bind the agent port; ping round-trip ≤ 1 ms on KVM.

### Requirement: Text, image and file clipboard (FR-EN08-003)
Virtax SHALL provide "Text, image and file clipboard" as specified by FR-EN08-003 (BRD-08, priority Must, phase P2; Vietnamese: "Clipboard text, ảnh và file").

#### Scenario: Specified behaviour of FR-EN08-003
- **WHEN** the behaviour FR-EN08-003 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Formats text/plain UTF-8, text/html, image/png; files are a lazily fetched reference list via file copy (FR-EN08-008). Linux: X11 and Wayland (data-control protocol when the compositor offers it, otherwise the per-session process uses the portal/XWayland); Windows: the Win32 clipboard. Default 64 MiB cap; sync only when the mode allows.
- **AND** the VirtualBox reference for the current behaviour is `GuestHost/SharedClipboard/* (20,339 lines); Additions/x11/VBoxClient/clipboard*`

#### Scenario: Acceptance for FR-EN08-003
- **WHEN** the FR-EN08-003 acceptance check is run
- **THEN** GNOME Wayland and KDE Plasma 6 guests: bidirectional text/image copy within ≤ 300 ms; Windows 11: pasting 10 host files into Explorer copies them correctly.

### Requirement: Auto-resize via virtio-gpu and the agent (FR-EN08-004)
Virtax SHALL provide "Auto-resize via virtio-gpu and the agent" as specified by FR-EN08-004 (BRD-08, priority Must, phase P2; Vietnamese: "Auto-resize qua virtio-gpu và agent").

#### Scenario: Specified behaviour of FR-EN08-004
- **WHEN** the behaviour FR-EN08-004 covers is exercised
- **THEN** it behaves as BRD-08 specifies: virtax-vmm sends virtio-gpu EDID/hotplug when the Console resizes; Linux compositors switch modes themselves. When the guest does not (older Xorg, Windows viogpudo), the agent sets the mode via xrandr / Mutter/KWin D-Bus or ChangeDisplaySettingsEx and reports the result.
- **AND** the VirtualBox reference for the current behaviour is `Additions/x11/VBoxClient (display-svga-*); Additions/win/Graphics (WDDM, replaced by viogpudo)`

#### Scenario: Acceptance for FR-EN08-004
- **WHEN** the FR-EN08-004 acceptance check is run
- **THEN** Ubuntu 24.04 (Wayland) and Windows 11 on virtax-vmm: resizing the window makes the guest match within ≤ 500 ms; 2 monitors keep the right layout.

### Requirement: Event-driven time sync (FR-EN08-005)
Virtax SHALL provide "Event-driven time sync" as specified by FR-EN08-005 (BRD-08, priority Should, phase P2; Vietnamese: "Đồng bộ giờ theo sự kiện").

#### Scenario: Specified behaviour of FR-EN08-005
- **WHEN** the behaviour FR-EN08-005 covers is exercised
- **THEN** it behaves as BRD-08 specifies: The base clock uses hypervisor enlightenments (kvm-clock, Hyper-V reference TSC, CNTVCT on arm64); the agent only steps/slews when the host signals resume, restore or host wake, or when drift exceeds 1 s. No 10 s poll loop.
- **AND** the VirtualBox reference for the current behaviour is `Additions/common/VBoxService/VBoxServiceTimeSync.cpp:266-269,644`

#### Scenario: Acceptance for FR-EN08-005
- **WHEN** the FR-EN08-005 acceptance check is run
- **THEN** Closing the laptop lid for 8 h then opening it brings the guest clock within ±0.5 s in ≤ 2 s; while idle the agent has no time-sync wakeups.

### Requirement: Graceful shutdown, reboot and suspend via the agent (FR-EN08-006)
Virtax SHALL provide "Graceful shutdown, reboot and suspend via the agent" as specified by FR-EN08-006 (BRD-08, priority Must, phase P2; Vietnamese: "Tắt máy, reboot, suspend êm qua agent").

#### Scenario: Specified behaviour of FR-EN08-006
- **WHEN** the behaviour FR-EN08-006 covers is exercised
- **THEN** it behaves as BRD-08 specifies: The 'graceful' stop mode sends power.shutdown to the agent (systemd/InitiateShutdown), which acknowledges receipt; without an agent ACPI is used; on timeout (60 s default) Virtax reports it and offers power off. Reboot and suspend-to-RAM are supported when the guest allows.
- **AND** the VirtualBox reference for the current behaviour is `Devices/PC/DevACPI.cpp (power button); Main/src-client/ConsoleImpl.cpp (powerButton)`

#### Scenario: Acceptance for FR-EN08-006
- **WHEN** the FR-EN08-006 acceptance check is run
- **THEN** Windows 11 at the lock screen (where ACPI is often ignored) still shuts down gracefully via the agent within ≤ 30 s; 20 CI VMs shut down in parallel within ≤ 30 s.

### Requirement: Exec and guest shell (FR-EN08-007)
Virtax SHALL provide "Exec and guest shell" as specified by FR-EN08-007 (BRD-08, priority Should, phase P2; Vietnamese: "Exec và guest shell").

#### Scenario: Specified behaviour of FR-EN08-007
- **WHEN** the behaviour FR-EN08-007 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Exec takes argv, env, cwd, user and stdin; streams stdout/stderr; returns the exit code; supports timeout and kill. A PTY mode serves the Guest shell tab (UI-06) with window resize. Host-initiated only, under the exec_allowed policy (FR-CP07-011).
- **AND** the VirtualBox reference for the current behaviour is `HostServices/GuestControl/*; Additions/common/VBoxService/VBoxServiceControl*.cpp`

#### Scenario: Acceptance for FR-EN08-007
- **WHEN** the FR-EN08-007 acceptance check is run
- **THEN** `virtaxctl guest exec vm -- bash -c 'exit 3'` returns 3; Guest shell runs `htop` with correct colours and resize; a hung command is killed at the timeout.

### Requirement: Pipelined, resumable file copy (FR-EN08-008)
Virtax SHALL provide "Pipelined, resumable file copy" as specified by FR-EN08-008 (BRD-08, priority Must, phase P2; Vietnamese: "Chép file pipeline, resume được").

#### Scenario: Specified behaviour of FR-EN08-008
- **WHEN** the behaviour FR-EN08-008 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Bidirectional file/folder copy in 1 MiB chunks with ≥ 8 in flight, per-chunk checksums, resume after disconnects and preserved permissions/mtime; writes go to a temp file and are renamed so no partial file is left behind.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/GuestSessionImplTasks.cpp:420-425,753-758; Main/src-client/GuestFileImpl.cpp:1753-1754`

#### Scenario: Acceptance for FR-EN08-008
- **WHEN** the FR-EN08-008 acceptance check is run
- **THEN** A 4 GB host→guest file on KVM reaches ≥ 300 MB/s (≥ 5× VBox guest control); interrupting midway (restarting the virtax-vmm control channel) resumes from the last chunk.

### Requirement: IP reporting, guest info and heartbeat (FR-EN08-009)
Virtax SHALL provide "IP reporting, guest info and heartbeat" as specified by FR-EN08-009 (BRD-08, priority Must, phase P2; Vietnamese: "Báo IP, thông tin guest và heartbeat").

#### Scenario: Specified behaviour of FR-EN08-009
- **WHEN** the behaviour FR-EN08-009 covers is exercised
- **THEN** it behaves as BRD-08 specifies: The agent pushes interfaces/IPs (netlink; NotifyIpInterfaceChange on Windows), hostname, OS release and logged-in users as soon as they change; a heartbeat every 5 s; virtaxd raises a GuestUnresponsive event after 3 misses and GuestRecovered when it returns (replacing VMMDev's LogRel).
- **AND** the VirtualBox reference for the current behaviour is `Additions/common/VBoxService/VBoxServiceVMInfo.cpp; Devices/VMMDev/VMMDev.cpp:65-86 (heartbeat only logged)`

#### Scenario: Acceptance for FR-EN08-009
- **WHEN** the FR-EN08-009 acceptance check is run
- **THEN** A guest IP change is seen by the host within ≤ 2 s; freezing the agent with SIGSTOP raises Unresponsive within ≤ 16 s, shown on the Integration card.

### Requirement: Automatic virtio-fs mounts by tag (FR-EN08-010)
Virtax SHALL provide "Automatic virtio-fs mounts by tag" as specified by FR-EN08-010 (BRD-08, priority Must, phase P2; Vietnamese: "Tự mount virtio-fs theo tag").

#### Scenario: Specified behaviour of FR-EN08-010
- **WHEN** the behaviour FR-EN08-010 covers is exercised
- **THEN** it behaves as BRD-08 specifies: The agent receives the share list (tag, mount point, ro/rw) and mounts it: Linux `mount -t virtiofs` (or a systemd unit), Windows via viofs + the WinFsp service with a chosen drive letter; per-share mount status is reported to Integration.
- **AND** the VirtualBox reference for the current behaviour is `Additions/common/VBoxService/VBoxServiceAutoMount.cpp; Additions/linux/sharedfolders (vboxsf)`

#### Scenario: Acceptance for FR-EN08-010
- **WHEN** the FR-EN08-010 acceptance check is run
- **THEN** Adding share 'src' live yields /mnt/src in Linux guests and drive S: in Windows guests within ≤ 3 s; ro shares block writes.

### Requirement: Automated virtio-win install for Windows guests (FR-EN08-011)
Virtax SHALL provide "Automated virtio-win install for Windows guests" as specified by FR-EN08-011 (BRD-08, priority Must, phase P2; Vietnamese: "Cài virtio-win tự động cho Windows guest").

#### Scenario: Specified behaviour of FR-EN08-011
- **WHEN** the behaviour FR-EN08-011 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Windows boots from NVMe, so no driver is needed during setup; the unattended post-install step (CP-10) installs a version- and SHA-256-pinned virtio-win set: netkvm, viogpudo, vioinput, viofs + WinFsp, viosock, balloon, then the virtax-agent MSI. Once netkvm runs, Virtax offers to switch the NIC to virtio-net (FR-CP06-011).
- **AND** the VirtualBox reference for the current behaviour is `Main/UnattendedTemplates/win_postinstall.cmd; Main/src-server/UnattendedImpl.cpp`

#### Scenario: Acceptance for FR-EN08-011
- **WHEN** the FR-EN08-011 acceptance check is run
- **THEN** Quick Create Windows 11 on virtax-vmm reaches a desktop with virtio networking, auto-resize and clipboard with no manual steps; a checksum mismatch stops with VTX-GST-030.

### Requirement: Agent packaging per guest OS (FR-EN08-012)
Virtax SHALL provide "Agent packaging per guest OS" as specified by FR-EN08-012 (BRD-08, priority Must, phase P2; Vietnamese: "Đóng gói agent theo từng guest OS").

#### Scenario: Specified behaviour of FR-EN08-012
- **WHEN** the behaviour FR-EN08-012 covers is exercised
- **THEN** it behaves as BRD-08 specifies: .deb (Ubuntu/Debian), .rpm (Fedora/RHEL family), .apk (Alpine), pacman (Arch), Authenticode-signed MSI (Windows x64/arm64) and a best-effort FreeBSD tarball. Distributed via an auto-attached 'Virtax Guest Tools' ISO, cloud-init/unattended at VM creation and GPG-signed repos; the host offers agent updates when available (signed packages, user-confirmed).
- **AND** the VirtualBox reference for the current behaviour is `Additions/linux/installer/vboxadd.sh; Additions/win/Installer (NSIS) — replaced`

#### Scenario: Acceptance for FR-EN08-012
- **WHEN** the FR-EN08-012 acceptance check is run
- **THEN** Installing from the ISO succeeds in CI on Ubuntu 24.04, Fedora 41, Debian 13, Alpine 3.20 and Windows 11 x64/arm64; agent updates need no guest reboot.

### Requirement: Agent-channel security (FR-EN08-013)
Virtax SHALL provide "Agent-channel security" as specified by FR-EN08-013 (BRD-08, priority Must, phase P2; Vietnamese: "Bảo mật kênh agent").

#### Scenario: Specified behaviour of FR-EN08-013
- **WHEN** the behaviour FR-EN08-013 covers is exercised
- **THEN** it behaves as BRD-08 specifies: The guest is untrusted: the host validates schemas, caps sizes (messages ≤ 1 MiB except file data) and rate-limits; the guest may only send clipboard offers, display hints, info and heartbeats; exec/file/shutdown are host-initiated and policy-gated; host paths are never exposed to the guest. Host-side parsers run cargo-fuzz; every privileged command is audited.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/HGCM.cpp; Devices/VMMDev/VMMDevHGCM.cpp (guest-controlled request parsing)`

#### Scenario: Acceptance for FR-EN08-013
- **WHEN** the FR-EN08-013 acceptance check is run
- **THEN** A malicious guest sending 10,000 messages/s or a 100 MB message is throttled/disconnected with no virtax-vmm/virtaxd crash and a warning event; 24 h fuzzing: 0 panics.
