# Graphics & audio Specification

## Purpose
virtio-gpu 2D first, Venus/virgl (Linux), MoltenVK (macOS); virtio-snd + HDA for Windows guests. Module `EN-07` (Graphics & audio; Vietnamese: "Graphics & audio") from BRD-10, phase P2, priority Should. Owner crates: virtax-vmm-gpu, virtax-vmm-snd. Depends on: `engine/device-model-and-firmware`, `engine/display-and-input-data-plane`. VirtualBox reference: `src/VBox/Devices/Graphics (VMSVGA, 3D 52k), Devices/Audio`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: virtio-gpu 2D as the baseline (FR-EN07-001)
Virtax SHALL provide "virtio-gpu 2D as the baseline" as specified by FR-EN07-001 (BRD-10, priority Must, phase P2; Vietnamese: "virtio-gpu 2D là nền tảng").

#### Scenario: Specified behaviour of FR-EN07-001
- **WHEN** the behaviour FR-EN07-001 covers is exercised
- **THEN** it behaves as BRD-10 specifies: virtio-gpu 2D: up to 4 scanouts, EDID + hotplug for resolution changes, a separate cursor plane; damage comes from `RESOURCE_FLUSH` into the `virtax-frame` ring (EN-02), with no dirty-page tracking and no 50 Hz refresh timer.
- **AND** the VirtualBox reference for the current behaviour is `DisplayImpl.cpp:4056; DevVGA-SVGA.cpp:6000-6065`

#### Scenario: Acceptance for FR-EN07-001
- **WHEN** the FR-EN07-001 acceptance check is run
- **THEN** Static desktop → 0 frames, 0 wakeups; resizing the Console → guest resolution follows within 500 ms; 1080p60 ≤ 5% host CPU for scanout.

### Requirement: ramfb/GOP for firmware and early boot (FR-EN07-002)
Virtax SHALL provide "ramfb/GOP for firmware and early boot" as specified by FR-EN07-002 (BRD-10, priority Must, phase P2; Vietnamese: "ramfb/GOP cho firmware và boot sớm").

#### Scenario: Specified behaviour of FR-EN07-002
- **WHEN** the behaviour FR-EN07-002 covers is exercised
- **THEN** it behaves as BRD-10 specifies: GOP through OVMF's `VirtioGpuDxe`; ramfb on arm64 and as a fallback; a smooth handover from firmware to the guest virtio-gpu driver.
- **AND** the VirtualBox reference for the current behaviour is `PC/DevQemuFwCfg.cpp:188-206; OvmfPkgX64.fdf:388-392`

#### Scenario: Acceptance for FR-EN07-002
- **WHEN** the FR-EN07-002 acceptance check is run
- **THEN** The boot menu shows on all 6 combos; any black screen during handover lasts ≤ 1 s.

### Requirement: Venus/virgl on Linux hosts (FR-EN07-003)
Virtax SHALL provide "Venus/virgl on Linux hosts" as specified by FR-EN07-003 (BRD-10, priority Should, phase P3; Vietnamese: "Venus/virgl trên host Linux").

#### Scenario: Specified behaviour of FR-EN07-003
- **WHEN** the behaviour FR-EN07-003 covers is exercised
- **THEN** it behaves as BRD-10 specifies: 3D for Linux guests through virglrenderer ≥ 1.0 (Venus/Vulkan, virgl/OpenGL), blob resources, optional DRM native context; the render server runs in a separate sandboxed process; scanout is exported as dmabuf to the Console (EN-02).
- **AND** the VirtualBox reference for the current behaviour is `Graphics/DevVGA-SVGA3d*.cpp; Config.kmk:666-697`

#### Scenario: Acceptance for FR-EN07-003
- **WHEN** the FR-EN07-003 acceptance check is run
- **THEN** An Ubuntu guest runs vkcube and glmark2 at ≥ 50% of native on the same GPU; a render-server crash leaves the VM running in 2D.

### Requirement: Venus + MoltenVK on macOS (FR-EN07-004)
Virtax SHALL provide "Venus + MoltenVK on macOS" as specified by FR-EN07-004 (BRD-10, priority Should, phase P3; Vietnamese: "Venus + MoltenVK trên macOS").

#### Scenario: Specified behaviour of FR-EN07-004
- **WHEN** the behaviour FR-EN07-004 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Linux guests use Venus and the macOS host translates Vulkan → Metal with MoltenVK (as libkrun/krunkit do); scanout goes through an IOSurface with no CPU copy.
- **AND** the VirtualBox reference for the current behaviour is `DevVGA-SVGA3d-dx-dx11.cpp:3975-4060; Config.kmk:683-690`

#### Scenario: Acceptance for FR-EN07-004
- **WHEN** the FR-EN07-004 acceptance check is run
- **THEN** M-series Mac on macOS 15: GNOME in the guest stays ≥ 55 fps while dragging windows; 0 CPU copies per frame.

### Requirement: gfxstream as an experimental option (FR-EN07-005)
Virtax SHALL provide "gfxstream as an experimental option" as specified by FR-EN07-005 (BRD-10, priority Could, phase P3; Vietnamese: "gfxstream là tuỳ chọn thử nghiệm").

#### Scenario: Specified behaviour of FR-EN07-005
- **WHEN** the behaviour FR-EN07-005 covers is exercised
- **THEN** it behaves as BRD-10 specifies: On Windows hosts (no mature virglrenderer), a gfxstream backend via rutabaga_gfx serves Linux guests behind an experimental flag; the capability matrix states each host's support.
- **AND** the VirtualBox reference for the current behaviour is `new (analysis 06 §6)`

