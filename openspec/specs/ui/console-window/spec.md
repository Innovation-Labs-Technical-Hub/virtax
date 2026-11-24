# Console window Specification

## Purpose
wgpu guest display, input capture, window/fullscreen/scaled/seamless modes, mini toolbar, status chips, soft keyboard. Module `UI-05` (Console window; Vietnamese: "Cửa sổ Console") from BRD-04, phase P1, priority Must. Owner crates: virtax-console. Depends on: `engine/display-and-input-data-plane`, `control-plane/control-plane-api-and-ipc`. VirtualBox reference: `src/VBox/Frontends/VirtualBox/src/runtime, softkeyboard, platform/*/Keyboard*`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: wgpu surface with dirty-rect uploads (FR-UI05-001)
Virtax SHALL provide "wgpu surface with dirty-rect uploads" as specified by FR-UI05-001 (BRD-04, priority Must, phase P1; Vietnamese: "Surface wgpu chỉ upload dirty rect").

#### Scenario: Specified behaviour of FR-UI05-001
- **WHEN** the behaviour FR-UI05-001 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Each guest screen is a wgpu texture; the console uploads exactly the dirty rects with `Queue::write_texture` using the ring stride (no 256-byte alignment needed), scales on the GPU (nearest for integer ratios, linear otherwise, configurable), and presents with FIFO by default and Mailbox in "Low latency" mode. Without a GPU (RDP, nested VMs) it uses wgpu's fallback adapter (WARP on Windows, llvmpipe/lavapipe on Linux).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIFrameBuffer.cpp:1269-1470`

#### Scenario: Acceptance for FR-UI05-001
- **WHEN** the FR-UI05-001 acceptance check is run
- **THEN** 1080p60 full motion: console CPU ≤ 3% with 0 CPU copies besides the upload (byte counters); a GPU-less RDP session still shows ≥ 30 fps at 1080p.

### Requirement: 0% CPU when the screen is static or hidden (FR-UI05-002)
Virtax SHALL provide "0% CPU when the screen is static or hidden" as specified by FR-UI05-002 (BRD-04, priority Must, phase P1; Vietnamese: "0% CPU khi màn hình đứng yên hoặc bị ẩn").

#### Scenario: Specified behaviour of FR-UI05-002
- **WHEN** the behaviour FR-UI05-002 covers is exercised
- **THEN** it behaves as BRD-04 specifies: The console thread sleeps on the eventfd/socket and renders only on a notify, a cursor update or chrome interaction. When occluded, minimized or on another workspace it stops uploading and tells the adapter to slow down; the macOS Dock icon updates ≤ 1 Hz (can be off) instead of on every paint.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIMachineView.cpp:1979-1990`

#### Scenario: Acceptance for FR-UI05-002
- **WHEN** the FR-UI05-002 acceptance check is run
- **THEN** With a static guest for 60 s the console renders 0 frames at ≤ 1 wakeup/s and ≈ 0% CPU (measured with `powermetrics`/`perf`/ETW); minimizing stops adapter copies within ≤ 100 ms.

### Requirement: Follow-host HiDPI by default (FR-UI05-003)
Virtax SHALL provide "Follow-host HiDPI by default" as specified by FR-UI05-003 (BRD-04, priority Must, phase P1; Vietnamese: "HiDPI mặc định theo host").

#### Scenario: Specified behaviour of FR-UI05-003
- **WHEN** the behaviour FR-UI05-003 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Default is "Follow host": when the guest supports auto-resize, the console sends a size hint in physical pixels and the scale factor of the monitor holding the window; otherwise it scales on the GPU. Options: follow host / 100% / 200% / integer only (retro guests) / custom, stored per VM in `ui_prefs`; moving to a monitor with another DPR is handled automatically.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/extradata/UIExtraDataManager.cpp:2513-2531`

#### Scenario: Acceptance for FR-UI05-003
- **WHEN** the FR-UI05-003 acceptance check is run
- **THEN** On a 200% display, an Ubuntu guest with Guest Additions gets the physical resolution within ≤ 1 s with crisp text (image diff); moving to a 100% display sends a new hint within ≤ 300 ms.

### Requirement: Window / fullscreen / scaled without window re-creation (FR-UI05-004)
Virtax SHALL provide "Window / fullscreen / scaled without window re-creation" as specified by FR-UI05-004 (BRD-04, priority Must, phase P1; Vietnamese: "Window / fullscreen / scaled không tạo lại cửa sổ").

