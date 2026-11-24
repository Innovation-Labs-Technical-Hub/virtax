# Display & input data plane Specification

## Purpose
Triple-buffered SHM ring + damage rects + cursor; direct input channel; P2 GPU handles (dmabuf/IOSurface/DXGI). Module `EN-02` (Display & input data plane; Vietnamese: "Data plane hiển thị & input") from BRD-04, phase P1, priority Must. Owner crates: virtax-frame, vbx-shmfb (C++ in-tree adapter). Depends on: `engine/engine-trait-and-legacy-adapter`. VirtualBox reference: `src/VBox/Main/src-client/DisplayImpl.cpp, DisplaySourceBitmapImpl.cpp, Frontends/VBoxSDL/Framebuffer.cpp`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: `vbx-shmfb` adapter inside VBoxHeadless (FR-EN02-001)
Virtax SHALL provide "`vbx-shmfb` adapter inside VBoxHeadless" as specified by FR-EN02-001 (BRD-04, priority Must, phase P1; Vietnamese: "Adapter `vbx-shmfb` trong VBoxHeadless").

#### Scenario: Specified behaviour of FR-EN02-001
- **WHEN** the behaviour FR-EN02-001 covers is exercised
- **THEN** it behaves as BRD-04 specifies: An in-tree GPLv3 C++ adapter (≤ 2k LOC) built into the Engine Pack and enabled by a VBoxHeadless launch argument passed by `virtaxd`. It implements `IFramebuffer` inside the VM process without the `UpdateImage` capability, reads VRAM through `IDisplay::querySourceBitmap` → `IDisplaySourceBitmap::queryBitmapInfo`, and runs in the signed hardened process like stock VBoxHeadless.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VBoxSDL/Framebuffer.cpp:342-468; VBoxHeadless/VBoxHeadless.cpp:1286`

#### Scenario: Acceptance for FR-EN02-001
- **WHEN** the FR-EN02-001 acceptance check is run
- **THEN** The hardened Engine Pack build starts VMs with the adapter on all 6 host combos; 4K@60 reaches the console with ≤ 1 frame added latency; code review finds 0 dependencies outside Main/IPRT.

### Requirement: DisplayChannel setup and handle passing (FR-EN02-002)
Virtax SHALL provide "DisplayChannel setup and handle passing" as specified by FR-EN02-002 (BRD-04, priority Must, phase P1; Vietnamese: "Thiết lập DisplayChannel và chuyển handle").

#### Scenario: Specified behaviour of FR-EN02-002
- **WHEN** the behaviour FR-EN02-002 covers is exercised
- **THEN** it behaves as BRD-04 specifies: `Runtime.OpenDisplay(machine, screen, mode)` returns `DisplayChannel{endpoint, token, width, height, stride, format, protocol_version}`. The console connects to the adapter endpoint (a UDS in a 0700 folder / a named pipe with a per-SID DACL), presents the token and receives the SHM: an fd via `SCM_RIGHTS` (Linux, macOS) or a handle `DuplicateHandle`d into the console process (Windows). No SHM object has a public name.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/DisplayImpl.cpp:925-1006 (remote UpdateImage path)`

#### Scenario: Acceptance for FR-EN02-002
- **WHEN** the FR-EN02-002 acceptance check is run
- **THEN** `OpenDisplay` to first frame ≤ 300 ms; another user's process cannot open the endpoint or the SHM (permission tests on 3 OSes).

### Requirement: SHM ring layout v1 (FR-EN02-003)
Virtax SHALL provide "SHM ring layout v1" as specified by FR-EN02-003 (BRD-04, priority Must, phase P1; Vietnamese: "Bố cục SHM ring v1").

