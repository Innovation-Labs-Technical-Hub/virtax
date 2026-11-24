# Guest integration Specification

## Purpose
Clipboard, shared folders, auto-resize, time sync, exec/file copy; P1 via Guest Additions, P2 via virtax-agent (vsock). Module `CP-07` (Guest integration; Vietnamese: "Tích hợp guest") from BRD-08, phase P1, priority Should. Owner crates: virtaxd, virtax-agent. Depends on: `engine/guest-agent-and-drivers`, `control-plane/vm-lifecycle-and-runtime-orchestration`. VirtualBox reference: `src/VBox/Main/src-client/Guest*Impl.cpp, src/VBox/HostServices (SharedFolders, SharedClipboard, DragAndDrop, GuestProperties)`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Unified Integration model (FR-CP07-001)
Virtax SHALL provide "Unified Integration model" as specified by FR-CP07-001 (BRD-08, priority Must, phase P1; Vietnamese: "Mô hình Integration thống nhất").

#### Scenario: Specified behaviour of FR-CP07-001
- **WHEN** the behaviour FR-CP07-001 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Each VM has a `guest/integration` resource: provider (guest-additions | virtax-agent | none), version, and per-feature state (clipboard, dnd, shared_folders, auto_resize, time_sync, exec, files, ip_report, heartbeat) = active | available | disabled-by-policy | unsupported | error. Phase 1 derives it from GA run level and facilities; Phase 2 from agent capabilities. UI and CLI only read this model.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/GuestImpl.cpp:402-420 (AdditionsRunLevel); Main/idl/VirtualBox.xidl (AdditionsFacilityType)`

#### Scenario: Acceptance for FR-CP07-001
- **WHEN** the FR-CP07-001 acceptance check is run
- **THEN** A VM without GA shows provider none with every feature 'available' plus an Install action; after GA is installed the Integration card updates within ≤ 5 s through Events.Watch without a refresh.

### Requirement: One-click Guest Tools install and upgrade (FR-CP07-002)
Virtax SHALL provide "One-click Guest Tools install and upgrade" as specified by FR-CP07-002 (BRD-08, priority Must, phase P1; Vietnamese: "Cài và nâng cấp Guest Tools một click").

#### Scenario: Specified behaviour of FR-CP07-002
- **WHEN** the behaviour FR-CP07-002 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Phase 1: when GA runs, upgrade through guest control (the UpdateAdditions mechanism) as an Operation; otherwise mount the GA ISO with OS-specific step-by-step guidance. For Linux guests whose kernel already ships vboxguest/vboxsf (mainline), prefer the distro's user-space package or install only VBoxService/VBoxClient, without building modules. A reboot requirement is reported.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/GuestSessionImplTasks.cpp:2979 (UpdateAdditions); Additions/linux/installer/vboxadd.sh:595-599`

#### Scenario: Acceptance for FR-CP07-002
- **WHEN** the FR-CP07-002 acceptance check is run
- **THEN** An Ubuntu 24.04 guest without gcc/headers installs Guest Tools within ≤ 90 s with a working clipboard; a GA 7.2.x upgrade shows %/ETA in the Task center.

### Requirement: Shared clipboard with modes and policy (FR-CP07-003)
Virtax SHALL provide "Shared clipboard with modes and policy" as specified by FR-CP07-003 (BRD-08, priority Must, phase P1; Vietnamese: "Shared clipboard có chế độ và chính sách").

#### Scenario: Specified behaviour of FR-CP07-003
- **WHEN** the behaviour FR-CP07-003 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Modes off / host→guest / guest→host / bidirectional, changeable live (Live badge). Supports Unicode text and images; clipboard file transfer is off by default. Virtax-created VMs default to bidirectional; imported VMs keep their setting. Clipboard contents are never logged.
- **AND** the VirtualBox reference for the current behaviour is `HostServices/SharedClipboard/*; GuestHost/SharedClipboard/*; Main/src-client/ConsoleImplConfigCommon.cpp:3994-4012`

#### Scenario: Acceptance for FR-CP07-003
- **WHEN** the FR-CP07-003 acceptance check is run
- **THEN** Copying 1 MB of text on the host pastes in the guest within ≤ 300 ms; a 4K PNG pastes into Paint/GIMP at the right size; switching to off takes effect immediately.

### Requirement: Host ↔ guest file drag-and-drop (FR-CP07-004)
Virtax SHALL provide "Host ↔ guest file drag-and-drop" as specified by FR-CP07-004 (BRD-08, priority Should, phase P1; Vietnamese: "Kéo-thả file host ↔ guest").

#### Scenario: Specified behaviour of FR-CP07-004
- **WHEN** the behaviour FR-CP07-004 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Dragging files/folders from the host onto the Console window copies them into the guest (default target Downloads/Desktop) and vice versa; each drag is an Operation with %, cancel and clear errors. Phase 1 uses GA DnD (host→guest on by default, guest→host optional); Phase 2 uses the agent's file copy.
- **AND** the VirtualBox reference for the current behaviour is `HostServices/DragAndDrop/*; Main/src-client/GuestDnDPrivate.cpp, GuestDnDTargetImpl.cpp, GuestDnDSourceImpl.cpp`