#### Scenario: Specified behaviour of FR-UI05-004
- **WHEN** the behaviour FR-UI05-004 covers is exercised
- **THEN** it behaves as BRD-04 specifies: The mode manager reconfigures the same native window and wgpu surface (borderless or OS-native fullscreen; scaled = fit to window, no scrollbars); capture state, chips and toolbar position are preserved. Host+F (fullscreen), Host+C (scaled) and Host+A (fit window) keep their VirtualBox meanings.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIMachine.cpp:1150-1180`

#### Scenario: Acceptance for FR-UI05-004
- **WHEN** the FR-UI05-004 acceptance check is run
- **THEN** Cycling the 3 modes 100 times keeps the same native window handle, each switch ≤ 200 ms with 0 black frames (120 fps screen capture).

### Requirement: Seamless mode (FR-UI05-005)
Virtax SHALL provide "Seamless mode" as specified by FR-UI05-005 (BRD-04, priority Should, phase P1; Vietnamese: "Seamless mode").

#### Scenario: Specified behaviour of FR-UI05-005
- **WHEN** the behaviour FR-UI05-005 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Host+L shows only guest windows on the host desktop using the guest's visible region: `SetWindowRgn` on Windows, XShape on X11, a transparent window on macOS, surface alpha + input region on Wayland; enabled only when Guest Additions report support.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/seamless/UIMachineWindowSeamless.cpp:383-410`

#### Scenario: Acceptance for FR-UI05-005
- **WHEN** the FR-UI05-005 acceptance check is run
- **THEN** A Windows 11 guest with GA enters seamless in ≤ 300 ms, clicks pass through empty areas to the host desktop, and the region updates within 1 frame when a guest window opens.

### Requirement: Visual multi-monitor mapping (FR-UI05-006)
Virtax SHALL provide "Visual multi-monitor mapping" as specified by FR-UI05-006 (BRD-04, priority Should, phase P1; Vietnamese: "Ánh xạ đa màn hình trực quan").

#### Scenario: Specified behaviour of FR-UI05-006
- **WHEN** the behaviour FR-UI05-006 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Multi-monitor VMs (up to 8 in P1): each guest screen is its own window/surface; a drag-and-drop mapper assigns guest screens to host monitors with per-monitor fullscreen, stored per VM in `ui_prefs`; plugging/unplugging a host monitor proposes a new mapping.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIMultiScreenLayout.cpp`

#### Scenario: Acceptance for FR-UI05-006
- **WHEN** the FR-UI05-006 acceptance check is run
- **THEN** A 2-screen VM on a 2-monitor host returns to the saved fullscreen mapping after restart; unplugging one host monitor moves its guest window to the remaining monitor within ≤ 1 s.

### Requirement: One mini toolbar for every mode (FR-UI05-007)
Virtax SHALL provide "One mini toolbar for every mode" as specified by FR-UI05-007 (BRD-04, priority Must, phase P1; Vietnamese: "Mini toolbar thống nhất cho mọi mode").

#### Scenario: Specified behaviour of FR-UI05-007
- **WHEN** the behaviour FR-UI05-007 covers is exercised
- **THEN** it behaves as BRD-04 specifies: One shared floating toolbar: Pin, Pause, Reset, Shut down ▾, Send keys ▾, Mode ▾, Screenshot, Fullscreen. Docked at the top in window mode; auto-hidden in fullscreen/seamless and revealed after a 300 ms hover on the top edge or Host+Home; keyboard operable, position remembered in `ui_prefs`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/widgets/UIMiniToolBar.cpp:308-349`

#### Scenario: Acceptance for FR-UI05-007
- **WHEN** the FR-UI05-007 acceptance check is run
- **THEN** The same toolbar (screenshot diff) appears in all 4 modes; every button has an accessible name; Host+Home opens it and focuses the first button.

### Requirement: Status chips with device menus (FR-UI05-008)
Virtax SHALL provide "Status chips with device menus" as specified by FR-UI05-008 (BRD-04, priority Must, phase P1; Vietnamese: "Status chip có menu thiết bị").

