# Non-functional requirements Specification

## Purpose
Measurable quality targets (performance, reliability, security, accessibility, portability and more) from the NFR tables of BRD-00 to BRD-12. Each requirement cites its BRD; the CI performance gates of FR-PL01-007 measure them. Implementation status (2026-11-24): not yet measured in CI.

## Requirements

### Requirement: Manager shows a 50-VM list (NFR-00-01)
Virtax SHALL meet NFR-00-01 (BRD-00, Performance, phase P1): Manager shows a 50-VM list — cold ≤ 1.5 s · warm ≤ 0.4 s.

#### Scenario: Target for NFR-00-01
- **WHEN** Manager shows a 50-VM list is measured
- **THEN** the result meets: cold ≤ 1.5 s · warm ≤ 0.4 s
- **AND** the Vietnamese source reads "Manager hiện danh sách 50 VM"

### Requirement: Idle CPU (Manager open, no VM running) (NFR-00-02)
Virtax SHALL meet NFR-00-02 (BRD-00, Performance, phase P1): Idle CPU (Manager open, no VM running) — ≤ 0.5% · ≤ 5 wakeups/s.

#### Scenario: Target for NFR-00-02
- **WHEN** Idle CPU (Manager open, no VM running) is measured
- **THEN** the result meets: ≤ 0.5% · ≤ 5 wakeups/s
- **AND** the Vietnamese source reads "CPU khi idle (Manager mở, không VM chạy)"

### Requirement: Manager memory (NFR-00-03)
Virtax SHALL meet NFR-00-03 (BRD-00, Performance, phase P1): Manager memory — RSS ≤ 120 MB (50 VMs).

#### Scenario: Target for NFR-00-03
- **WHEN** Manager memory is measured
- **THEN** the result meets: RSS ≤ 120 MB (50 VMs)
- **AND** the Vietnamese source reads "Bộ nhớ Manager"

### Requirement: Added console latency (NFR-00-04)
Virtax SHALL meet NFR-00-04 (BRD-00, Performance, phase P1): Added console latency — p95 ≤ 16.7 ms @60 Hz.

#### Scenario: Target for NFR-00-04
- **WHEN** Added console latency is measured
- **THEN** the result meets: p95 ≤ 16.7 ms @60 Hz
- **AND** the Vietnamese source reads "Độ trễ console thêm vào"

### Requirement: Local control-plane API calls (NFR-00-05)
Virtax SHALL meet NFR-00-05 (BRD-00, Performance, phase P1): Local control-plane API calls — p95 ≤ 5 ms (read) · ≤ 50 ms (update).

#### Scenario: Target for NFR-00-05
- **WHEN** Local control-plane API calls is measured
- **THEN** the result meets: p95 ≤ 5 ms (read) · ≤ 50 ms (update)
- **AND** the Vietnamese source reads "Gọi API control plane cục bộ"

### Requirement: Crash-free sessions (NFR-00-06)
Virtax SHALL meet NFR-00-06 (BRD-00, Reliability, phase P1): Crash-free sessions — ≥ 99.5% (beta) · ≥ 99.9% (GA).

#### Scenario: Target for NFR-00-06
- **WHEN** Crash-free sessions is measured
- **THEN** the result meets: ≥ 99.5% (beta) · ≥ 99.9% (GA)
- **AND** the Vietnamese source reads "Phiên không crash"

### Requirement: VMs survive UI or daemon restarts (NFR-00-07)
Virtax SHALL meet NFR-00-07 (BRD-00, Reliability, phase P2): VMs survive UI or daemon restarts — 0 VM bị kill khi restart virtaxd.

#### Scenario: Target for NFR-00-07
- **WHEN** VMs survive UI or daemon restarts is measured
- **THEN** the result meets: 0 VM bị kill khi restart virtaxd
- **AND** the Vietnamese source reads "VM vẫn chạy khi UI hoặc daemon restart"

### Requirement: No permanently elevated process except a minimal helper (NFR-00-08)
Virtax SHALL meet NFR-00-08 (BRD-00, Security, phase P1): No permanently elevated process except a minimal helper — helper ≤ 3k LOC, audit trước GA.

#### Scenario: Target for NFR-00-08
- **WHEN** No permanently elevated process except a minimal helper is measured
- **THEN** the result meets: helper ≤ 3k LOC, audit trước GA
- **AND** the Vietnamese source reads "Không process nào chạy root/admin thường trực trừ helper tối thiểu"

### Requirement: VM creation completed unaided (NFR-00-09)
Virtax SHALL meet NFR-00-09 (BRD-00, Usability, phase P1): VM creation completed unaided — ≥ 80% test users · SUS ≥ 80.

#### Scenario: Target for NFR-00-09
- **WHEN** VM creation completed unaided is measured
- **THEN** the result meets: ≥ 80% test users · SUS ≥ 80
- **AND** the Vietnamese source reads "Hoàn thành tạo VM không trợ giúp"

### Requirement: WCAG 2.2 AA for the desktop UI (NFR-00-10)
Virtax SHALL meet NFR-00-10 (BRD-00, Accessibility, phase P1): WCAG 2.2 AA for the desktop UI — contrast ≥ 4.5:1 · 100% keyboard · screen reader labels.

#### Scenario: Target for NFR-00-10
- **WHEN** WCAG 2.2 AA for the desktop UI is measured
- **THEN** the result meets: contrast ≥ 4.5:1 · 100% keyboard · screen reader labels
- **AND** the Vietnamese source reads "Tuân thủ WCAG 2.2 AA cho UI desktop"

### Requirement: One codebase for 6 host combos (NFR-00-11)
Virtax SHALL meet NFR-00-11 (BRD-00, Portability, phase P1): One codebase for 6 host combos — ≤ 5% code platform-specific ngoài crate hv/platform.

#### Scenario: Target for NFR-00-11
- **WHEN** One codebase for 6 host combos is measured
- **THEN** the result meets: ≤ 5% code platform-specific ngoài crate hv/platform
- **AND** the Vietnamese source reads "Cùng codebase cho 6 tổ hợp host"

### Requirement: Rust code quality (NFR-00-12)
Virtax SHALL meet NFR-00-12 (BRD-00, Maintainability, phase P1): Rust code quality — clippy -D warnings · unsafe chỉ trong crate *-sys/hv · coverage ≥ 70% core.

#### Scenario: Target for NFR-00-12
- **WHEN** Rust code quality is measured
- **THEN** the result meets: clippy -D warnings · unsafe chỉ trong crate *-sys/hv · coverage ≥ 70% core
- **AND** the Vietnamese source reads "Chất lượng code Rust"

### Requirement: GPLv3 compatibility (NFR-00-13)
Virtax SHALL meet NFR-00-13 (BRD-00, Licensing, phase P1): GPLv3 compatibility — cargo-deny: 0 license vi phạm; SBOM mỗi release.

#### Scenario: Target for NFR-00-13
- **WHEN** GPLv3 compatibility is measured
- **THEN** the result meets: cargo-deny: 0 license vi phạm; SBOM mỗi release
- **AND** the Vietnamese source reads "Tương thích GPLv3"

### Requirement: Install & upgrade (NFR-00-14)
Virtax SHALL meet NFR-00-14 (BRD-00, Install, phase P2): Install & upgrade — ≤ 2 min, không reboot (Pha 2), không build kernel module.

#### Scenario: Target for NFR-00-14
- **WHEN** Install & upgrade is measured
- **THEN** the result meets: ≤ 2 min, không reboot (Pha 2), không build kernel module
- **AND** the Vietnamese source reads "Cài đặt và nâng cấp"

### Requirement: App launch to an interactive VM list (NFR-01-01)
Virtax SHALL meet NFR-01-01 (BRD-01, Performance, phase P1): App launch to an interactive VM list — 50 VMs: cold ≤ 1.5 s · warm ≤ 0.4 s; 500 VMs: cold ≤ 2.0 s.

#### Scenario: Target for NFR-01-01
- **WHEN** App launch to an interactive VM list is measured
- **THEN** the result meets: 50 VMs: cold ≤ 1.5 s · warm ≤ 0.4 s; 500 VMs: cold ≤ 2.0 s
- **AND** the Vietnamese source reads "Từ lúc mở app tới danh sách VM tương tác được"

### Requirement: List and search at large VM counts (NFR-01-02)
Virtax SHALL meet NFR-01-02 (BRD-01, Performance, phase P1): List and search at large VM counts — 2,000 VMs scroll p95 ≤ 16.7 ms · search ≤ 30 ms/keystroke @500.

#### Scenario: Target for NFR-01-02
- **WHEN** List and search at large VM counts is measured
- **THEN** the result meets: 2,000 VMs scroll p95 ≤ 16.7 ms · search ≤ 30 ms/keystroke @500
- **AND** the Vietnamese source reads "Danh sách và search với số lượng VM lớn"

### Requirement: Host resources with an idle Manager (no VM / 1 running VM with preview visible) (NFR-01-03)
Virtax SHALL meet NFR-01-03 (BRD-01, Performance, phase P1): Host resources with an idle Manager (no VM / 1 running VM with preview visible) — ≤ 0.5% CPU, ≤ 5 wakeups/s, 0 frames / ≤ 1.5% CPU.