#### Scenario: Acceptance for FR-CP07-004
- **WHEN** the FR-CP07-004 acceptance check is run
- **THEN** Dragging a 500 MB / 2,000-file folder into an Ubuntu guest shows an ETA and finishes without errors; cancelling midway leaves no partial file under its real name.

### Requirement: Shared folders: live add/edit and safety checks (FR-CP07-005)
Virtax SHALL provide "Shared folders: live add/edit and safety checks" as specified by FR-CP07-005 (BRD-08, priority Must, phase P1; Vietnamese: "Shared folder: thêm/sửa live và kiểm tra an toàn").

#### Scenario: Specified behaviour of FR-CP07-005
- **WHEN** the behaviour FR-CP07-005 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Shared folders (name, host path, guest mount point, read-only, auto-mount, symlink policy) can be added/removed live on a running VM, persistent or transient. Checks: the path exists; warnings when sharing the whole home, `/`, `~/.ssh` or keychain folders; read-write by default only for user-picked folders. Phase 1 uses vbsf.
- **AND** the VirtualBox reference for the current behaviour is `HostServices/SharedFolders/vbsf.cpp; Main/src-all/SharedFolderImpl.cpp; Frontends/VirtualBox/src/settings/editors/UISharedFolderDetailsEditor.cpp`

#### Scenario: Acceptance for FR-CP07-005
- **WHEN** the FR-CP07-005 acceptance check is run
- **THEN** Adding ~/src to a running VM makes it appear at /mnt/src within ≤ 3 s; choosing ~ (the whole home) warns and suggests a subfolder; read-only blocks writes in the guest.

### Requirement: Shared folders over virtio-fs (Phase 2) (FR-CP07-006)
Virtax SHALL provide "Shared folders over virtio-fs (Phase 2)" as specified by FR-CP07-006 (BRD-08, priority Must, phase P2; Vietnamese: "Shared folder qua virtio-fs (Pha 2)").