#### Scenario: Specified behaviour of FR-EN02-003
- **WHEN** the behaviour FR-EN02-003 covers is exercised
- **THEN** it behaves as BRD-04 specifies: A 4 KiB header (magic `VXFB`, version, generation, size, stride, BGRA8 format, slot count = 3, `write_seq`, `reader_slot`, `cursor_seq`), 3 slot descriptors (seq, state, up to 64 rects), 3 page-aligned pixel areas sized for the maximum resolution, and a 256×256 ARGB cursor area. Every shared field is atomic with acquire/release ordering; an unknown version makes the console refuse with `VTX-CON-010`.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/DisplaySourceBitmapImpl.cpp (raw VRAM pointer, in-process only)`

#### Scenario: Acceptance for FR-EN02-003
- **WHEN** the FR-EN02-003 acceptance check is run
- **THEN** The layout spec has C++ ↔ Rust binary-compatibility tests (same offsets and sizes); 24 h of header fuzzing crashes neither side.

### Requirement: Triple buffering: the writer never waits (FR-EN02-004)
Virtax SHALL provide "Triple buffering: the writer never waits" as specified by FR-EN02-004 (BRD-04, priority Must, phase P1; Vietnamese: "Triple buffer: writer không bao giờ chờ").

#### Scenario: Specified behaviour of FR-EN02-004
- **WHEN** the behaviour FR-EN02-004 covers is exercised
- **THEN** it behaves as BRD-04 specifies: The adapter writes into a slot that is neither the one the console is reading nor the one just published, copies the union of damage since that slot was last written (each slot tracks its own age), then publishes `write_seq`. The console marks `reader_slot` while reading; overtaken frames are skipped, with no tearing and no lock shared between the two processes.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIFrameBuffer.cpp:242-244,988-1017`

#### Scenario: Acceptance for FR-EN02-004
- **WHEN** the FR-EN02-004 acceptance check is run
- **THEN** Stalling the console on purpose for 2 s does not slow the VM display thread (`NotifyUpdate` p99 unchanged); a test pattern never shows tearing over 1 hour.

### Requirement: Damage coalescing ≤ 1 per vsync, plus notify (FR-EN02-005)
Virtax SHALL provide "Damage coalescing ≤ 1 per vsync, plus notify" as specified by FR-EN02-005 (BRD-04, priority Must, phase P1; Vietnamese: "Gộp damage ≤ 1 lần/vsync và notify").