#### Scenario: Acceptance for FR-EN07-005
- **WHEN** the FR-EN07-005 acceptance check is run
- **THEN** Flag on → vkcube runs in a Linux guest on Windows 11; flag off → back to 2D with no reconfiguration.

### Requirement: Policy for the Windows-guest 3D gap (FR-EN07-006)
Virtax SHALL provide "Policy for the Windows-guest 3D gap" as specified by FR-EN07-006 (BRD-10, priority Must, phase P2; Vietnamese: "Chính sách cho khoảng trống 3D của guest Windows").

#### Scenario: Specified behaviour of FR-EN07-006
- **WHEN** the behaviour FR-EN07-006 covers is exercised
- **THEN** it behaves as BRD-10 specifies: `virtax-vmm` offers 2D only for Windows guests (viogpudo). "Move to virtax-vmm" blocks VMs using VMSVGA 3D; they stay on the legacy engine (VMSVGA + DXVK/DXMT), still installable in Phase 3. VFIO GPU passthrough is the Phase-3 path (CP-08).
- **AND** the VirtualBox reference for the current behaviour is `DevVGA-SVGA3d-dx-dx11.cpp; Devices/Makefile.kmk:351-401`

#### Scenario: Acceptance for FR-EN07-006
- **WHEN** the FR-EN07-006 acceptance check is run
- **THEN** A Windows VM with 3D → report "3D requires the legacy engine", Move locked unless "accept 2D" is chosen; in P3 the legacy engine remains selectable.

### Requirement: virtio-snd for Linux guests (FR-EN07-007)
Virtax SHALL provide "virtio-snd for Linux guests" as specified by FR-EN07-007 (BRD-10, priority Should, phase P2; Vietnamese: "virtio-snd cho guest Linux").

#### Scenario: Specified behaviour of FR-EN07-007
- **WHEN** the behaviour FR-EN07-007 covers is exercised
- **THEN** it behaves as BRD-10 specifies: virtio-snd with playback and capture, 44.1/48 kHz PCM, guest-chosen periods, event-driven. The microphone is off by default, enabled per VM and gated by the OS permission prompt.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Audio/DevIchAc97.cpp; Main/src-all/Global.cpp:266`

#### Scenario: Acceptance for FR-EN07-007
- **WHEN** the FR-EN07-007 acceptance check is run
- **THEN** Ubuntu guest audio latency ≤ 40 ms; no stream → 0 wakeups; the mic works only after enabling and OS consent.

### Requirement: HDA for Windows guests (FR-EN07-008)
Virtax SHALL provide "HDA for Windows guests" as specified by FR-EN07-008 (BRD-10, priority Should, phase P2; Vietnamese: "HDA cho guest Windows").

#### Scenario: Specified behaviour of FR-EN07-008
- **WHEN** the behaviour FR-EN07-008 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Intel HDA + codec so Windows uses in-box `hdaudio.sys`; DMA timers run on the audio thread, not a vCPU; 10-20 ms buffers; headless VMs have no audio by default.
- **AND** the VirtualBox reference for the current behaviour is `Devices/Audio/DevHda.cpp:5162-5166; DevHdaStream.cpp:780,878`

#### Scenario: Acceptance for FR-EN07-008
- **WHEN** the FR-EN07-008 acceptance check is run
- **THEN** Windows 11 plays audio with the in-box driver; 10 minutes at 50% host load: 0 underruns; idle HDA → 0 wakeups/s.

### Requirement: Host audio backends (FR-EN07-009)
Virtax SHALL provide "Host audio backends" as specified by FR-EN07-009 (BRD-10, priority Should, phase P2; Vietnamese: "Backend audio host").

#### Scenario: Specified behaviour of FR-EN07-009
- **WHEN** the behaviour FR-EN07-009 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Event-driven WASAPI shared mode (Windows), CoreAudio AudioUnit (macOS), native PipeWire or PulseAudio with ALSA fallback (Linux); follows the default device when it changes; one resample; at most 2 copies guest → host.
- **AND** the VirtualBox reference for the current behaviour is `Audio/DrvHostAudio*.cpp; AudioMixer.cpp:1906`

#### Scenario: Acceptance for FR-EN07-009
- **WHEN** the FR-EN07-009 acceptance check is run
- **THEN** Changing the default output → guest audio follows within 1 s; 48 kHz stereo uses ≤ 1% of a core.

### Requirement: Power policy for graphics and audio (FR-EN07-010)
Virtax SHALL provide "Power policy for graphics and audio" as specified by FR-EN07-010 (BRD-10, priority Should, phase P2; Vietnamese: "Chính sách năng lượng cho đồ hoạ và audio").

#### Scenario: Specified behaviour of FR-EN07-010
- **WHEN** the behaviour FR-EN07-010 covers is exercised
- **THEN** it behaves as BRD-10 specifies: Hidden/minimised Console → scanout stops copying frames (the guest still gets vblank); the Battery profile caps at 30 fps; background VMs can be muted to a null sink by preference.
- **AND** the VirtualBox reference for the current behaviour is `DisplayImpl.cpp:4056; DevVGA-SVGA.cpp:6000-6065`

#### Scenario: Acceptance for FR-EN07-010
- **WHEN** the FR-EN07-010 acceptance check is run
- **THEN** Minimised Console → display thread ≤ 1 wakeup/s; on battery → ≤ 30 fps measured on the Console.
