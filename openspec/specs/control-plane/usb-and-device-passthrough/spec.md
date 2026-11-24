# USB & device passthrough Specification

## Purpose
USB filters, live attach/detach, USB/IP, VFIO on Linux (P3), webcam/smart card. Module `CP-08` (USB & device passthrough; Vietnamese: "USB & passthrough thiết bị") from BRD-08, phase P2, priority Could. Owner crates: virtaxd, virtax-helper. Depends on: `platform/host-integration-and-power`, `engine/device-model-and-firmware`. VirtualBox reference: `src/VBox/Main/src-server/USBProxyService.cpp, HostDrivers/VBoxUSB, Devices/USB`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Preserve legacy VMs' USB configuration (preparatory) (FR-CP08-001)
Virtax SHALL provide "Preserve legacy VMs' USB configuration (preparatory)" as specified by FR-CP08-001 (BRD-08, priority Should, phase P1; Vietnamese: "Giữ nguyên cấu hình USB của VM legacy (chuẩn bị)").

#### Scenario: Specified behaviour of FR-CP08-001
- **WHEN** the behaviour FR-CP08-001 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Phase-1 preparatory work: virtaxd reads and shows, read-only, the USB controllers and filters of VirtualBox VMs and never drops them when saving other settings; USB keeps working through the legacy engine as before.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/USBDeviceFiltersImpl.cpp; include/VBox/settings.h:114-132`

#### Scenario: Acceptance for FR-CP08-001
- **WHEN** the FR-CP08-001 acceptance check is run
- **THEN** A VM with 3 USB filters: change RAM in Virtax, reopen in the VirtualBox GUI, and all 3 filters are intact.

### Requirement: Event-driven host USB inventory (FR-CP08-002)
Virtax SHALL provide "Event-driven host USB inventory" as specified by FR-CP08-002 (BRD-08, priority Should, phase P2; Vietnamese: "Danh sách thiết bị USB host theo event").

#### Scenario: Specified behaviour of FR-CP08-002
- **WHEN** the behaviour FR-CP08-002 covers is exercised
- **THEN** it behaves as BRD-08 specifies: A host device list (VID:PID, manufacturer, name, serial, speed, port path, state: available / attached to VM X / used by host / busy) updated from hotplug events: udev netlink (Linux), IOKit notifications (macOS), CM_Register_Notification (Windows); no periodic polling.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/linux/USBProxyBackendLinux.cpp:323-340; Main/src-server/darwin/USBProxyBackendDarwin.cpp:141-148`

#### Scenario: Acceptance for FR-CP08-002
- **WHEN** the FR-CP08-002 acceptance check is run
- **THEN** Plugging in a USB device lists it within ≤ 1 s on 3 OSes; with no plug/unplug, virtaxd has 0 USB-related wakeups/s.

### Requirement: USB filters created from devices, with conflict checks (FR-CP08-003)
Virtax SHALL provide "USB filters created from devices, with conflict checks" as specified by FR-CP08-003 (BRD-08, priority Should, phase P2; Vietnamese: "USB filter tạo từ thiết bị, có xung đột").