#### Scenario: Specified behaviour of FR-CP07-006
- **WHEN** the behaviour FR-CP07-006 covers is exercised
- **THEN** it behaves as BRD-08 specifies: On virtax-vmm, shared folders use virtio-fs: Linux hosts run virtiofsd (Rust) as a sandboxed sidecar per share; macOS/Windows hosts use an in-process FUSE-over-virtio server in virtax-vmm (libkrun's passthrough fs as reference). Same UI and same `[integration].shared_folders`; the agent mounts by tag.
- **AND** the VirtualBox reference for the current behaviour is `HostServices/SharedFolders/vbsf.cpp:1320-1356,1456-1525; Devices/VMMDev/VMMDevHGCM.cpp:831-862`

#### Scenario: Acceptance for FR-CP07-006
- **WHEN** the FR-CP07-006 acceptance check is run
- **THEN** `git status` on a 50k-file repo over virtio-fs is ≥ 2× faster than vbsf on the same host; a host-side edit is visible in the guest within ≤ 1 s (cache=auto).

### Requirement: Guest display auto-resize (FR-CP07-007)
Virtax SHALL provide "Guest display auto-resize" as specified by FR-CP07-007 (BRD-08, priority Should, phase P1; Vietnamese: "Tự đổi độ phân giải guest").

#### Scenario: Specified behaviour of FR-CP07-007
- **WHEN** the behaviour FR-CP07-007 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Per-VM auto-resize toggle; when the Console window changes size or DPI, virtaxd sends a hint (Phase 1: IDisplay setVideoModeHint to GA/VMSVGA; Phase 2: virtio-gpu EDID/hotplug + agent). Adaptive 100-300 ms debounce instead of a fixed 300 ms; multi-monitor aware.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/runtime/normal/UIMachineViewNormal.cpp:70; Main/src-client/DisplayImpl.cpp; Additions/x11/VBoxClient (display)`

#### Scenario: Acceptance for FR-CP07-007
- **WHEN** the FR-CP07-007 acceptance check is run
- **THEN** Resizing the Console from 1280×800 to 2560×1440 @2× makes Ubuntu/Windows guests switch resolution within ≤ 500 ms after the mouse is released.

### Requirement: Guest info and IPs in the read model (FR-CP07-008)
Virtax SHALL provide "Guest info and IPs in the read model" as specified by FR-CP07-008 (BRD-08, priority Must, phase P1; Vietnamese: "Guest info và IP vào read model").

#### Scenario: Specified behaviour of FR-CP07-008
- **WHEN** the behaviour FR-CP07-008 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Per-NIC IPs, hostname, OS release and logged-in users enter the read model via events (Phase 1: GuestPropertyChanged on /VirtualBox/GuestInfo/*; Phase 2: pushed by the agent on change), with no polling. They feed the Overview header, 'Copy SSH command' and `virtaxctl guest ip`.
- **AND** the VirtualBox reference for the current behaviour is `HostServices/GuestProperties/*; Additions/common/VBoxService/VBoxServiceVMInfo.cpp`

#### Scenario: Acceptance for FR-CP07-008
- **WHEN** the FR-CP07-008 acceptance check is run
- **THEN** When the guest's DHCP IP changes, Overview and `virtaxctl guest ip --json` reflect it within ≤ 5 s; virtaxd makes no periodic guest-property enumerations (checked by tracing).

### Requirement: Pipelined exec and file copy (Phase 1) (FR-CP07-009)
Virtax SHALL provide "Pipelined exec and file copy (Phase 1)" as specified by FR-CP07-009 (BRD-08, priority Should, phase P1; Vietnamese: "Exec và chép file có pipeline (Pha 1)").

#### Scenario: Specified behaviour of FR-CP07-009
- **WHEN** the behaviour FR-CP07-009 covers is exercised
- **THEN** it behaves as BRD-08 specifies: virtaxd wraps IGuestSession: commands run with streamed stdout/stderr, exit code and timeout; file/folder copies are Operations that keep several requests in flight within API limits to beat the synchronous 64 KiB path. Guest credentials come from the OS keystore (CP-12), never `machine.toml`.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/GuestSessionImplTasks.cpp:420-425,753-758; Main/src-client/GuestFileImpl.cpp:1753-1754`

#### Scenario: Acceptance for FR-CP07-009
- **WHEN** the FR-CP07-009 acceptance check is run
- **THEN** Copying a 2 GB file into the guest via guest control is ≥ 2× faster than VBoxManage guestcontrol copyto on the same host; `virtaxctl guest exec` returns the right exit code and supports Ctrl-C.

### Requirement: Time sync right after resume/restore (FR-CP07-010)
Virtax SHALL provide "Time sync right after resume/restore" as specified by FR-CP07-010 (BRD-08, priority Should, phase P1; Vietnamese: "Đồng bộ giờ ngay sau resume/restore").

#### Scenario: Specified behaviour of FR-CP07-010
- **WHEN** the behaviour FR-CP07-010 covers is exercised
- **THEN** it behaves as BRD-08 specifies: After resume, snapshot restore or host wake from sleep, virtaxd requests an immediate sync (Phase 1: trigger VBoxService TimeSync via guest property/set-on-restore; Phase 2: the agent's time.sync command) instead of waiting for the 10 s poll cycle. Drift > 2 s is shown.
- **AND** the VirtualBox reference for the current behaviour is `Additions/common/VBoxService/VBoxServiceTimeSync.cpp:147,266-269,644`

#### Scenario: Acceptance for FR-CP07-010
- **WHEN** the FR-CP07-010 acceptance check is run
- **THEN** Restoring a 3-day-old snapshot leaves the guest clock within ±1 s within ≤ 2 s of resuming.

### Requirement: Per-VM integration policy and audit (FR-CP07-011)
Virtax SHALL provide "Per-VM integration policy and audit" as specified by FR-CP07-011 (BRD-08, priority Should, phase P1; Vietnamese: "Chính sách tích hợp theo VM và audit").

#### Scenario: Specified behaviour of FR-CP07-011
- **WHEN** the behaviour FR-CP07-011 covers is exercised
- **THEN** it behaves as BRD-08 specifies: `[integration]` in `machine.toml` holds clipboard, dnd, shared_folders, exec_allowed and file_copy_allowed. Virtax-created VMs allow exec/files; imported VMs keep them off until the user enables them. Every exec/file copy is written to the VM audit log (command, user, time, result; never contents).
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/GuestSessionImpl.cpp; Frontends/VirtualBox/src/settings/machine/UIMachineSettingsGeneral.cpp:434-665 (Features tab)`

#### Scenario: Acceptance for FR-CP07-011
- **WHEN** the FR-CP07-011 acceptance check is run
- **THEN** An imported VM with exec_allowed=false makes `virtaxctl guest exec` return VTX-GST-010 'disabled by policy' with the enabling command; the audit log has one line per exec.

### Requirement: Integration migration on 'Move to virtax-vmm' (FR-CP07-012)
Virtax SHALL provide "Integration migration on 'Move to virtax-vmm'" as specified by FR-CP07-012 (BRD-08, priority Must, phase P2; Vietnamese: "Chuyển tích hợp khi 'Move to virtax-vmm'").

#### Scenario: Specified behaviour of FR-CP07-012
- **WHEN** the behaviour FR-CP07-012 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Preflight while the VM still runs on the legacy engine: install virtax-agent and virtio drivers (Windows: virtio-win; Linux: check in-box virtio modules), map shared folders to virtio-fs, and only then switch engines. Once stable on virtax-vmm, Virtax offers to remove GA (Linux: clean up the /etc/depmod.d override). Rollback keeps GA intact.
- **AND** the VirtualBox reference for the current behaviour is `Additions/linux/installer/vboxadd.sh:595-599; analysis 05 §5.3 (virtio-era mapping)`

#### Scenario: Acceptance for FR-CP07-012
- **WHEN** the FR-CP07-012 acceptance check is run
- **THEN** A Windows 11 VM with GA goes through 'Move to virtax-vmm', which installs virtio-win + agent and boots virtax-vmm with working network, clipboard and auto-resize; rolling back to legacy keeps GA working.