#### Scenario: Target for NFR-01-03
- **WHEN** Host resources with an idle Manager (no VM / 1 running VM with preview visible) is measured
- **THEN** the result meets: ≤ 0.5% CPU, ≤ 5 wakeups/s, 0 frames / ≤ 1.5% CPU
- **AND** the Vietnamese source reads "Tài nguyên host khi Manager idle (không VM chạy / 1 VM chạy, preview hiện)"

### Requirement: UI thread never blocked; action feedback (NFR-01-04)
Virtax SHALL meet NFR-01-04 (BRD-01, Performance, phase P1): UI thread never blocked; action feedback — 0 tasks > 50 ms · feedback ≤ 100 ms · progress shown if > 100 ms.

#### Scenario: Target for NFR-01-04
- **WHEN** UI thread never blocked; action feedback is measured
- **THEN** the result meets: 0 tasks > 50 ms · feedback ≤ 100 ms · progress shown if > 100 ms
- **AND** the Vietnamese source reads "UI thread không bị chặn; phản hồi thao tác"

### Requirement: Manager memory (NFR-01-05)
Virtax SHALL meet NFR-01-05 (BRD-01, Performance, phase P1): Manager memory — RSS ≤ 120 MB @50 VMs · ≤ 160 MB @500 VMs.

#### Scenario: Target for NFR-01-05
- **WHEN** Manager memory is measured
- **THEN** the result meets: RSS ≤ 120 MB @50 VMs · ≤ 160 MB @500 VMs
- **AND** the Vietnamese source reads "Bộ nhớ Manager"

### Requirement: Event latency and completeness to the UI (NFR-01-06)
Virtax SHALL meet NFR-01-06 (BRD-01, Reliability, phase P1): Event latency and completeness to the UI — p95 ≤ 250 ms · 0 lost terminal events across reconnects.

#### Scenario: Target for NFR-01-06
- **WHEN** Event latency and completeness to the UI is measured
- **THEN** the result meets: p95 ≤ 250 ms · 0 lost terminal events across reconnects
- **AND** the Vietnamese source reads "Độ trễ và độ đầy đủ của event tới UI"

### Requirement: WCAG 2.2 AA, 100% keyboard, screen readers (NFR-01-07)
Virtax SHALL meet NFR-01-07 (BRD-01, Accessibility, phase P1): WCAG 2.2 AA, 100% keyboard, screen readers — 0 A/AA issues · NVDA/VoiceOver/Orca pass 10 flows.

#### Scenario: Target for NFR-01-07
- **WHEN** WCAG 2.2 AA, 100% keyboard, screen readers is measured
- **THEN** the result meets: 0 A/AA issues · NVDA/VoiceOver/Orca pass 10 flows
- **AND** the Vietnamese source reads "WCAG 2.2 AA, bàn phím 100%, screen reader"

### Requirement: Token contrast in all 3 themes (NFR-01-08)
Virtax SHALL meet NFR-01-08 (BRD-01, Accessibility, phase P1): Token contrast in all 3 themes — 100% text pairs ≥ 4.5:1 · UI glyphs ≥ 3:1.

#### Scenario: Target for NFR-01-08
- **WHEN** Token contrast in all 3 themes is measured
- **THEN** the result meets: 100% text pairs ≥ 4.5:1 · UI glyphs ≥ 3:1
- **AND** the Vietnamese source reads "Contrast của token ở cả 3 theme"

### Requirement: Complete VI/EN without truncation (NFR-01-09)
Virtax SHALL meet NFR-01-09 (BRD-01, i18n, phase P1): Complete VI/EN without truncation — 0 missing keys · pseudo-locale +35% → 0 truncations.

#### Scenario: Target for NFR-01-09
- **WHEN** Complete VI/EN without truncation is measured
- **THEN** the result meets: 0 missing keys · pseudo-locale +35% → 0 truncations
- **AND** the Vietnamese source reads "VI/EN đầy đủ, không cắt chữ"

### Requirement: Learnable and discoverable (NFR-01-10)
Virtax SHALL meet NFR-01-10 (BRD-01, Usability, phase P1): Learnable and discoverable — SUS ≥ 80 · 90% find a card's Edit in ≤ 10 s · ≤ 2 actions to any tab.

#### Scenario: Target for NFR-01-10
- **WHEN** Learnable and discoverable is measured
- **THEN** the result meets: SUS ≥ 80 · 90% find a card's Edit in ≤ 10 s · ≤ 2 actions to any tab
- **AND** the Vietnamese source reads "Dễ học và dễ tìm"

### Requirement: ISO detection time (NFR-02-01)
Virtax SHALL meet NFR-02-01 (BRD-02, Performance, phase P1): ISO detection time — p95 ≤ 2 s (SSD) · ≤ 8 s (USB HDD) · cache ≤ 50 ms.

#### Scenario: Target for NFR-02-01
- **WHEN** ISO detection time is measured
- **THEN** the result meets: p95 ≤ 2 s (SSD) · ≤ 8 s (USB HDD) · cache ≤ 50 ms
- **AND** the Vietnamese source reads "Thời gian detect ISO"

### Requirement: UI never blocked during detect/create/import/export (NFR-02-02)
Virtax SHALL meet NFR-02-02 (BRD-02, Performance, phase P1): UI never blocked during detect/create/import/export — UI thread stall ≤ 50 ms p99 · Quick Create mở ≤ 150 ms.

#### Scenario: Target for NFR-02-02
- **WHEN** UI never blocked during detect/create/import/export is measured
- **THEN** the result meets: UI thread stall ≤ 50 ms p99 · Quick Create mở ≤ 150 ms
- **AND** the Vietnamese source reads "UI không bị khoá trong detect/create/import/export"

### Requirement: VM creation (excluding installer) until ready to boot (NFR-02-03)
Virtax SHALL meet NFR-02-03 (BRD-02, Performance, phase P1): VM creation (excluding installer) until ready to boot — ≤ 3 s trên NVMe SSD, disk dynamic.

#### Scenario: Target for NFR-02-03
- **WHEN** VM creation (excluding installer) until ready to boot is measured
- **THEN** the result meets: ≤ 3 s trên NVMe SSD, disk dynamic
- **AND** the Vietnamese source reads "Tạo VM (không tính installer) tới trạng thái sẵn sàng boot"

### Requirement: Unattended success rate on the reference matrix (NFR-02-04)
Virtax SHALL meet NFR-02-04 (BRD-02, Reliability, phase P1): Unattended success rate on the reference matrix — ≥ 95% · 12 ISO × 6 host, nightly.

#### Scenario: Target for NFR-02-04
- **WHEN** Unattended success rate on the reference matrix is measured
- **THEN** the result meets: ≥ 95% · 12 ISO × 6 host, nightly
- **AND** the Vietnamese source reads "Tỷ lệ cài unattended thành công trên ma trận tham chiếu"

### Requirement: Cancel/rollback leaves no residue (NFR-02-05)
Virtax SHALL meet NFR-02-05 (BRD-02, Reliability, phase P1): Cancel/rollback leaves no residue — 0 file/dòng registry mồ côi sau 100 lần huỷ ngẫu nhiên.

#### Scenario: Target for NFR-02-05
- **WHEN** Cancel/rollback leaves no residue is measured
- **THEN** the result meets: 0 file/dòng registry mồ côi sau 100 lần huỷ ngẫu nhiên
- **AND** the Vietnamese source reads "Huỷ/rollback không để lại rác"

### Requirement: No plaintext secrets at rest after install (NFR-02-06)
Virtax SHALL meet NFR-02-06 (BRD-02, Security, phase P1): No plaintext secrets at rest after install — 0 hit khi quét · media phụ xoá ≤ 60 s.

#### Scenario: Target for NFR-02-06
- **WHEN** No plaintext secrets at rest after install is measured
- **THEN** the result meets: 0 hit khi quét · media phụ xoá ≤ 60 s
- **AND** the Vietnamese source reads "Không có bí mật plaintext lưu trên đĩa sau khi cài"

### Requirement: Import/export/convert throughput and I/O priority (NFR-02-07)
Virtax SHALL meet NFR-02-07 (BRD-02, Performance, phase P1): Import/export/convert throughput and I/O priority — ≥ 80% tốc độ tuần tự · VM khác p95 latency +≤ 20%.

#### Scenario: Target for NFR-02-07
- **WHEN** Import/export/convert throughput and I/O priority is measured
- **THEN** the result meets: ≥ 80% tốc độ tuần tự · VM khác p95 latency +≤ 20%
- **AND** the Vietnamese source reads "Thông lượng và độ ưu tiên I/O của import/export/convert"

### Requirement: Reading `.vbox` and importing OVAs from many sources (NFR-02-08)
Virtax SHALL meet NFR-02-08 (BRD-02, Compatibility, phase P1): Reading `.vbox` and importing OVAs from many sources — .vbox 1.3-1.22: 100% parse, 0 diff · OVA ≥ 95%.

#### Scenario: Target for NFR-02-08
- **WHEN** Reading `.vbox` and importing OVAs from many sources is measured
- **THEN** the result meets: .vbox 1.3-1.22: 100% parse, 0 diff · OVA ≥ 95%
- **AND** the Vietnamese source reads "Đọc `.vbox` và import OVA từ nhiều nguồn"