#### Scenario: Specified behaviour of FR-CP08-003
- **WHEN** the behaviour FR-CP08-003 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Create a filter from a plugged-in device in one click (VID/PID/serial prefilled) or edit fields with wildcards; actions auto-attach / ask / ignore. A global 'never capture' list (the host's only keyboard/mouse, mounted drives). A warning when two running VMs have filters matching the same device.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/USBDeviceFilterImpl.cpp; Frontends/VirtualBox/src/settings/editors/UIUSBFilterDetailsEditor.cpp`

#### Scenario: Acceptance for FR-CP08-003
- **WHEN** the FR-CP08-003 acceptance check is run
- **THEN** Creating a filter from a USB-serial board takes ≤ 2 interactions; a filter matching 2 running VMs warns with both VM names before saving.

### Requirement: Live USB attach/detach (FR-CP08-004)
Virtax SHALL provide "Live USB attach/detach" as specified by FR-CP08-004 (BRD-08, priority Should, phase P2; Vietnamese: "Attach/detach USB live").

#### Scenario: Specified behaviour of FR-CP08-004
- **WHEN** the behaviour FR-CP08-004 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Attach/detach from the Console USB chip or from Overview; the legacy engine uses IConsole::attachUSBDevice/detachUSBDevice, virtax-vmm uses FR-CP08-006…008 (Phase 3). The device always returns to the host on detach, VM power-off or crash; 'device in use by the host' errors have a quick fix.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/ConsoleImpl.cpp (attachUSBDevice); Main/src-server/USBProxyService.cpp`

#### Scenario: Acceptance for FR-CP08-004
- **WHEN** the FR-CP08-004 acceptance check is run
- **THEN** Attaching a device takes ≤ 2 s; killing the VM process makes the device reappear on the host within ≤ 3 s.

### Requirement: xHCI-only controller policy (FR-CP08-005)
Virtax SHALL provide "xHCI-only controller policy" as specified by FR-CP08-005 (BRD-08, priority Should, phase P2; Vietnamese: "Chính sách chỉ dùng xHCI").

#### Scenario: Specified behaviour of FR-CP08-005
- **WHEN** the behaviour FR-CP08-005 covers is exercised
- **THEN** it behaves as BRD-08 specifies: New VMs only get xHCI (USB 3, doorbell-driven, no 1 kHz frame timers); legacy VMs with OHCI/EHCI are offered a switch to xHCI with a driver warning for very old guests. On virtax-vmm, xHCI comes from EN-04 and serves the HID tablet, USB-MSD and passthrough.
- **AND** the VirtualBox reference for the current behaviour is `Devices/USB/DevOHCI.cpp:4552; DevEHCI.cpp:931-932; DevXHCI.cpp:2161,5252-5269`

#### Scenario: Acceptance for FR-CP08-005
- **WHEN** the FR-CP08-005 acceptance check is run
- **THEN** Switching a VM from OHCI+EHCI to xHCI cuts idle host wakeups by ≥ 500/s (powertop/ETW) with the tablet pointer still working.

### Requirement: USB passthrough on Linux hosts (usbfs) (FR-CP08-006)
Virtax SHALL provide "USB passthrough on Linux hosts (usbfs)" as specified by FR-CP08-006 (BRD-08, priority Could, phase P3; Vietnamese: "Passthrough USB trên Linux host (usbfs)").

#### Scenario: Specified behaviour of FR-CP08-006
- **WHEN** the behaviour FR-CP08-006 covers is exercised
- **THEN** it behaves as BRD-08 specifies: virtax-vmm uses usbfs (USBDEVFS_*) via rusb/libusb; virtax-helper grants per-device access (a temporary ACL on /dev/bus/usb/BBB/DDD) with no global group or custom udev rules; the kernel driver is detached on attach and reattached on release.
- **AND** the VirtualBox reference for the current behaviour is `Devices/USB/linux/USBProxyDevice-linux.cpp:1057,1313; Installer/linux/VBoxCreateUSBNode.sh`

#### Scenario: Acceptance for FR-CP08-006
- **WHEN** the FR-CP08-006 acceptance check is run
- **THEN** An FTDI USB-serial device works in an Ubuntu guest on virtax-vmm with `screen /dev/ttyUSB0`; on detach the host can use it again immediately and the ACL is removed.

### Requirement: USB passthrough on macOS (IOKit) (FR-CP08-007)
Virtax SHALL provide "USB passthrough on macOS (IOKit)" as specified by FR-CP08-007 (BRD-08, priority Could, phase P3; Vietnamese: "Passthrough USB trên macOS (IOKit)").

#### Scenario: Specified behaviour of FR-CP08-007
- **WHEN** the behaviour FR-CP08-007 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Uses IOUSBHost/IOKit with the `com.apple.security.device.usb` entitlement; requests exclusive access and explains when another app holds the device; releases the device when the VM stops.
- **AND** the VirtualBox reference for the current behaviour is `Devices/USB/darwin/USBProxyDevice-darwin.cpp:38-41`

#### Scenario: Acceptance for FR-CP08-007
- **WHEN** the FR-CP08-007 acceptance check is run
- **THEN** A YubiKey reaches and works in a Windows 11 arm64 guest on an M-series Mac; when a host app holds it, the message names the app.

### Requirement: Windows USB without dispatch-table patching (FR-CP08-008)
Virtax SHALL provide "Windows USB without dispatch-table patching" as specified by FR-CP08-008 (BRD-08, priority Could, phase P3; Vietnamese: "USB trên Windows không vá dispatch table").

#### Scenario: Specified behaviour of FR-CP08-008
- **WHEN** the behaviour FR-CP08-008 covers is exercised
- **THEN** it behaves as BRD-08 specifies: No VBoxUSBMon-style driver (swapping usbhub MajorFunction pointers). The main path is USB/IP via usbipd-win (the helper binds/attaches the device to virtax-vmm over localhost); the fallback is the helper assigning WinUSB to the chosen device and restoring the original driver on release.
- **AND** the VirtualBox reference for the current behaviour is `HostDrivers/VBoxUSB/win/mon/VBoxUsbHook.cpp:61; VBoxUsbMon.cpp:871; Devices/USB/win/`

#### Scenario: Acceptance for FR-CP08-008
- **WHEN** the FR-CP08-008 acceptance check is run
- **THEN** On Windows 11 with HVCI on, a USB-serial device attaches to an virtax-vmm VM; after detach Device Manager shows the original driver again; Virtax installs no kernel driver of its own.

### Requirement: USB/IP client for remote devices (FR-CP08-009)
Virtax SHALL provide "USB/IP client for remote devices" as specified by FR-CP08-009 (BRD-08, priority Could, phase P3; Vietnamese: "USB/IP client cho thiết bị từ xa").

#### Scenario: Specified behaviour of FR-CP08-009
- **WHEN** the behaviour FR-CP08-009 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Add USB/IP servers (Linux usbipd, usbipd-win) by host:port, browse exported devices and attach them to a VM like local ones; reused for the Windows path in FR-CP08-008.
- **AND** the VirtualBox reference for the current behaviour is `Devices/USB/usbip/USBProxyDevice-usbip.cpp:57-66,737-753`

#### Scenario: Acceptance for FR-CP08-009
- **WHEN** the FR-CP08-009 acceptance check is run
- **THEN** A device exported from another Linux machine on the LAN attaches to a VM within ≤ 3 s and works over a < 1 ms LAN RTT.

### Requirement: Webcam and smart card (FR-CP08-010)
Virtax SHALL provide "Webcam and smart card" as specified by FR-CP08-010 (BRD-08, priority Could, phase P2; Vietnamese: "Webcam và smart card").

#### Scenario: Specified behaviour of FR-CP08-010
- **WHEN** the behaviour FR-CP08-010 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Webcam: an emulated UVC device fed by the host camera (V4L2 / AVFoundation / Media Foundation) with camera and resolution choice and an indicator in the Console. Smart card: an emulated CCID reader bridged to host PC/SC. Phase 2 via the legacy engine (emulated USB), Phase 3 on virtax-vmm.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Video/UsbWebcam.cpp, HostWebcam-*.cpp; Devices/Security/UsbCardReader.cpp; Main/src-client/EmulatedUSBImpl.cpp`

#### Scenario: Acceptance for FR-CP08-010
- **WHEN** the FR-CP08-010 acceptance check is run
- **THEN** The host webcam appears in Teams/Zoom in a Windows 11 guest at 720p ≥ 25 fps; a PIV card is readable in the guest through the emulated reader.

### Requirement: VFIO PCI passthrough on Linux (FR-CP08-011)
Virtax SHALL provide "VFIO PCI passthrough on Linux" as specified by FR-CP08-011 (BRD-08, priority Could, phase P3; Vietnamese: "VFIO PCI passthrough trên Linux").

#### Scenario: Specified behaviour of FR-CP08-011
- **WHEN** the behaviour FR-CP08-011 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Expert mode: list PCI devices by IOMMU group, check that the IOMMU is on, the whole group can be detached and the device supports reset; the helper binds vfio-pci and restores the original driver when the VM stops. Clear warnings when the device is a GPU/NIC in use by the host.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Bus/DevPciVfio.cpp; Config.kmk:815-818; Main/src-server/HostPCIDeviceImpl.cpp`

#### Scenario: Acceptance for FR-CP08-011
- **WHEN** the FR-CP08-011 acceptance check is run
- **THEN** A second PCIe NIC passed through to a Linux guest reaches line rate; a group containing the host's primary GPU is blocked with an explanation.

### Requirement: Passthrough safety (FR-CP08-012)
Virtax SHALL provide "Passthrough safety" as specified by FR-CP08-012 (BRD-08, priority Should, phase P2; Vietnamese: "An toàn khi passthrough").

#### Scenario: Specified behaviour of FR-CP08-012
- **WHEN** the behaviour FR-CP08-012 covers is exercised
- **THEN** it behaves as BRD-08 specifies: Never capture: the host boot disk's controller, the only keyboard/mouse, devices on the 'never capture' list. The first attach of a device asks for confirmation; devices auto-return when the VM stops/crashes; every attach/detach is audited.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/USBProxyService.cpp; Main/src-server/HostUSBDeviceImpl.cpp`

#### Scenario: Acceptance for FR-CP08-012
- **WHEN** the FR-CP08-012 acceptance check is run
- **THEN** Trying to attach the laptop's only keyboard is blocked with VTX-USB-010; crashing a VM holding 3 devices returns all 3 to the host within ≤ 3 s.