#### Scenario: Specified behaviour of FR-UI05-008
- **WHEN** the behaviour FR-UI05-008 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Replace 13 tiny 16 px indicators with labelled chips: Disk (activity), Optical (mount/eject ISO), Network (plug/unplug cable, change attachment), USB (device list), Shared folders, Display (resolution × scale), Audio (mic on/off), Clipboard (mode and last transfer), Engine (KVM/HVF/WHP/HM). Activity arrives as pushed events at ≤ 4 Hz and only while the chip is visible.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIIndicatorsPool.cpp:264-1459,1549-1592`

#### Scenario: Acceptance for FR-UI05-008
- **WHEN** the FR-UI05-008 acceptance check is run
- **THEN** Mounting an ISO from the Optical chip on a running VM takes ≤ 2 clicks; 0 `GetDeviceActivity` polling calls; hiding the status bar ends the daemon subscription within ≤ 1 s.

### Requirement: Clear capture model, always-visible chip, host-key onboarding (FR-UI05-009)
Virtax SHALL provide "Clear capture model, always-visible chip, host-key onboarding" as specified by FR-UI05-009 (BRD-04, priority Must, phase P1; Vietnamese: "Mô hình capture rõ ràng, chip luôn thấy, onboarding host key").

#### Scenario: Specified behaviour of FR-UI05-009
- **WHEN** the behaviour FR-UI05-009 covers is exercised
- **THEN** it behaves as BRD-04 specifies: States: Not captured / Keyboard captured / Pointer locked (relative). While captured a chip always shows "Input captured — press ⟨Host⟩ to release" (bilingual text with an icon); click-to-capture when there is no mouse integration; losing focus releases immediately. The first console launch shows a one-step host-key overlay (Right Ctrl on Windows/Linux, left ⌘ on macOS) that can be changed by pressing a key; laptops without Right Ctrl get an alternative suggested.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIKeyboardHandler.cpp:490,1233; UIExtraDataManager.cpp:853-866`

#### Scenario: Acceptance for FR-UI05-009
- **WHEN** the FR-UI05-009 acceptance check is run
- **THEN** A 20-new-user test: ≥ 90% release input on the first try; switching apps via the OS while captured releases capture within ≤ 50 ms; screen readers announce the chip.

### Requirement: Per-platform capture backends (FR-UI05-010)
Virtax SHALL provide "Per-platform capture backends" as specified by FR-UI05-010 (BRD-04, priority Must, phase P1; Vietnamese: "Backend capture theo nền tảng").

#### Scenario: Specified behaviour of FR-UI05-010
- **WHEN** the behaviour FR-UI05-010 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Windows: a `WH_KEYBOARD_LL` hook only while captured and focused, Raw Input + `ClipCursor` for relative mouse. macOS: `CGEventTap` (guided Input Monitoring permission) for ⌘-Tab, ⌘-Space; pointer lock via `CGAssociateMouseAndMouseCursorPosition`. X11: keyboard/pointer grabs + XInput2 raw motion. Wayland: `zwp_keyboard_shortcuts_inhibit_manager_v1`, `zwp_pointer_constraints_v1` (lock), `zwp_relative_pointer_manager_v1`; when a protocol is missing, explain and offer XWayland.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIKeyboardHandler.cpp:526-579,1415,2265-2279`

#### Scenario: Acceptance for FR-UI05-010
- **WHEN** the FR-UI05-010 acceptance check is run
- **THEN** Test matrix: Alt+Tab/Win/Super reach the guest while captured on Windows 11, macOS 14/15, Ubuntu 24.04 X11, GNOME 46 and KDE 6 Wayland; relative mouse in a guest game does not drift on Wayland.

### Requirement: Physical-key translation without stuck keys (FR-UI05-011)
Virtax SHALL provide "Physical-key translation without stuck keys" as specified by FR-UI05-011 (BRD-04, priority Must, phase P1; Vietnamese: "Dịch phím vật lý, không kẹt phím").

#### Scenario: Specified behaviour of FR-UI05-011
- **WHEN** the behaviour FR-UI05-011 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Keys are sent by physical position (scancode set 1 or HID usage depending on the engine), independent of the host layout; IME is disabled while captured; every held key is released (`releaseKeys`) on capture release, focus loss or window close; Caps/Num/Scroll LEDs sync from `KeyboardLedsChanged` events.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/platform/win/WinKeyboard.cpp; platform/nix/XKeyboard-new.cpp`

#### Scenario: Acceptance for FR-UI05-011
- **WHEN** the FR-UI05-011 acceptance check is run
- **THEN** 1,000 cycles of holding Shift and Alt-Tabbing out leave 0 stuck keys in the guest; a Vietnamese Telex or AZERTY host layout does not garble keys in a US guest.

### Requirement: Send keys palette and type text (FR-UI05-012)
Virtax SHALL provide "Send keys palette and type text" as specified by FR-UI05-012 (BRD-04, priority Must, phase P1; Vietnamese: "Palette Send keys và gõ văn bản").