### Requirement: Quick Create and side sheets meet WCAG 2.2 AA (NFR-02-09)
Virtax SHALL meet NFR-02-09 (BRD-02, Accessibility, phase P1): Quick Create and side sheets meet WCAG 2.2 AA — 100% bàn phím · contrast ≥ 4.5:1 · target ≥ 32 px.

#### Scenario: Target for NFR-02-09
- **WHEN** Quick Create and side sheets meet WCAG 2.2 AA is measured
- **THEN** the result meets: 100% bàn phím · contrast ≥ 4.5:1 · target ≥ 32 px
- **AND** the Vietnamese source reads "Quick Create và các side sheet đạt WCAG 2.2 AA"

### Requirement: First-time users create a VM unaided (NFR-02-10)
Virtax SHALL meet NFR-02-10 (BRD-02, Usability, phase P1): First-time users create a VM unaided — ≥ 80% hoàn thành · SUS ≥ 80 · ≤ 3 thao tác bắt buộc.

#### Scenario: Target for NFR-02-10
- **WHEN** First-time users create a VM unaided is measured
- **THEN** the result meets: ≥ 80% hoàn thành · SUS ≥ 80 · ≤ 3 thao tác bắt buộc
- **AND** the Vietnamese source reads "Người dùng mới tự tạo được VM"

### Requirement: Sheet open, search and validation (NFR-03-01)
Virtax SHALL meet NFR-03-01 (BRD-03, Performance, phase P1): Sheet open, search and validation — open ≤ 150 ms · search ≤ 30 ms · validate ≤ 100 ms.

#### Scenario: Target for NFR-03-01
- **WHEN** Sheet open, search and validation is measured
- **THEN** the result meets: open ≤ 150 ms · search ≤ 30 ms · validate ≤ 100 ms
- **AND** the Vietnamese source reads "Mở sheet, search và validate"

### Requirement: Local configuration API (NFR-03-02)
Virtax SHALL meet NFR-03-02 (BRD-03, Performance, phase P1): Local configuration API — GetMachine p95 ≤ 5 ms · UpdateMachine p95 ≤ 50 ms · dry run ≤ 20 ms.

#### Scenario: Target for NFR-03-02
- **WHEN** Local configuration API is measured
- **THEN** the result meets: GetMachine p95 ≤ 5 ms · UpdateMachine p95 ≤ 50 ms · dry run ≤ 20 ms
- **AND** the Vietnamese source reads "API cấu hình cục bộ"

### Requirement: Registry with many VMs (NFR-03-03)
Virtax SHALL meet NFR-03-03 (BRD-03, Scalability, phase P1): Registry with many VMs — 2,000 VMs: ListMachines ≤ 30 ms · DB ≤ 50 MB · reconcile ≤ 5 s.

#### Scenario: Target for NFR-03-03
- **WHEN** Registry with many VMs is measured
- **THEN** the result meets: 2,000 VMs: ListMachines ≤ 30 ms · DB ≤ 50 MB · reconcile ≤ 5 s
- **AND** the Vietnamese source reads "Registry với nhiều VM"

### Requirement: Crash and power-loss consistency (NFR-03-04)
Virtax SHALL meet NFR-03-04 (BRD-03, Reliability, phase P1): Crash and power-loss consistency — 0 corrupt files in 10,000 injections · PRAGMA integrity_check = ok.

#### Scenario: Target for NFR-03-04
- **WHEN** Crash and power-loss consistency is measured
- **THEN** the result meets: 0 corrupt files in 10,000 injections · PRAGMA integrity_check = ok
- **AND** the Vietnamese source reads "Nhất quán khi crash hoặc mất điện"