#### Scenario: Specified behaviour of FR-EN02-005
- **WHEN** the behaviour FR-EN02-005 covers is exercised
- **THEN** it behaves as BRD-04 specifies: The adapter coalesces `NotifyUpdate` on the leading edge: it publishes immediately if nothing was published in the current frame interval and merges the rest until the interval ends (60 Hz default, 30 Hz in the Battery profile, following the console's hint); more than 64 rects or > 50% of the area falls back to a bounding box. It wakes the console via eventfd/pipe/Event with a {seq, slot, rects} message. With no console attached it makes 0 copies.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/DisplayImpl.cpp:912-918; UIMachineView.cpp:765-812`

#### Scenario: Acceptance for FR-EN02-005
- **WHEN** the FR-EN02-005 acceptance check is run
- **THEN** A guest issuing 1,000 updates/s yields ≤ 60 console notifies/s; a single update reaches the console ≤ 1 ms after `NotifyUpdate`; a headless VM with no console copies 0 bytes.

### Requirement: Resolution changes and SHM generations (FR-EN02-006)
Virtax SHALL provide "Resolution changes and SHM generations" as specified by FR-EN02-006 (BRD-04, priority Must, phase P1; Vietnamese: "Đổi độ phân giải và generation SHM").

#### Scenario: Specified behaviour of FR-EN02-006
- **WHEN** the behaviour FR-EN02-006 covers is exercised
- **THEN** it behaves as BRD-04 specifies: On `NotifyChange` the adapter re-queries the source bitmap and sends a RESIZE message; if slots are too small it allocates a new SHM (bumping `generation`) and sends the new handle on the same connection; the console remaps and keeps its window without reconnecting.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIFrameBuffer.cpp:620-673 (NotifyChange)`

#### Scenario: Acceptance for FR-EN02-006
- **WHEN** the FR-EN02-006 acceptance check is run
- **THEN** Guest switching 800×600 → 3840×2160 → 1920×1080 shows each mode correctly within ≤ 100 ms with no SHM leak (fd/handle counts equal before and after).

### Requirement: Cursor channel (FR-EN02-007)
Virtax SHALL provide "Cursor channel" as specified by FR-EN02-007 (BRD-04, priority Must, phase P1; Vietnamese: "Kênh cursor").

#### Scenario: Specified behaviour of FR-EN02-007
- **WHEN** the behaviour FR-EN02-007 covers is exercised
- **THEN** it behaves as BRD-04 specifies: On `MousePointerShapeChanged` the adapter converts the pointer shape (AND/XOR mask or ARGB, ≤ 256×256, hotspot) to ARGB in the SHM cursor area, bumps `cursor_seq` and sends a CURSOR message; position and visibility use the same channel.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIMachine.cpp:2045-2240`

#### Scenario: Acceptance for FR-EN02-007
- **WHEN** the FR-EN02-007 acceptance check is run
- **THEN** A set of 20 Windows/Linux cursors (including mono AND/XOR ones) renders with correct pixels and hotspots; shape changes trigger no frame upload.

### Requirement: Direct input channel bypassing ipcd (FR-EN02-008)
Virtax SHALL provide "Direct input channel bypassing ipcd" as specified by FR-EN02-008 (BRD-04, priority Must, phase P1; Vietnamese: "Kênh input trực tiếp, không qua ipcd").

#### Scenario: Specified behaviour of FR-EN02-008
- **WHEN** the behaviour FR-EN02-008 covers is exercised
- **THEN** it behaves as BRD-04 specifies: The console sends on the same socket: KEY {scancode set 1 or HID usage, down/up}, MOUSE_ABS {x, y, buttons, wheel}, MOUSE_REL {dx, dy, dz, dw, buttons}, TOUCH {contacts}, RELEASE_ALL. The adapter calls in-process `IKeyboard::putScancodes` / `putUsageCode` / `releaseKeys` and `IMouse::putMouseEvent` / `putMouseEventAbsolute` / `putEventMultiTouch`; ordering is preserved and batched within ≤ 1 ms. `Runtime.SendInput` over gRPC is only a fallback.
- **AND** the VirtualBox reference for the current behaviour is `src/libs/xpcom18a4/ipc/ipcd/client/src/ipcDConnectService.cpp:89`

#### Scenario: Acceptance for FR-EN02-008
- **WHEN** the FR-EN02-008 acceptance check is run
- **THEN** Key/mouse to the `IKeyboard`/`IMouse` call p95 ≤ 2 ms; 100,000 random events keep their order; 0 input calls go through VBoxSVC (trace).

### Requirement: Capability tokens and channel security (FR-EN02-009)
Virtax SHALL provide "Capability tokens and channel security" as specified by FR-EN02-009 (BRD-04, priority Must, phase P1; Vietnamese: "Capability token và bảo mật kênh").

#### Scenario: Specified behaviour of FR-EN02-009
- **WHEN** the behaviour FR-EN02-009 covers is exercised
- **THEN** it behaves as BRD-04 specifies: `virtaxd` creates a random per-VM key when launching the adapter; tokens are single-use HMACs that expire after 30 s, bound to (VM, screen, client uid/pid, mode CONTROL | VIEW | PREVIEW). VIEW and PREVIEW are read-only and cannot send input; tokens are revoked when the VM stops. The adapter's message parser is fuzzed.
- **AND** the VirtualBox reference for the current behaviour is `new (VirtualBox relies on COM session locks only)`

#### Scenario: Acceptance for FR-EN02-009
- **WHEN** the FR-EN02-009 acceptance check is run
- **THEN** Reused, expired or wrong-uid tokens are refused and audited; a PREVIEW client sending KEY gets disconnected; 24 h of libFuzzer on the parser shows no crash.

### Requirement: Display hints and guest resize (FR-EN02-010)
Virtax SHALL provide "Display hints and guest resize" as specified by FR-EN02-010 (BRD-04, priority Must, phase P1; Vietnamese: "Hint hiển thị và resize guest").

#### Scenario: Specified behaviour of FR-EN02-010
- **WHEN** the behaviour FR-EN02-010 covers is exercised
- **THEN** it behaves as BRD-04 specifies: The console sends HINT {width, height in physical pixels, scale, screen on/off}; the adapter calls `IDisplay::setVideoModeHint`, `notifyScaleFactorChange` and `notifyHiDPIOutputPolicyChange` with a 100 ms debounce (instead of a fixed 300 ms) and reports back when the guest cannot auto-resize so the console switches to GPU scaling.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/normal/UIMachineViewNormal.cpp:70`

#### Scenario: Acceptance for FR-EN02-010
- **WHEN** the FR-EN02-010 acceptance check is run
- **THEN** Resizing the window continuously for 5 s sends ≤ 10 hints, and the final resolution matches the window within ≤ 400 ms of releasing the mouse.

### Requirement: Multiple screens and visible regions (FR-EN02-011)
Virtax SHALL provide "Multiple screens and visible regions" as specified by FR-EN02-011 (BRD-04, priority Should, phase P1; Vietnamese: "Nhiều màn hình và visible region").

#### Scenario: Specified behaviour of FR-EN02-011
- **WHEN** the behaviour FR-EN02-011 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Each guest screen (up to 8 in P1) gets its own ring and damage, opened with `OpenDisplay(screen)`; `SetVisibleRegion` for seamless becomes a REGION message (≤ 256 rects, coalesced per frame).
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/runtime/UIFrameBuffer.cpp:826-895 (SetVisibleRegion)`

#### Scenario: Acceptance for FR-EN02-011
- **WHEN** the FR-EN02-011 acceptance check is run
- **THEN** A 4-screen VM: each ring only gets its own screen's damage; seamless regions update within 1 frame of a guest change.

### Requirement: Adaptive refresh (Engine Pack patch) (FR-EN02-012)
Virtax SHALL provide "Adaptive refresh (Engine Pack patch)" as specified by FR-EN02-012 (BRD-04, priority Should, phase P1; Vietnamese: "Nhịp refresh thích ứng (patch Engine Pack)").

#### Scenario: Specified behaviour of FR-EN02-012
- **WHEN** the behaviour FR-EN02-012 covers is exercised
- **THEN** it behaves as BRD-04 specifies: A small Display patch in the Engine Pack: with no console attached or every console reporting hidden, slow the 20 ms refresh timer to 200 ms (or VBVA-driven only); a console becoming visible restores 20 ms immediately. The patch is submitted upstream.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/DisplayImpl.cpp:4056 (pfnSetRefreshRate 20 ms)`

#### Scenario: Acceptance for FR-EN02-012
- **WHEN** the FR-EN02-012 acceptance check is run
- **THEN** An idle headless Linux-guest VM wakes ≥ 40 times/s less than stock VirtualBox; opening a console shows a fresh frame within 1 frame.

### Requirement: Fallback frame source for BYO VirtualBox (FR-EN02-013)
Virtax SHALL provide "Fallback frame source for BYO VirtualBox" as specified by FR-EN02-013 (BRD-04, priority Should, phase P1; Vietnamese: "Nguồn frame dự phòng cho BYO VirtualBox").

#### Scenario: Specified behaviour of FR-EN02-013
- **WHEN** the behaviour FR-EN02-013 covers is exercised
- **THEN** it behaves as BRD-04 specifies: Official VirtualBox (Alpha, BYO) cannot load the adapter because of hardening; `vbox-adapter` then attaches a remote `IFramebuffer` with `UpdateImage` and `virtaxd` copies rects into the same SHM ring, so the Console code is unchanged. The Console shows a "Compatibility display mode" chip suggesting the Engine Pack. The VNC extension pack is the second fallback.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Main/src-client/DisplayImpl.cpp:985-1006; src/VBox/ExtPacks/VNC`

#### Scenario: Acceptance for FR-EN02-013
- **WHEN** the FR-EN02-013 acceptance check is run
- **THEN** On official VirtualBox 7.2.x: 1080p at ≥ 30 fps and ≤ 15% CPU; switching to the Engine Pack needs no console configuration change.

### Requirement: Frame statistics and diagnostics (FR-EN02-014)
Virtax SHALL provide "Frame statistics and diagnostics" as specified by FR-EN02-014 (BRD-04, priority Could, phase P1; Vietnamese: "Số đo frame và chẩn đoán").

#### Scenario: Specified behaviour of FR-EN02-014
- **WHEN** the behaviour FR-EN02-014 covers is exercised
- **THEN** it behaves as BRD-04 specifies: The adapter and console count fps, dropped frames, copied bytes, upload time and damage → present latency; shown in a console overlay (e.g. "60 fps · 0.9 ms upload · 2.1% CPU") and via `Metrics.Watch`; included in the `virtaxctl diag` bundle.
- **AND** the VirtualBox reference for the current behaviour is `new (VirtualBox exposes only STAM counters via the debugger)`

#### Scenario: Acceptance for FR-EN02-014
- **WHEN** the FR-EN02-014 acceptance check is run
- **THEN** Host+I toggles the overlay; values are within 5% of an external profiler; with the overlay off, counters add no wakeups.

### Requirement: Phase 2: zero-copy GPU handles (FR-EN02-015)
Virtax SHALL provide "Phase 2: zero-copy GPU handles" as specified by FR-EN02-015 (BRD-04, priority Must, phase P2; Vietnamese: "Pha 2: GPU handle zero-copy").

#### Scenario: Specified behaviour of FR-EN02-015
- **WHEN** the behaviour FR-EN02-015 covers is exercised
- **THEN** it behaves as BRD-04 specifies: `DisplayChannel` gains a GpuSurface variant: dmabuf fd + modifier (Linux), IOSurface (macOS), DXGI NT shared handle + fence (Windows). The console imports it as an external texture through wgpu-hal with no CPU copy; if import fails it falls back to the SHM ring automatically.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Devices/Graphics/DevVGA-SVGA3d-dx-dx11-output.cpp:145-199`

#### Scenario: Acceptance for FR-EN02-015
- **WHEN** the FR-EN02-015 acceptance check is run
- **THEN** A 1440p60 3D guest on virtax-vmm: 0 CPU bytes copied per frame and console CPU ≤ 2%; a forced import failure continues on the SHM ring with no lost picture.

### Requirement: Phase 2: virtax-vmm on the same protocol (FR-EN02-016)
Virtax SHALL provide "Phase 2: virtax-vmm on the same protocol" as specified by FR-EN02-016 (BRD-04, priority Must, phase P2; Vietnamese: "Pha 2: virtax-vmm dùng cùng giao thức").

#### Scenario: Specified behaviour of FR-EN02-016
- **WHEN** the behaviour FR-EN02-016 covers is exercised
- **THEN** it behaves as BRD-04 specifies: virtax-vmm turns virtio-gpu `RESOURCE_FLUSH` into damage and writes the same v1 ring (or GpuSurface), using the same token and input channel (virtio-input/USB tablet); the Console and Manager preview never need to know the engine.
- **AND** the VirtualBox reference for the current behaviour is `new (OpenVMM ships only a VNC console)`

#### Scenario: Acceptance for FR-EN02-016
- **WHEN** the FR-EN02-016 acceptance check is run
- **THEN** The same console test suite (latency, CPU, capture, resize) passes on both LegacyEngine and VirtaxVmmEngine without console code changes.