#### Scenario: Specified behaviour of FR-UI05-012
- **WHEN** the behaviour FR-UI05-012 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Ctrl/⌘-Shift-K opens the palette: Ctrl+Alt+Del, Ctrl+Alt+F1…F12, Ctrl+Alt+Backspace, Print Screen, SysRq, Super and custom sequences saved per VM; "Type text" sends host clipboard content as keystrokes (rate-limited, mapped to the guest layout) for guests without clipboard support.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIActionPoolRuntime.cpp (Host+Del, Host+Backspace)`

#### Scenario: Acceptance for FR-UI05-012
- **WHEN** the FR-UI05-012 acceptance check is run
- **THEN** Ctrl+Alt+Del reaches a Windows guest in ≤ 2 actions; typing 200 ASCII characters into a guest installer has 0 wrong characters.

### Requirement: Mouse integration and hardware cursor (FR-UI05-013)
Virtax SHALL provide "Mouse integration and hardware cursor" as specified by FR-UI05-013 (BRD-04, priority Must, phase P1; Vietnamese: "Mouse integration và con trỏ phần cứng").

#### Scenario: Specified behaviour of FR-UI05-013
- **WHEN** the behaviour FR-UI05-013 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Absolute pointing (Guest Additions, USB tablet; virtio-input in P2) is the default, so no pointer capture is needed. The guest cursor shape from the cursor channel becomes the OS cursor (hardware composited); only relative mode draws the cursor into the texture.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIMachine.cpp:2045-2240; UIMouseHandler.cpp:189`

#### Scenario: Acceptance for FR-UI05-013
- **WHEN** the FR-UI05-013 acceptance check is run
- **THEN** Moving the mouse over a static guest causes 0 texture uploads (only the OS cursor moves); guest cursor shape changes show on the host within 1 frame.

### Requirement: Soft keyboard (FR-UI05-014)
Virtax SHALL provide "Soft keyboard" as specified by FR-UI05-014 (BRD-04, priority Could, phase P1; Vietnamese: "Soft keyboard").

#### Scenario: Specified behaviour of FR-UI05-014
- **WHEN** the behaviour FR-UI05-014 covers is exercised
- **THEN** it behaves as BRD-04 specifies: An overlay virtual keyboard (design-system KeyCap components) with US, ISO, JIS… layouts, sticky modifiers, numpad and media keys; can import VirtualBox's XML layout files; useful on touch devices and for keys the host lacks.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/softkeyboard/UISoftKeyboard.cpp; Frontends/VirtualBox/xml/*.xml`

#### Scenario: Acceptance for FR-UI05-014
- **WHEN** the FR-UI05-014 acceptance check is run
- **THEN** All 14 VirtualBox XML layouts import; pressing Ctrl+Alt+F2 with sticky keys on the soft keyboard switches TTY in a Linux guest.

### Requirement: Screenshots and a recording hook (FR-UI05-015)
Virtax SHALL provide "Screenshots and a recording hook" as specified by FR-UI05-015 (BRD-04, priority Should, phase P1; Vietnamese: "Screenshot và điểm móc recording").

#### Scenario: Specified behaviour of FR-UI05-015
- **WHEN** the behaviour FR-UI05-015 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Screenshots (Host+E or the toolbar button) read the current texture once, save a PNG and copy it to the clipboard, optionally with the cursor. The console exposes `FrameTap` (frame + damage stream) as the hook for a Phase-2 recorder using OS encoders (Media Foundation, VideoToolbox, VA-API); P1 ships the API without an encoder.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIActionPoolRuntime.cpp:1018 (TakeScreenshot)`

#### Scenario: Acceptance for FR-UI05-015
- **WHEN** the FR-UI05-015 acceptance check is run
- **THEN** A 4K screenshot reaches file + clipboard in ≤ 300 ms and matches the source pixels (image diff); `FrameTap` adds no CPU with no subscriber.

### Requirement: Lifecycle overlays instead of dialogs (FR-UI05-016)
Virtax SHALL provide "Lifecycle overlays instead of dialogs" as specified by FR-UI05-016 (BRD-04, priority Should, phase P1; Vietnamese: "Overlay vòng đời thay dialog").

#### Scenario: Specified behaviour of FR-UI05-016
- **WHEN** the behaviour FR-UI05-016 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Paused → a dimmed layer + Resume (replacing R-20, GPU-scaled); VM off → keep the last frame + Start/Close; boot failure → a "No bootable device" overlay with "Mount ISO and retry" (replacing R-16); closing the window follows the remembered choice: keep running in background, save state, ACPI, power off (replacing R-15).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIBootFailureDialog.cpp:94-109; UIVMCloseDialog.cpp:143-182`

#### Scenario: Acceptance for FR-UI05-016
- **WHEN** the FR-UI05-016 acceptance check is run
- **THEN** Closing the console with "Keep running" leaves the VM Running and reopening shows a full screen within ≤ 1 s; a boot failure is fixed by mounting an ISO and rebooting in ≤ 3 actions.