### Requirement: `.vbox` and VirtualBox GUI compatibility (NFR-03-05)
Virtax SHALL meet NFR-03-05 (BRD-03, Compatibility, phase P1): `.vbox` and VirtualBox GUI compatibility — 100% corpus 1.3-1.22 · 0 semantic diffs · 0 new GUI/* keys.

#### Scenario: Target for NFR-03-05
- **WHEN** `.vbox` and VirtualBox GUI compatibility is measured
- **THEN** the result meets: 100% corpus 1.3-1.22 · 0 semantic diffs · 0 new GUI/* keys
- **AND** the Vietnamese source reads "Tương thích `.vbox` và VirtualBox GUI"

### Requirement: Host OS: no I/O or wakeups when nothing changes (NFR-03-06)
Virtax SHALL meet NFR-03-06 (BRD-03, Performance, phase P1): Host OS: no I/O or wakeups when nothing changes — 0 writes/min idle · ui_prefs ≤ 1 write/s · 0 fsync on UI thread.

#### Scenario: Target for NFR-03-06
- **WHEN** Host OS: no I/O or wakeups when nothing changes is measured
- **THEN** the result meets: 0 writes/min idle · ui_prefs ≤ 1 write/s · 0 fsync on UI thread
- **AND** the Vietnamese source reads "Host OS: không I/O hay wakeup khi không có thay đổi"

### Requirement: File permissions and secrets (NFR-03-07)
Virtax SHALL meet NFR-03-07 (BRD-03, Security, phase P1): File permissions and secrets — 0600/0700 or user-only ACL · 0 secrets in TOML/SQLite (CI scan).

#### Scenario: Target for NFR-03-07
- **WHEN** File permissions and secrets is measured
- **THEN** the result meets: 0600/0700 or user-only ACL · 0 secrets in TOML/SQLite (CI scan)
- **AND** the Vietnamese source reads "Quyền file và bí mật"

### Requirement: Understanding when changes apply (NFR-03-08)
Virtax SHALL meet NFR-03-08 (BRD-03, Usability, phase P1): Understanding when changes apply — ≥ 90% correct predictions · any field found ≤ 10 s.

#### Scenario: Target for NFR-03-08
- **WHEN** Understanding when changes apply is measured
- **THEN** the result meets: ≥ 90% correct predictions · any field found ≤ 10 s
- **AND** the Vietnamese source reads "Hiểu thời điểm có hiệu lực"

### Requirement: Accessible forms (NFR-03-09)
Virtax SHALL meet NFR-03-09 (BRD-03, Accessibility, phase P1): Accessible forms — 100% labelled fields · errors announced · 5 tasks keyboard-only.

#### Scenario: Target for NFR-03-09
- **WHEN** Accessible forms is measured
- **THEN** the result meets: 100% labelled fields · errors announced · 5 tasks keyboard-only
- **AND** the Vietnamese source reads "Form truy cập được"

### Requirement: Identical formats on 6 host combos (NFR-03-10)
Virtax SHALL meet NFR-03-10 (BRD-03, Portability, phase P1): Identical formats on 6 host combos — relative paths · case-insensitive FS safe · LF/CRLF tolerant.

#### Scenario: Target for NFR-03-10
- **WHEN** Identical formats on 6 host combos is measured
- **THEN** the result meets: relative paths · case-insensitive FS safe · LF/CRLF tolerant
- **AND** the Vietnamese source reads "Định dạng giống nhau trên 6 tổ hợp host"

### Requirement: Added guest-damage-to-photon latency (NFR-04-01)
Virtax SHALL meet NFR-04-01 (BRD-04, Performance, phase P1): Added guest-damage-to-photon latency — p95 ≤ 16.7 ms @60 Hz (FIFO) · ≤ 10 ms (Mailbox).

#### Scenario: Target for NFR-04-01
- **WHEN** Added guest-damage-to-photon latency is measured
- **THEN** the result meets: p95 ≤ 16.7 ms @60 Hz (FIFO) · ≤ 10 ms (Mailbox)
- **AND** the Vietnamese source reads "Độ trễ thêm từ damage guest tới photon"

### Requirement: Host CPU for console + adapter (NFR-04-02)
Virtax SHALL meet NFR-04-02 (BRD-04, Performance, phase P1): Host CPU for console + adapter — 1080p60 ≤ 5% · 4K60 ≤ 10% (4-core reference laptop).

#### Scenario: Target for NFR-04-02
- **WHEN** Host CPU for console + adapter is measured
- **THEN** the result meets: 1080p60 ≤ 5% · 4K60 ≤ 10% (4-core reference laptop)
- **AND** the Vietnamese source reads "CPU host cho console + adapter"

### Requirement: Host OS cost while the guest screen is static (NFR-04-03)
Virtax SHALL meet NFR-04-03 (BRD-04, Performance, phase P1): Host OS cost while the guest screen is static — console 0% CPU · ≤ 1 wakeup/s · 0 bytes copied.

#### Scenario: Target for NFR-04-03
- **WHEN** Host OS cost while the guest screen is static is measured
- **THEN** the result meets: console 0% CPU · ≤ 1 wakeup/s · 0 bytes copied
- **AND** the Vietnamese source reads "Host OS khi màn hình guest đứng yên"

### Requirement: Input-to-engine latency (NFR-04-04)
Virtax SHALL meet NFR-04-04 (BRD-04, Performance, phase P1): Input-to-engine latency — p95 ≤ 2 ms · 100% ordered.

#### Scenario: Target for NFR-04-04
- **WHEN** Input-to-engine latency is measured
- **THEN** the result meets: p95 ≤ 2 ms · 100% ordered
- **AND** the Vietnamese source reads "Độ trễ input tới engine"

### Requirement: Memory (NFR-04-05)
Virtax SHALL meet NFR-04-05 (BRD-04, Performance, phase P1): Memory — SHM ≤ 3 × max frame + 1 MiB per screen · console RSS ≤ 80 MB excl. SHM.

#### Scenario: Target for NFR-04-05
- **WHEN** Memory is measured
- **THEN** the result meets: SHM ≤ 3 × max frame + 1 MiB per screen · console RSS ≤ 80 MB excl. SHM
- **AND** the Vietnamese source reads "Bộ nhớ"

### Requirement: Mode switches and resize (NFR-04-06)
Virtax SHALL meet NFR-04-06 (BRD-04, Usability, phase P1): Mode switches and resize — ≤ 200 ms · same native window · 0 black frames.

#### Scenario: Target for NFR-04-06
- **WHEN** Mode switches and resize is measured
- **THEN** the result meets: ≤ 200 ms · same native window · 0 black frames
- **AND** the Vietnamese source reads "Đổi mode và resize"

### Requirement: Console isolation from the VM (NFR-04-07)
Virtax SHALL meet NFR-04-07 (BRD-04, Reliability, phase P1): Console isolation from the VM — console crash → 0 VM impact · reconnect ≤ 1 s · 0 stuck keys / 1,000 cycles.

#### Scenario: Target for NFR-04-07
- **WHEN** Console isolation from the VM is measured
- **THEN** the result meets: console crash → 0 VM impact · reconnect ≤ 1 s · 0 stuck keys / 1,000 cycles
- **AND** the Vietnamese source reads "Console tách biệt khỏi VM"

### Requirement: Data-plane channel (NFR-04-08)
Virtax SHALL meet NFR-04-08 (BRD-04, Security, phase P1): Data-plane channel — single-use token TTL 30 s · user-only endpoint · 24 h fuzz clean.

#### Scenario: Target for NFR-04-08
- **WHEN** Data-plane channel is measured
- **THEN** the result meets: single-use token TTL 30 s · user-only endpoint · 24 h fuzz clean
- **AND** the Vietnamese source reads "Kênh data plane"

### Requirement: Accessible console chrome (NFR-04-09)
Virtax SHALL meet NFR-04-09 (BRD-04, Accessibility, phase P1): Accessible console chrome — 100% keyboard · capture state announced · labelled chips.

#### Scenario: Target for NFR-04-09
- **WHEN** Accessible console chrome is measured
- **THEN** the result meets: 100% keyboard · capture state announced · labelled chips
- **AND** the Vietnamese source reads "Chrome console truy cập được"

### Requirement: Capture and display matrix (NFR-04-10)
Virtax SHALL meet NFR-04-10 (BRD-04, Portability, phase P1): Capture and display matrix — Win 10/11 · macOS 14/15 · X11 · GNOME 46+ / KDE 6 Wayland · x64 + arm64.

#### Scenario: Target for NFR-04-10
- **WHEN** Capture and display matrix is measured
- **THEN** the result meets: Win 10/11 · macOS 14/15 · X11 · GNOME 46+ / KDE 6 Wayland · x64 + arm64
- **AND** the Vietnamese source reads "Ma trận capture và hiển thị"

### Requirement: Lifecycle command latency and state updates (NFR-05-01)
Virtax SHALL meet NFR-05-01 (BRD-05, Performance, phase P1): Lifecycle command latency and state updates — ack ≤ 100 ms p95 · event → UI ≤ 200 ms p95.

#### Scenario: Target for NFR-05-01
- **WHEN** Lifecycle command latency and state updates is measured
- **THEN** the result meets: ack ≤ 100 ms p95 · event → UI ≤ 200 ms p95
- **AND** the Vietnamese source reads "Độ trễ lệnh vòng đời và cập nhật trạng thái"

### Requirement: VM start (click → first frame, VBoxSVC resident) (NFR-05-02)
Virtax SHALL meet NFR-05-02 (BRD-05, Performance, phase P1): VM start (click → first frame, VBoxSVC resident) — p50 ≤ 3 s Linux/macOS · ≤ 4 s Windows.

#### Scenario: Target for NFR-05-02
- **WHEN** VM start (click → first frame, VBoxSVC resident) is measured
- **THEN** the result meets: p50 ≤ 3 s Linux/macOS · ≤ 4 s Windows
- **AND** the Vietnamese source reads "Start VM (click → frame đầu, VBoxSVC đã thường trú)"

### Requirement: Supervisor/daemon wakeups with 5 running VMs (NFR-05-03)
Virtax SHALL meet NFR-05-03 (BRD-05, Performance, phase P1): Supervisor/daemon wakeups with 5 running VMs — ≤ 20 wakeup/s tổng · 0 timer polling.

#### Scenario: Target for NFR-05-03
- **WHEN** Supervisor/daemon wakeups with 5 running VMs is measured
- **THEN** the result meets: ≤ 20 wakeup/s tổng · 0 timer polling
- **AND** the Vietnamese source reads "Wakeup của supervisor/daemon khi 5 VM đang chạy"

### Requirement: Crash detection and no stuck states (NFR-05-04)
Virtax SHALL meet NFR-05-04 (BRD-05, Reliability, phase P1): Crash detection and no stuck states — ≤ 1 s · 0 VM kẹt sau 1.000 vòng chaos.

#### Scenario: Target for NFR-05-04
- **WHEN** Crash detection and no stuck states is measured
- **THEN** the result meets: ≤ 1 s · 0 VM kẹt sau 1.000 vòng chaos
- **AND** the Vietnamese source reads "Phát hiện crash và không kẹt trạng thái"

### Requirement: Snapshot safety under power loss/process kill midway (NFR-05-05)
Virtax SHALL meet NFR-05-05 (BRD-05, Reliability, phase P1): Snapshot safety under power loss/process kill midway — 100/100 fault injection → cây nhất quán.

#### Scenario: Target for NFR-05-05
- **WHEN** Snapshot safety under power loss/process kill midway is measured
- **THEN** the result meets: 100/100 fault injection → cây nhất quán
- **AND** the Vietnamese source reads "Snapshot an toàn khi mất điện/kill process giữa chừng"

### Requirement: Merge throughput and impact on the foreground VM (NFR-05-06)
Virtax SHALL meet NFR-05-06 (BRD-05, Performance, phase P1): Merge throughput and impact on the foreground VM — ≥ 70% tuần tự (offline) · p95 latency ≤ 1.5× (online).

#### Scenario: Target for NFR-05-06
- **WHEN** Merge throughput and impact on the foreground VM is measured
- **THEN** the result meets: ≥ 70% tuần tự (offline) · p95 latency ≤ 1.5× (online)
- **AND** the Vietnamese source reads "Thông lượng merge và ảnh hưởng tới VM foreground"

### Requirement: Log viewer (NFR-05-07)
Virtax SHALL meet NFR-05-07 (BRD-05, Performance, phase P1): Log viewer — 256 MB mở ≤ 1 s · regex 1M dòng ≤ 300 ms · RAM +≤ 64 MB.

#### Scenario: Target for NFR-05-07
- **WHEN** Log viewer is measured
- **THEN** the result meets: 256 MB mở ≤ 1 s · regex 1M dòng ≤ 300 ms · RAM +≤ 64 MB
- **AND** the Vietnamese source reads "Log viewer"

### Requirement: Performance tab cost (NFR-05-08)
Virtax SHALL meet NFR-05-08 (BRD-05, Performance, phase P1): Performance tab cost — ≤ 0.5% CPU host khi mở · 0 khi đóng.

#### Scenario: Target for NFR-05-08
- **WHEN** Performance tab cost is measured
- **THEN** the result meets: ≤ 0.5% CPU host khi mở · 0 khi đóng
- **AND** the Vietnamese source reads "Chi phí tab Performance"

### Requirement: Host↔guest file copy (NFR-05-09)
Virtax SHALL meet NFR-05-09 (BRD-05, Performance, phase P1): Host↔guest file copy — P1 ≥ 2× VBox 7.2 · P2 ≥ 200 MB/s.

#### Scenario: Target for NFR-05-09
- **WHEN** Host↔guest file copy is measured
- **THEN** the result meets: P1 ≥ 2× VBox 7.2 · P2 ≥ 200 MB/s
- **AND** the Vietnamese source reads "Chép file host↔guest"

### Requirement: Tool tabs meet WCAG 2.2 AA (NFR-05-10)
Virtax SHALL meet NFR-05-10 (BRD-05, Accessibility, phase P1): Tool tabs meet WCAG 2.2 AA — 100% bàn phím · bảng thay thế cho chart.

#### Scenario: Target for NFR-05-10
- **WHEN** Tool tabs meet WCAG 2.2 AA is measured
- **THEN** the result meets: 100% bàn phím · bảng thay thế cho chart
- **AND** the Vietnamese source reads "Các tab công cụ đạt WCAG 2.2 AA"

### Requirement: Open the Media manager (500 media, warm cache) (NFR-06-01)
Virtax SHALL meet NFR-06-01 (BRD-06, Performance, phase P1): Open the Media manager (500 media, warm cache) — p95 ≤ 300 ms · 0 image open.

#### Scenario: Target for NFR-06-01
- **WHEN** Open the Media manager (500 media, warm cache) is measured
- **THEN** the result meets: p95 ≤ 300 ms · 0 image open
- **AND** the Vietnamese source reads "Mở Media manager (500 medium, cache ấm)"

### Requirement: virtaxd starts without probing media (NFR-06-02)
Virtax SHALL meet NFR-06-02 (BRD-06, Performance, phase P1): virtaxd starts without probing media — 0 VDOpen/open() image file · cold start ≤ 1 s.

#### Scenario: Target for NFR-06-02
- **WHEN** virtaxd starts without probing media is measured
- **THEN** the result meets: 0 VDOpen/open() image file · cold start ≤ 1 s
- **AND** the Vietnamese source reads "virtaxd khởi động không probe media"

### Requirement: Phase-1 throughput after extradata tuning (NFR-06-03)
Virtax SHALL meet NFR-06-03 (BRD-06, Performance, phase P1): Phase-1 throughput after extradata tuning — fio 1 MiB seq QD32 ≥ 1.5× stock VBox.

#### Scenario: Target for NFR-06-03
- **WHEN** Phase-1 throughput after extradata tuning is measured
- **THEN** the result meets: fio 1 MiB seq QD32 ≥ 1.5× stock VBox
- **AND** the Vietnamese source reads "Throughput Pha 1 sau tinh chỉnh extradata"

### Requirement: virtax-vmm random IOPS vs VBox 7.2 (BG-05) (NFR-06-04)
Virtax SHALL meet NFR-06-04 (BRD-06, Performance, phase P2): virtax-vmm random IOPS vs VBox 7.2 (BG-05) — 4k randread QD32 ≥ 1.5× · 3 OS.

#### Scenario: Target for NFR-06-04
- **WHEN** virtax-vmm random IOPS vs VBox 7.2 (BG-05) is measured
- **THEN** the result meets: 4k randread QD32 ≥ 1.5× · 3 OS
- **AND** the Vietnamese source reads "IOPS ngẫu nhiên của virtax-vmm so với VBox 7.2 (BG-05)"

### Requirement: Host CPU cost of I/O (NFR-06-05)
Virtax SHALL meet NFR-06-05 (BRD-06, Performance, phase P2): Host CPU cost of I/O — ≤ 60% CPU/100k IOPS so với VBox.

#### Scenario: Target for NFR-06-05
- **WHEN** Host CPU cost of I/O is measured
- **THEN** the result meets: ≤ 60% CPU/100k IOPS so với VBox
- **AND** the Vietnamese source reads "Chi phí CPU host cho I/O"

### Requirement: Block-backend wakeups with no I/O (NFR-06-06)
Virtax SHALL meet NFR-06-06 (BRD-06, Performance, phase P2): Block-backend wakeups with no I/O — 0 wakeup/s.

#### Scenario: Target for NFR-06-06
- **WHEN** Block-backend wakeups with no I/O is measured
- **THEN** the result meets: 0 wakeup/s
- **AND** the Vietnamese source reads "Wakeup của block backend khi không có I/O"

### Requirement: Data durability on crash/power loss (NFR-06-07)
Virtax SHALL meet NFR-06-07 (BRD-06, Reliability, phase P2): Data durability on crash/power loss — 1 000 lần crash: 0 lỗi metadata · 0 mất dữ liệu đã fsync.

#### Scenario: Target for NFR-06-07
- **WHEN** Data durability on crash/power loss is measured
- **THEN** the result meets: 1 000 lần crash: 0 lỗi metadata · 0 mất dữ liệu đã fsync
- **AND** the Vietnamese source reads "Bền vững dữ liệu khi crash/mất điện"

### Requirement: Convert/move/clone never lose data (NFR-06-08)
Virtax SHALL meet NFR-06-08 (BRD-06, Reliability, phase P1): Convert/move/clone never lose data — 0 sự cố beta · bản gốc giữ tới khi verify.

#### Scenario: Target for NFR-06-08
- **WHEN** Convert/move/clone never lose data is measured
- **THEN** the result meets: 0 sự cố beta · bản gốc giữ tới khi verify
- **AND** the Vietnamese source reads "Convert/move/clone không mất dữ liệu"

### Requirement: Space returned after TRIM (qcow2/raw) (NFR-06-09)
Virtax SHALL meet NFR-06-09 (BRD-06, Efficiency, phase P2): Space returned after TRIM (qcow2/raw) — ≥ 90% trong ≤ 60 s.

#### Scenario: Target for NFR-06-09
- **WHEN** Space returned after TRIM (qcow2/raw) is measured
- **THEN** the result meets: ≥ 90% trong ≤ 60 s
- **AND** the Vietnamese source reads "Trả dung lượng sau TRIM (qcow2/raw)"

### Requirement: Media, ISO library, Preferences meet WCAG 2.2 AA (NFR-06-10)
Virtax SHALL meet NFR-06-10 (BRD-06, Accessibility, phase P1): Media, ISO library, Preferences meet WCAG 2.2 AA — 100% bàn phím · contrast ≥ 4.5:1 · nhãn AccessKit.

#### Scenario: Target for NFR-06-10
- **WHEN** Media, ISO library, Preferences meet WCAG 2.2 AA is measured
- **THEN** the result meets: 100% bàn phím · contrast ≥ 4.5:1 · nhãn AccessKit
- **AND** the Vietnamese source reads "Media, ISO library, Preferences theo WCAG 2.2 AA"

### Requirement: virtax-vmm NAT guest→host throughput (iperf3) (NFR-07-01)
Virtax SHALL meet NFR-07-01 (BRD-07, Performance, phase P2): virtax-vmm NAT guest→host throughput (iperf3) — ≥ 2× VBox 7.2 · 3 OS.

#### Scenario: Target for NFR-07-01
- **WHEN** virtax-vmm NAT guest→host throughput (iperf3) is measured
- **THEN** the result meets: ≥ 2× VBox 7.2 · 3 OS
- **AND** the Vietnamese source reads "Throughput NAT guest→host của virtax-vmm (iperf3)"

### Requirement: Bridged/host-only throughput (TAP+vhost-net, vmnet) (NFR-07-02)
Virtax SHALL meet NFR-07-02 (BRD-07, Performance, phase P2): Bridged/host-only throughput (TAP+vhost-net, vmnet) — ≥ 2× VBox 7.2 · ≥ 1.5× qua helper relay.

#### Scenario: Target for NFR-07-02
- **WHEN** Bridged/host-only throughput (TAP+vhost-net, vmnet) is measured
- **THEN** the result meets: ≥ 2× VBox 7.2 · ≥ 1.5× qua helper relay
- **AND** the Vietnamese source reads "Throughput bridged/host-only (TAP+vhost-net, vmnet)"

### Requirement: Engine Pack vmnet batching (macOS) (NFR-07-03)
Virtax SHALL meet NFR-07-03 (BRD-07, Performance, phase P1): Engine Pack vmnet batching (macOS) — ≥ 1.5× VBox 7.2 gốc · CPU ≤ gốc.

#### Scenario: Target for NFR-07-03
- **WHEN** Engine Pack vmnet batching (macOS) is measured
- **THEN** the result meets: ≥ 1.5× VBox 7.2 gốc · CPU ≤ gốc
- **AND** the Vietnamese source reads "vmnet batch trong Engine Pack (macOS)"

### Requirement: Guest↔host latency (ping, TAP+vhost-net) (NFR-07-04)
Virtax SHALL meet NFR-07-04 (BRD-07, Performance, phase P2): Guest↔host latency (ping, TAP+vhost-net) — p50 ≤ 0.15 ms · p99 ≤ 0.5 ms.

#### Scenario: Target for NFR-07-04
- **WHEN** Guest↔host latency (ping, TAP+vhost-net) is measured
- **THEN** the result meets: p50 ≤ 0.15 ms · p99 ≤ 0.5 ms
- **AND** the Vietnamese source reads "Độ trễ guest↔host (ping, TAP+vhost-net)"

### Requirement: Network-backend wakeups without traffic (NFR-07-05)
Virtax SHALL meet NFR-07-05 (BRD-07, Efficiency, phase P2): Network-backend wakeups without traffic — 0 wakeup/s (ngoài timer guest).

#### Scenario: Target for NFR-07-05
- **WHEN** Network-backend wakeups without traffic is measured
- **THEN** the result meets: 0 wakeup/s (ngoài timer guest)
- **AND** the Vietnamese source reads "Wakeup của backend mạng khi không có traffic"

### Requirement: Port-forward conflicts detected before start (NFR-07-06)
Virtax SHALL meet NFR-07-06 (BRD-07, Reliability, phase P1): Port-forward conflicts detected before start — 100% · kiểm ≤ 50 ms.

#### Scenario: Target for NFR-07-06
- **WHEN** Port-forward conflicts detected before start is measured
- **THEN** the result meets: 100% · kiểm ≤ 50 ms
- **AND** the Vietnamese source reads "Phát hiện xung đột port forward trước start"

### Requirement: DNS recovery after host network/VPN change (NFR-07-07)
Virtax SHALL meet NFR-07-07 (BRD-07, Reliability, phase P1): DNS recovery after host network/VPN change — ≤ 5 s · không cần renew guest.

#### Scenario: Target for NFR-07-07
- **WHEN** DNS recovery after host network/VPN change is measured
- **THEN** the result meets: ≤ 5 s · không cần renew guest
- **AND** the Vietnamese source reads "Phục hồi DNS khi host đổi mạng/VPN"

### Requirement: Minimal network privilege (NFR-07-08)
Virtax SHALL meet NFR-07-08 (BRD-07, Security, phase P2): Minimal network privilege — 0 driver mạng riêng · ≤ 8 lệnh helper · virtax-vmm 0 capability.

#### Scenario: Target for NFR-07-08
- **WHEN** Minimal network privilege is measured
- **THEN** the result meets: 0 driver mạng riêng · ≤ 8 lệnh helper · virtax-vmm 0 capability
- **AND** the Vietnamese source reads "Đặc quyền mạng tối thiểu"

### Requirement: Create a host-only network and attach a VM (NFR-07-09)
Virtax SHALL meet NFR-07-09 (BRD-07, Usability, phase P1): Create a host-only network and attach a VM — ≤ 3 thao tác · ≥ 80% không trợ giúp.

#### Scenario: Target for NFR-07-09
- **WHEN** Create a host-only network and attach a VM is measured
- **THEN** the result meets: ≤ 3 thao tác · ≥ 80% không trợ giúp
- **AND** the Vietnamese source reads "Tạo host-only network và gắn VM"

### Requirement: virtax-agent footprint (NFR-08-01)
Virtax SHALL meet NFR-08-01 (BRD-08, Performance, phase P2): virtax-agent footprint — binary ≤ 6 MB · RSS ≤ 15 MB · idle ≤ 0.2 wakeup/s.

#### Scenario: Target for NFR-08-01
- **WHEN** virtax-agent footprint is measured
- **THEN** the result meets: binary ≤ 6 MB · RSS ≤ 15 MB · idle ≤ 0.2 wakeup/s
- **AND** the Vietnamese source reads "Footprint của virtax-agent"

### Requirement: Host → guest clipboard latency (1 MB text) (NFR-08-02)
Virtax SHALL meet NFR-08-02 (BRD-08, Performance, phase P1): Host → guest clipboard latency (1 MB text) — p95 ≤ 300 ms.

#### Scenario: Target for NFR-08-02
- **WHEN** Host → guest clipboard latency (1 MB text) is measured
- **THEN** the result meets: p95 ≤ 300 ms
- **AND** the Vietnamese source reads "Độ trễ clipboard host → guest (1 MB text)"

### Requirement: Auto-resize latency after mouse release (NFR-08-03)
Virtax SHALL meet NFR-08-03 (BRD-08, Performance, phase P1): Auto-resize latency after mouse release — p95 ≤ 500 ms.

#### Scenario: Target for NFR-08-03
- **WHEN** Auto-resize latency after mouse release is measured
- **THEN** the result meets: p95 ≤ 500 ms
- **AND** the Vietnamese source reads "Độ trễ auto-resize sau khi thả chuột"

### Requirement: Host → guest file copy throughput (4 GB) (NFR-08-04)
Virtax SHALL meet NFR-08-04 (BRD-08, Performance, phase P2): Host → guest file copy throughput (4 GB) — P1 ≥ 2× VBox · P2 ≥ 300 MB/s (KVM).

#### Scenario: Target for NFR-08-04
- **WHEN** Host → guest file copy throughput (4 GB) is measured
- **THEN** the result meets: P1 ≥ 2× VBox · P2 ≥ 300 MB/s (KVM)
- **AND** the Vietnamese source reads "Throughput chép file host → guest (4 GB)"

### Requirement: Shared-folder metadata (git status, 50k files) (NFR-08-05)
Virtax SHALL meet NFR-08-05 (BRD-08, Performance, phase P2): Shared-folder metadata (git status, 50k files) — virtio-fs ≥ 2× vbsf.

#### Scenario: Target for NFR-08-05
- **WHEN** Shared-folder metadata (git status, 50k files) is measured
- **THEN** the result meets: virtio-fs ≥ 2× vbsf
- **AND** the Vietnamese source reads "Metadata shared folder (git status, 50k file)"

### Requirement: USB hotplug detection and attach (NFR-08-06)
Virtax SHALL meet NFR-08-06 (BRD-08, Performance, phase P2): USB hotplug detection and attach — phát hiện ≤ 1 s · attach ≤ 2 s · 0 poll.

#### Scenario: Target for NFR-08-06
- **WHEN** USB hotplug detection and attach is measured
- **THEN** the result meets: phát hiện ≤ 1 s · attach ≤ 2 s · 0 poll
- **AND** the Vietnamese source reads "Phát hiện hotplug USB và attach"

### Requirement: Integration working at first login (Quick Create) (NFR-08-07)
Virtax SHALL meet NFR-08-07 (BRD-08, Reliability, phase P1): Integration working at first login (Quick Create) — ≥ 95% lần chạy CI · Ubuntu + Win11.

#### Scenario: Target for NFR-08-07
- **WHEN** Integration working at first login (Quick Create) is measured
- **THEN** the result meets: ≥ 95% lần chạy CI · Ubuntu + Win11
- **AND** the Vietnamese source reads "Tích hợp chạy ở lần đăng nhập đầu (Quick Create)"

### Requirement: Agent-channel parser robustness against malicious guests (NFR-08-08)
Virtax SHALL meet NFR-08-08 (BRD-08, Security, phase P2): Agent-channel parser robustness against malicious guests — fuzz 24 h: 0 panic · rate limit · message ≤ 1 MiB.

#### Scenario: Target for NFR-08-08
- **WHEN** Agent-channel parser robustness against malicious guests is measured
- **THEN** the result meets: fuzz 24 h: 0 panic · rate limit · message ≤ 1 MiB
- **AND** the Vietnamese source reads "Độ bền parser kênh agent trước guest độc hại"

### Requirement: Agent version compatibility (NFR-08-09)
Virtax SHALL meet NFR-08-09 (BRD-08, Compatibility, phase P2): Agent version compatibility — host hỗ trợ agent N-2 · cập nhật không reboot.

#### Scenario: Target for NFR-08-09
- **WHEN** Agent version compatibility is measured
- **THEN** the result meets: host hỗ trợ agent N-2 · cập nhật không reboot
- **AND** the Vietnamese source reads "Tương thích version agent"

### Requirement: Local API latency (NFR-09-01)
Virtax SHALL meet NFR-09-01 (BRD-09, Performance, phase P1): Local API latency — đọc p95 ≤ 5 ms · ghi ≤ 50 ms · ListMachines(50) ≤ 10 ms.

#### Scenario: Target for NFR-09-01
- **WHEN** Local API latency is measured
- **THEN** the result meets: đọc p95 ≤ 5 ms · ghi ≤ 50 ms · ListMachines(50) ≤ 10 ms
- **AND** the Vietnamese source reads "Độ trễ API cục bộ"

### Requirement: Event delivery (NFR-09-02)
Virtax SHALL meet NFR-09-02 (BRD-09, Reliability, phase P1): Event delivery — p95 ≤ 50 ms · 0 event kết thúc mất / 10k burst.

#### Scenario: Target for NFR-09-02
- **WHEN** Event delivery is measured
- **THEN** the result meets: p95 ≤ 50 ms · 0 event kết thúc mất / 10k burst
- **AND** the Vietnamese source reads "Giao event"

### Requirement: Daemon at idle (NFR-09-03)
Virtax SHALL meet NFR-09-03 (BRD-09, Performance, phase P1): Daemon at idle — ≤ 5 wakeup/s · ≤ 0.1% CPU · RSS ≤ 40 MB (50 VM).

#### Scenario: Target for NFR-09-03
- **WHEN** Daemon at idle is measured
- **THEN** the result meets: ≤ 5 wakeup/s · ≤ 0.1% CPU · RSS ≤ 40 MB (50 VM)
- **AND** the Vietnamese source reads "Daemon khi rảnh"

### Requirement: RPC throughput and concurrent streams (NFR-09-04)
Virtax SHALL meet NFR-09-04 (BRD-09, Scalability, phase P1): RPC throughput and concurrent streams — ≥ 2,000 RPC/s đọc/core · 100 Watch stream.

#### Scenario: Target for NFR-09-04
- **WHEN** RPC throughput and concurrent streams is measured
- **THEN** the result meets: ≥ 2,000 RPC/s đọc/core · 100 Watch stream
- **AND** the Vietnamese source reads "Thông lượng RPC và số stream đồng thời"

### Requirement: Per-user isolation and proto decoding robustness (NFR-09-05)
Virtax SHALL meet NFR-09-05 (BRD-09, Security, phase P1): Per-user isolation and proto decoding robustness — 0 truy cập chéo user · cargo-fuzz 24 h 0 crash.

#### Scenario: Target for NFR-09-05
- **WHEN** Per-user isolation and proto decoding robustness is measured
- **THEN** the result meets: 0 truy cập chéo user · cargo-fuzz 24 h 0 crash
- **AND** the Vietnamese source reads "Cách ly theo user và độ bền giải mã proto"

### Requirement: API and CLI contract stability (NFR-09-06)
Virtax SHALL meet NFR-09-06 (BRD-09, Compatibility, phase P1): API and CLI contract stability — buf breaking 0 vi phạm trong v1 · JSON schema semver.

#### Scenario: Target for NFR-09-06
- **WHEN** API and CLI contract stability is measured
- **THEN** the result meets: buf breaking 0 vi phạm trong v1 · JSON schema semver
- **AND** the Vietnamese source reads "Ổn định contract API và CLI"

### Requirement: CLI (NFR-09-07)
Virtax SHALL meet NFR-09-07 (BRD-09, Performance, phase P1): CLI — cold start ≤ 30 ms · completion ≤ 100 ms.

#### Scenario: Target for NFR-09-07
- **WHEN** CLI is measured
- **THEN** the result meets: cold start ≤ 30 ms · completion ≤ 100 ms
- **AND** the Vietnamese source reads "CLI"

### Requirement: Observability overhead (NFR-09-08)
Virtax SHALL meet NFR-09-08 (BRD-09, Performance, phase P1): Observability overhead — metrics ≤ 0.2% core (10 VM @1 Hz) · log ≤ 1% CPU.

#### Scenario: Target for NFR-09-08
- **WHEN** Observability overhead is measured
- **THEN** the result meets: metrics ≤ 0.2% core (10 VM @1 Hz) · log ≤ 1% CPU
- **AND** the Vietnamese source reads "Chi phí quan sát hệ thống"

### Requirement: Diagnostics data and egress (NFR-09-09)
Virtax SHALL meet NFR-09-09 (BRD-09, Privacy, phase P1): Diagnostics data and egress — 0 rò rỉ trong corpus · 0 egress khi chưa opt-in.

#### Scenario: Target for NFR-09-09
- **WHEN** Diagnostics data and egress is measured
- **THEN** the result meets: 0 rò rỉ trong corpus · 0 egress khi chưa opt-in
- **AND** the Vietnamese source reads "Dữ liệu chẩn đoán và egress"

### Requirement: Client reconnect after daemon restart (NFR-09-10)
Virtax SHALL meet NFR-09-10 (BRD-09, Reliability, phase P1): Client reconnect after daemon restart — reconnect + resync ≤ 2 s.

#### Scenario: Target for NFR-09-10
- **WHEN** Client reconnect after daemon restart is measured
- **THEN** the result meets: reconnect + resync ≤ 2 s
- **AND** the Vietnamese source reads "Client kết nối lại sau khi daemon restart"

### Requirement: Calls through LegacyEngine (local) (NFR-10-01)
Virtax SHALL meet NFR-10-01 (BRD-10, Performance, phase P1): Calls through LegacyEngine (local) — p95 ≤ 5 ms read · ≤ 50 ms update.

#### Scenario: Target for NFR-10-01
- **WHEN** Calls through LegacyEngine (local) is measured
- **THEN** the result meets: p95 ≤ 5 ms read · ≤ 50 ms update
- **AND** the Vietnamese source reads "Lời gọi qua LegacyEngine (cục bộ)"

### Requirement: Engine Pack VM process, idle Linux guest (NFR-10-02)
Virtax SHALL meet NFR-10-02 (BRD-10, Performance, phase P1): Engine Pack VM process, idle Linux guest — ≤ 1% one core · ≤ 20 wakeups/s.

#### Scenario: Target for NFR-10-02
- **WHEN** Engine Pack VM process, idle Linux guest is measured
- **THEN** the result meets: ≤ 1% one core · ≤ 20 wakeups/s
- **AND** the Vietnamese source reads "VM process Engine Pack, guest Linux idle"

### Requirement: `virtax-vmm` VM, idle Linux guest (NFR-10-03)
Virtax SHALL meet NFR-10-03 (BRD-10, Performance, phase P2): `virtax-vmm` VM, idle Linux guest — ≤ 0.5% one core · ≤ 10 wakeups/s.

#### Scenario: Target for NFR-10-03
- **WHEN** `virtax-vmm` VM, idle Linux guest is measured
- **THEN** the result meets: ≤ 0.5% one core · ≤ 10 wakeups/s
- **AND** the Vietnamese source reads "VM `virtax-vmm`, guest Linux idle"

### Requirement: `virtax-vmm` I/O versus VirtualBox 7.2 on the same host (NFR-10-04)
Virtax SHALL meet NFR-10-04 (BRD-10, Performance, phase P2): `virtax-vmm` I/O versus VirtualBox 7.2 on the same host — fio 4k QD32 ≥ 1.5× · iperf3 ≥ 2×.

#### Scenario: Target for NFR-10-04
- **WHEN** `virtax-vmm` I/O versus VirtualBox 7.2 on the same host is measured
- **THEN** the result meets: fio 4k QD32 ≥ 1.5× · iperf3 ≥ 2×
- **AND** the Vietnamese source reads "I/O của `virtax-vmm` so với VirtualBox 7.2 cùng host"

### Requirement: VM start on `virtax-vmm` (NFR-10-05)
Virtax SHALL meet NFR-10-05 (BRD-10, Performance, phase P2): VM start on `virtax-vmm` — request → firmware ≤ 300 ms · Ubuntu login ≤ 10 s.

#### Scenario: Target for NFR-10-05
- **WHEN** VM start on `virtax-vmm` is measured
- **THEN** the result meets: request → firmware ≤ 300 ms · Ubuntu login ≤ 10 s
- **AND** the Vietnamese source reads "Khởi động VM trên `virtax-vmm`"

### Requirement: VMM memory overhead (2D, excluding guest RAM) (NFR-10-06)
Virtax SHALL meet NFR-10-06 (BRD-10, Performance, phase P2): VMM memory overhead (2D, excluding guest RAM) — RSS ≤ 64 MiB per VM.

#### Scenario: Target for NFR-10-06
- **WHEN** VMM memory overhead (2D, excluding guest RAM) is measured
- **THEN** the result meets: RSS ≤ 64 MiB per VM
- **AND** the Vietnamese source reads "Overhead bộ nhớ của VMM (2D, không tính RAM guest)"

### Requirement: VMs survive daemon restarts; long soak (NFR-10-07)
Virtax SHALL meet NFR-10-07 (BRD-10, Reliability, phase P2): VMs survive daemon restarts; long soak — 0 VM killed · 24 h soak on 5 combos.

#### Scenario: Target for NFR-10-07
- **WHEN** VMs survive daemon restarts; long soak is measured
- **THEN** the result meets: 0 VM killed · 24 h soak on 5 combos
- **AND** the Vietnamese source reads "VM sống sót khi daemon restart; soak dài"

### Requirement: Bounded `unsafe`, fuzzed guest-facing parsers (NFR-10-08)
Virtax SHALL meet NFR-10-08 (BRD-10, Security, phase P2): Bounded `unsafe`, fuzzed guest-facing parsers — unsafe only in *-sys/virtax-vmm-hv · ≥ 24 CPU-h fuzz/release.

#### Scenario: Target for NFR-10-08
- **WHEN** Bounded `unsafe`, fuzzed guest-facing parsers is measured
- **THEN** the result meets: unsafe only in *-sys/virtax-vmm-hv · ≥ 24 CPU-h fuzz/release
- **AND** the Vietnamese source reads "Giới hạn `unsafe`, fuzz parser hướng guest"

### Requirement: Hypervisor-specific code stays in `virtax-vmm-hv` (NFR-10-09)
Virtax SHALL meet NFR-10-09 (BRD-10, Portability, phase P2): Hypervisor-specific code stays in `virtax-vmm-hv` — ≤ 5% elsewhere.

#### Scenario: Target for NFR-10-09
- **WHEN** Hypervisor-specific code stays in `virtax-vmm-hv` is measured
- **THEN** the result meets: ≤ 5% elsewhere
- **AND** the Vietnamese source reads "Code đặc thù hypervisor nằm trong `virtax-vmm-hv`"

### Requirement: VirtualBox 7.2.x support, no unintended `.vbox` changes (NFR-10-10)
Virtax SHALL meet NFR-10-10 (BRD-10, Compatibility, phase P1): VirtualBox 7.2.x support, no unintended `.vbox` changes — 7.2.0-7.2.x tested · 0 GUI/* keys written.

#### Scenario: Target for NFR-10-10
- **WHEN** VirtualBox 7.2.x support, no unintended `.vbox` changes is measured
- **THEN** the result meets: 7.2.0-7.2.x tested · 0 GUI/* keys written
- **AND** the Vietnamese source reads "Hỗ trợ VirtualBox 7.2.x, không đổi `.vbox` ngoài ý muốn"

### Requirement: Small, audited privileged code (NFR-11-01)
Virtax SHALL meet NFR-11-01 (BRD-11, Security, phase P1): Small, audited privileged code — helper ≤ 3k LOC · audit 0 High before GA.

#### Scenario: Target for NFR-11-01
- **WHEN** Small, audited privileged code is measured
- **THEN** the result meets: helper ≤ 3k LOC · audit 0 High before GA
- **AND** the Vietnamese source reads "Code đặc quyền nhỏ và được audit"

### Requirement: No setuid, no sockets open to other users (NFR-11-02)
Virtax SHALL meet NFR-11-02 (BRD-11, Security, phase P1): No setuid, no sockets open to other users — 0 setuid (P2) · 0 world-accessible IPC.

#### Scenario: Target for NFR-11-02
- **WHEN** No setuid, no sockets open to other users is measured
- **THEN** the result meets: 0 setuid (P2) · 0 world-accessible IPC
- **AND** the Vietnamese source reads "Không setuid, không socket mở cho user khác"

### Requirement: Fuzzing of guest-facing parsers (NFR-11-03)
Virtax SHALL meet NFR-11-03 (BRD-11, Security, phase P2): Fuzzing of guest-facing parsers — ≥ 24 CPU-h/release · 0 open crashes.

#### Scenario: Target for NFR-11-03
- **WHEN** Fuzzing of guest-facing parsers is measured
- **THEN** the result meets: ≥ 24 CPU-h/release · 0 open crashes
- **AND** the Vietnamese source reads "Fuzzing parser hướng guest"

### Requirement: Licensing, SBOM, reproducible builds (NFR-11-04)
Virtax SHALL meet NFR-11-04 (BRD-11, Supply chain, phase P1): Licensing, SBOM, reproducible builds — cargo-deny 0 · SBOM 100% · Linux bit-identical.

#### Scenario: Target for NFR-11-04
- **WHEN** Licensing, SBOM, reproducible builds is measured
- **THEN** the result meets: cargo-deny 0 · SBOM 100% · Linux bit-identical
- **AND** the Vietnamese source reads "License, SBOM, build tái lập"

### Requirement: Security overhead at Windows VM start (NFR-11-05)
Virtax SHALL meet NFR-11-05 (BRD-11, Performance, phase P2): Security overhead at Windows VM start — ≤ 50 ms (vs ≥ 640 ms).

#### Scenario: Target for NFR-11-05
- **WHEN** Security overhead at Windows VM start is measured
- **THEN** the result meets: ≤ 50 ms (vs ≥ 640 ms)
- **AND** the Vietnamese source reads "Overhead bảo mật khi start VM trên Windows"

### Requirement: Disk-encryption overhead (NFR-11-06)
Virtax SHALL meet NFR-11-06 (BRD-11, Performance, phase P2): Disk-encryption overhead — throughput ≤ −15% · CPU ≤ +5%.

#### Scenario: Target for NFR-11-06
- **WHEN** Disk-encryption overhead is measured
- **THEN** the result meets: throughput ≤ −15% · CPU ≤ +5%
- **AND** the Vietnamese source reads "Overhead mã hoá disk"

### Requirement: Update check and apply (NFR-11-07)
Virtax SHALL meet NFR-11-07 (BRD-11, Update, phase P1): Update check and apply — ≤ 1 check/24 h · apply ≤ 2 min · 0 VMs stopped · no reboot.

#### Scenario: Target for NFR-11-07
- **WHEN** Update check and apply is measured
- **THEN** the result meets: ≤ 1 check/24 h · apply ≤ 2 min · 0 VMs stopped · no reboot
- **AND** the Vietnamese source reads "Kiểm tra và áp dụng cập nhật"

### Requirement: Automatic rollback on a failing build (NFR-11-08)
Virtax SHALL meet NFR-11-08 (BRD-11, Reliability, phase P1): Automatic rollback on a failing build — ≥ 99% success in fault injection · ≤ 30 s.

#### Scenario: Target for NFR-11-08
- **WHEN** Automatic rollback on a failing build is measured
- **THEN** the result meets: ≥ 99% success in fault injection · ≤ 30 s
- **AND** the Vietnamese source reads "Rollback tự động khi bản mới lỗi"

### Requirement: No identifiers in update requests (NFR-11-09)
Virtax SHALL meet NFR-11-09 (BRD-11, Privacy, phase P1): No identifiers in update requests — 0 persistent IDs · 0 egress when disabled.

#### Scenario: Target for NFR-11-09
- **WHEN** No identifiers in update requests is measured
- **THEN** the result meets: 0 persistent IDs · 0 egress when disabled
- **AND** the Vietnamese source reads "Không định danh trong request cập nhật"

### Requirement: Plugin isolation (NFR-11-10)
Virtax SHALL meet NFR-11-10 (BRD-11, Isolation, phase P2): Plugin isolation — crash impact 0 · idle ≤ 0.1% CPU · ≤ 30 MB RSS.

#### Scenario: Target for NFR-11-10
- **WHEN** Plugin isolation is measured
- **THEN** the result meets: crash impact 0 · idle ≤ 0.1% CPU · ≤ 30 MB RSS
- **AND** the Vietnamese source reads "Cô lập plugin"

### Requirement: Workspace build from a clean clone / incremental (8-core laptop) (NFR-12-01)
Virtax SHALL meet NFR-12-01 (BRD-12, Build, phase P1): Workspace build from a clean clone / incremental (8-core laptop) — ≤ 15 min · ≤ 2 min.

#### Scenario: Target for NFR-12-01
- **WHEN** Workspace build from a clean clone / incremental (8-core laptop) is measured
- **THEN** the result meets: ≤ 15 min · ≤ 2 min
- **AND** the Vietnamese source reads "Build workspace từ clone sạch / incremental (laptop 8 core)"

### Requirement: PR CI duration and test stability (NFR-12-02)
Virtax SHALL meet NFR-12-02 (BRD-12, CI, phase P1): PR CI duration and test stability — p90 ≤ 20 min · flaky ≤ 1%.

#### Scenario: Target for NFR-12-02
- **WHEN** PR CI duration and test stability is measured
- **THEN** the result meets: p90 ≤ 20 min · flaky ≤ 1%
- **AND** the Vietnamese source reads "Thời gian CI cho PR và độ ổn định test"

### Requirement: Tag to signed release (NFR-12-03)
Virtax SHALL meet NFR-12-03 (BRD-12, Release, phase P1): Tag to signed release — ≤ 2 h · 0 manual steps.

#### Scenario: Target for NFR-12-03
- **WHEN** Tag to signed release is measured
- **THEN** the result meets: ≤ 2 h · 0 manual steps
- **AND** the Vietnamese source reads "Từ tag tới bản phát hành có chữ ký"

### Requirement: Install and upgrade (NFR-12-04)
Virtax SHALL meet NFR-12-04 (BRD-12, Install, phase P1): Install and upgrade — ≤ 2 min · no reboot · no kernel build.

#### Scenario: Target for NFR-12-04
- **WHEN** Install and upgrade is measured
- **THEN** the result meets: ≤ 2 min · no reboot · no kernel build
- **AND** the Vietnamese source reads "Cài đặt và nâng cấp"

### Requirement: Manager + daemon + CLI package size (no Engine Pack) (NFR-12-05)
Virtax SHALL meet NFR-12-05 (BRD-12, Install, phase P1): Manager + daemon + CLI package size (no Engine Pack) — ≤ 60 MB per arch.

#### Scenario: Target for NFR-12-05
- **WHEN** Manager + daemon + CLI package size (no Engine Pack) is measured
- **THEN** the result meets: ≤ 60 MB per arch
- **AND** the Vietnamese source reads "Kích thước gói Manager + daemon + CLI (không Engine Pack)"

### Requirement: Idle cost of the daemon and helper (NFR-12-06)
Virtax SHALL meet NFR-12-06 (BRD-12, Performance, phase P1): Idle cost of the daemon and helper — daemon ≤ 20 wakeups/s, ≤ 0.2% CPU · helper not running.

#### Scenario: Target for NFR-12-06
- **WHEN** Idle cost of the daemon and helper is measured
- **THEN** the result meets: daemon ≤ 20 wakeups/s, ≤ 0.2% CPU · helper not running
- **AND** the Vietnamese source reads "Chi phí idle của daemon và helper"

### Requirement: System-wide timer-resolution changes by Virtax (NFR-12-07)
Virtax SHALL meet NFR-12-07 (BRD-12, Performance, phase P1): System-wide timer-resolution changes by Virtax — 0 (powercfg /energy).

#### Scenario: Target for NFR-12-07
- **WHEN** System-wide timer-resolution changes by Virtax is measured
- **THEN** the result meets: 0 (powercfg /energy)
- **AND** the Vietnamese source reads "Thay đổi timer resolution toàn hệ thống do Virtax"

### Requirement: Platform baseline (NFR-12-08)
Virtax SHALL meet NFR-12-08 (BRD-12, Compatibility, phase P1): Platform baseline — glibc ≥ 2.28 · Windows 10 22H2+ · macOS 14+.

#### Scenario: Target for NFR-12-08
- **WHEN** Platform baseline is measured
- **THEN** the result meets: glibc ≥ 2.28 · Windows 10 22H2+ · macOS 14+
- **AND** the Vietnamese source reads "Baseline nền tảng"

### Requirement: Hypervisor check latency (NFR-12-09)
Virtax SHALL meet NFR-12-09 (BRD-12, Usability, phase P1): Hypervisor check latency — ≤ 500 ms cold · ≤ 20 ms cached.

#### Scenario: Target for NFR-12-09
- **WHEN** Hypervisor check latency is measured
- **THEN** the result meets: ≤ 500 ms cold · ≤ 20 ms cached
- **AND** the Vietnamese source reads "Độ trễ kiểm tra hypervisor"

### Requirement: No setuid, small helper (see NFR-11-01) (NFR-12-10)
Virtax SHALL meet NFR-12-10 (BRD-12, Security, phase P1): No setuid, small helper (see NFR-11-01) — 0 setuid · helper ≤ 3k LOC.

#### Scenario: Target for NFR-12-10
- **WHEN** No setuid, small helper (see NFR-11-01) is measured
- **THEN** the result meets: 0 setuid · helper ≤ 3k LOC
- **AND** the Vietnamese source reads "Không setuid, helper nhỏ (xem NFR-11-01)"
