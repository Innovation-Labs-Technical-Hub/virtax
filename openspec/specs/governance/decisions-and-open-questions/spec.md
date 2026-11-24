# Decisions and open questions Specification

## Purpose
Product decisions (D1–D6 from the architecture document) and the open questions listed in each BRD, with their owners and proposed defaults. Nothing here is decided unless its status says CONFIRMED; agents never resolve these by guessing.

## Requirements

### Requirement: Decision D1 — UI framework
Decision D1 SHALL be settled by the project owner before work that depends on it is merged. Status: **TO BE CONFIRMED**. Source: architecture document §"Decisions needed from you".

#### Scenario: Work that depends on D1
- **WHEN** a change proposal depends on D1
- **THEN** it cites D1 and follows the confirmed decision, or, while D1 is TO BE CONFIRMED, follows the proposed default and says so
- **AND** the proposed default is: Slint (bake-off against GPUI in weeks 2-4)
- **AND** the deadline is: Week 4 — every screen depends on it

### Requirement: Decision D2 — Phase 1 engine distribution
Decision D2 SHALL be settled by the project owner before work that depends on it is merged. Status: **TO BE CONFIRMED**. Source: architecture document §"Decisions needed from you".

#### Scenario: Work that depends on D2
- **WHEN** a change proposal depends on D2
- **THEN** it cites D2 and follows the confirmed decision, or, while D2 is TO BE CONFIRMED, follows the proposed default and says so
- **AND** the proposed default is: Alpha = BYO VirtualBox 7.2.x; Beta = Engine Pack
- **AND** the deadline is: Month 2 — affects packaging & code signing

### Requirement: Decision D3 — Purchase code-signing certificates
Decision D3 SHALL be settled by the project owner before work that depends on it is merged. Status: **TO BE CONFIRMED**. Source: architecture document §"Decisions needed from you".

#### Scenario: Work that depends on D3
- **WHEN** a change proposal depends on D3
- **THEN** it cites D3 and follows the confirmed decision, or, while D3 is TO BE CONFIRMED, follows the proposed default and says so
- **AND** the proposed default is: Windows OV/EV + Apple Developer ID in month 1
- **AND** the deadline is: EV + attestation takes 2-6 weeks

### Requirement: Decision D4 — Phase 2 base engine
Decision D4 SHALL be settled by the project owner before work that depends on it is merged. Status: **TO BE CONFIRMED**. Source: architecture document §"Decisions needed from you".

#### Scenario: Work that depends on D4
- **WHEN** a change proposal depends on D4
- **THEN** it cites D4 and follows the confirmed decision, or, while D4 is TO BE CONFIRMED, follows the proposed default and says so
- **AND** the proposed default is: OpenVMM crates, rust-vmm fallback on Linux
- **AND** the deadline is: Month 6 — before hiring the VMM lead
- **AND** As built: `virtax-vmm` uses rust-vmm crates on KVM, behind `crates/virtax-vmm-hv`, because OpenVMM's crates are not published (provisional until D4 is confirmed).

### Requirement: Decision D5 — Product name & bundle ID
Decision D5 SHALL be settled by the project owner before work that depends on it is merged. Status: **CONFIRMED (2026-11-24)**. Source: architecture document §"Decisions needed from you".

#### Scenario: Work that depends on D5
- **WHEN** a change proposal depends on D5
- **THEN** it cites D5 and follows the confirmed decision, or, while D5 is TO BE CONFIRMED, follows the proposed default and says so
- **AND** the proposed default is: Keep "OxBox" until beta
- **AND** the deadline is: Before beta — affects signing, domain
- **AND** Decision: the product is named Virtax; identifiers follow it (`virtax*`, `VTX-*` error codes). The bundle ID is still TO BE CONFIRMED.

### Requirement: Decision D6 — Telemetry
Decision D6 SHALL be settled by the project owner before work that depends on it is merged. Status: **TO BE CONFIRMED**. Source: architecture document §"Decisions needed from you".

#### Scenario: Work that depends on D6
- **WHEN** a change proposal depends on D6
- **THEN** it cites D6 and follows the confirmed decision, or, while D6 is TO BE CONFIRMED, follows the proposed default and says so
- **AND** the proposed default is: Off by default, opt-in crash reports
- **AND** the deadline is: Before the public alpha

### Requirement: Open question Q-00-01 — Official product name (Virtax is a codename)?
Open question Q-00-01 (BRD-00) SHALL be answered by its owner (Sponsor) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-00-01
- **WHEN** a change implements behaviour covered by Q-00-01
- **THEN** it follows the confirmed answer, or, while Q-00-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Keep Virtax until beta; trademark check before GA.
- **AND** the Vietnamese source reads "Tên sản phẩm chính thức (Virtax chỉ là tên mã)?" with default "Giữ Virtax tới beta; kiểm tra trademark trước GA."

### Requirement: Open question Q-00-02 — Phase-1 engine distribution: BYO VirtualBox or a self-built Engine Pack?
Open question Q-00-02 (BRD-00) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-00-02
- **WHEN** a change implements behaviour covered by Q-00-02
- **THEN** it follows the confirmed answer, or, while Q-00-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Alpha = BYO; Beta = Engine Pack (driverless on macOS/Linux/Win-arm64).
- **AND** the Vietnamese source reads "Phân phối engine Pha 1: BYO VirtualBox hay Engine Pack tự build?" with default "Alpha = BYO; Beta = Engine Pack (driverless trên macOS/Linux/Win-arm64)."

### Requirement: Open question Q-00-03 — Must Windows Home without Hyper-V be supported (HM only)?
Open question Q-00-03 (BRD-00) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-00-03
- **WHEN** a change implements behaviour covered by Q-00-03
- **THEN** it follows the confirmed answer, or, while Q-00-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Yes in Phase 1 (via the VirtualBox engine); Phase 2 uses WHP (available on Home via Virtual Machine Platform).
- **AND** the Vietnamese source reads "Có cần hỗ trợ Windows Home không có Hyper-V (chỉ HM)?" with default "Có ở Pha 1 (qua engine VirtualBox); Pha 2 dùng WHP (có trên Home qua Virtual Machine Platform)."

### Requirement: Open question Q-01-01 — First run: follow the OS theme or always Dark?
Open question Q-01-01 (BRD-01) SHALL be answered by its owner (UX lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-01-01
- **WHEN** a change implements behaviour covered by Q-01-01
- **THEN** it follows the confirmed answer, or, while Q-01-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Follow the OS when it declares a preference; otherwise Dark (dark-first tokens).
- **AND** the Vietnamese source reads "Lần chạy đầu: theo theme của OS hay luôn Dark?" with default "Theo OS nếu OS có khai báo; nếu không thì Dark (token dark-first)."

### Requirement: Open question Q-01-02 — Show the equivalent `virtaxctl` command in the palette and Review diff by default?
Open question Q-01-02 (BRD-01) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-01-02
- **WHEN** a change implements behaviour covered by Q-01-02
- **THEN** it follows the confirmed answer, or, while Q-01-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Yes, switchable in Preferences (supports US-00-03).
- **AND** the Vietnamese source reads "Có hiện lệnh `virtaxctl` tương đương trong palette và Review diff theo mặc định không?" with default "Có, tắt được trong Preferences (hỗ trợ US-00-03)."

### Requirement: Open question Q-01-03 — Should legacy-VM tags live only in the registry (lost when the VM folder moves to another machine) or also in a sidecar file in the VM folder?
Open question Q-01-03 (BRD-01) SHALL be answered by its owner (Backend lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-01-03
- **WHEN** a change implements behaviour covered by Q-01-03
- **THEN** it follows the confirmed answer, or, while Q-01-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Registry in P1, included in `virtaxctl vm export`; an `virtax.meta.toml` sidecar considered in P2.
- **AND** the Vietnamese source reads "Tag của VM legacy chỉ nằm trong registry (mất khi chuyển thư mục VM sang máy khác) hay thêm file sidecar trong thư mục VM?" with default "Registry ở P1, xuất kèm `virtaxctl vm export`; sidecar `virtax.meta.toml` xem xét ở P2."

### Requirement: Open question Q-01-04 — Multi-window scope in P1: every tab or only Logs/Performance?
Open question Q-01-04 (BRD-01) SHALL be answered by its owner (UI lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-01-04
- **WHEN** a change implements behaviour covered by Q-01-04
- **THEN** it follows the confirmed answer, or, while Q-01-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: P1: Logs, Performance and a second Manager window; other tabs in P2.
- **AND** the Vietnamese source reads "Phạm vi đa cửa sổ ở P1: mọi tab hay chỉ Logs/Performance?" with default "P1: Logs, Performance và cửa sổ Manager thứ hai; các tab khác ở P2."

### Requirement: Open question Q-02-01 — Should "Install in background" (no Console during install) default to on or off?
Open question Q-02-01 (BRD-02) SHALL be answered by its owner (UX lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-02-01
- **WHEN** a change implements behaviour covered by Q-02-01
- **THEN** it follows the confirmed answer, or, while Q-02-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Off for desktop ISOs (users like to watch), on for server ISOs.
- **AND** the Vietnamese source reads "Mặc định "Install in background" (không mở Console khi cài) bật hay tắt?" with default "Tắt cho ISO desktop (người dùng muốn xem), bật cho ISO server."

### Requirement: Open question Q-02-02 — Should Phase 1 install Guest Additions when a BYO engine ships no Additions ISO?
Open question Q-02-02 (BRD-02) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-02-02
- **WHEN** a change implements behaviour covered by Q-02-02
- **THEN** it follows the confirmed answer, or, while Q-02-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Yes when the engine's Additions ISO is found; otherwise turn the toggle off and explain why.
- **AND** the Vietnamese source reads "Pha 1 có tự cài Guest Additions khi BYO engine không kèm ISO Additions không?" with default "Có nếu tìm thấy ISO Additions của engine; nếu không thì tắt toggle và giải thích."

### Requirement: Open question Q-02-03 — Default export format: OVF 1.0 (widest compatibility) or OVF 2.0?
Open question Q-02-03 (BRD-02) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-02-03
- **WHEN** a change implements behaviour covered by Q-02-03
- **THEN** it follows the confirmed answer, or, while Q-02-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: OVF 1.0 with a SHA-256 manifest; OVF 2.0 one click away.
- **AND** the Vietnamese source reads "Định dạng export mặc định: OVF 1.0 (tương thích rộng) hay OVF 2.0?" with default "OVF 1.0 với manifest SHA-256; OVF 2.0 chọn được một click."

### Requirement: Open question Q-02-04 — Should ISOs/cloud images be downloadable straight from the catalogue?
Open question Q-02-04 (BRD-02) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-02-04
- **WHEN** a change implements behaviour covered by Q-02-04
- **THEN** it follows the confirmed answer, or, while Q-02-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: P1 only opens the official download page; P2 downloads with SHA-256 verification after confirmation.
- **AND** the Vietnamese source reads "Có cho tải ISO/cloud image trực tiếp từ catalog không?" with default "P1 chỉ mở trang tải chính thức; P2 tải có kiểm tra SHA-256 sau khi xác nhận."

### Requirement: Open question Q-02-05 — Are linked clones allowed across engines (legacy source, virtax-vmm clone)?
Open question Q-02-05 (BRD-02) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-02-05
- **WHEN** a change implements behaviour covered by Q-02-05
- **THEN** it follows the confirmed answer, or, while Q-02-05 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No; cross-engine clones are always full clones.
- **AND** the Vietnamese source reads "Có cho linked clone giữa hai engine (nguồn legacy, clone chạy virtax-vmm) không?" with default "Không; clone khác engine luôn là full clone."

### Requirement: Open question Q-03-01 — When Apply needs power-off on a running VM: ask or queue by default?
Open question Q-03-01 (BRD-03) SHALL be answered by its owner (UX lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-03-01
- **WHEN** a change implements behaviour covered by Q-03-01
- **THEN** it follows the confirmed answer, or, while Q-03-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Ask each time, with "Remember for this VM".
- **AND** the Vietnamese source reads "Khi Apply cần tắt máy trên VM đang chạy: mặc định hỏi hay tự xếp hàng?" with default "Hỏi mỗi lần, có "Nhớ cho VM này"."

### Requirement: Open question Q-03-02 — Should Virtax ever write `.vbox` directly (without VBoxSVC) for legacy VMs?
Open question Q-03-02 (BRD-03) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-03-02
- **WHEN** a change implements behaviour covered by Q-03-02
- **THEN** it follows the confirmed answer, or, while Q-03-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No in P1-P2; only through VBoxCAPI. The Rust reader is read-only.
- **AND** the Vietnamese source reads "Virtax có bao giờ ghi `.vbox` trực tiếp (không qua VBoxSVC) cho VM legacy không?" with default "Không ở P1-P2; chỉ qua VBoxCAPI. Bộ đọc Rust chỉ đọc."

### Requirement: Open question Q-03-03 — Disk paths in `machine.toml`: relative only, or absolute allowed?
Open question Q-03-03 (BRD-03) SHALL be answered by its owner (Backend lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-03-03
- **WHEN** a change implements behaviour covered by Q-03-03
- **THEN** it follows the confirmed answer, or, while Q-03-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Relative to the VM folder; absolute allowed for disks outside it, with a warning on export/move.
- **AND** the Vietnamese source reads "Đường dẫn disk trong `machine.toml`: chỉ tương đối hay cho phép tuyệt đối?" with default "Tương đối theo thư mục VM; tuyệt đối được phép cho disk ngoài thư mục, có cảnh báo khi export/move."

### Requirement: Open question Q-03-04 — How are manual `machine.toml` edits handled while the VM runs?
Open question Q-03-04 (BRD-03) SHALL be answered by its owner (Backend lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-03-04
- **WHEN** a change implements behaviour covered by Q-03-04
- **THEN** it follows the confirmed answer, or, while Q-03-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: The file watcher detects and validates; valid → becomes a pending change with a notice; invalid → rejected, file untouched, line/column reported.
- **AND** the Vietnamese source reads "Sửa tay `machine.toml` khi VM đang chạy xử lý thế nào?" with default "File watcher phát hiện, validate; hợp lệ → thành pending change kèm thông báo; không hợp lệ → từ chối, file giữ nguyên, báo dòng/cột."

### Requirement: Open question Q-04-01 — One Console process per VM, or one process for all console windows?
Open question Q-04-01 (BRD-04) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-04-01
- **WHEN** a change implements behaviour covered by Q-04-01
- **THEN** it follows the confirmed answer, or, while Q-04-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: One process per VM in P1 (fault isolation, simpler capture); revisit after RSS measurements.
- **AND** the Vietnamese source reads "Mỗi VM một process Console hay một process cho mọi cửa sổ console?" with default "Mỗi VM một process ở P1 (cô lập lỗi, capture đơn giản); xem lại khi đo RSS."

### Requirement: Open question Q-04-02 — Should keyboard auto-capture on console focus be on by default?
Open question Q-04-02 (BRD-04) SHALL be answered by its owner (UX lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-04-02
- **WHEN** a change implements behaviour covered by Q-04-02
- **THEN** it follows the confirmed answer, or, while Q-04-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: On (as in VirtualBox) but always with the chip and immediate release on focus loss; switchable in Preferences › Input.
- **AND** the Vietnamese source reads "Auto-capture bàn phím khi cửa sổ console được focus có bật mặc định không?" with default "Bật (như VirtualBox) nhưng luôn có chip và thả ngay khi mất focus; tắt được trong Preferences › Input."

### Requirement: Open question Q-04-03 — Default present mode: FIFO (power-saving) or Mailbox (low latency)?
Open question Q-04-03 (BRD-04) SHALL be answered by its owner (Console/graphics engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-04-03
- **WHEN** a change implements behaviour covered by Q-04-03
- **THEN** it follows the confirmed answer, or, while Q-04-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: FIFO by default; Mailbox with "Low latency" on and when on AC power.
- **AND** the Vietnamese source reads "Present mode mặc định: FIFO (tiết kiệm điện) hay Mailbox (trễ thấp)?" with default "FIFO mặc định; Mailbox khi bật "Low latency" và khi cắm sạc."

### Requirement: Open question Q-04-04 — Default macOS host key: keep left ⌘ like VirtualBox, or switch to right ⌘?
Open question Q-04-04 (BRD-04) SHALL be answered by its owner (UX lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-04-04
- **WHEN** a change implements behaviour covered by Q-04-04
- **THEN** it follows the confirmed answer, or, while Q-04-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Keep the VirtualBox default for migrating users; onboarding lets users change it with one key press.
- **AND** the Vietnamese source reads "Host key mặc định trên macOS: giữ ⌘ trái như VirtualBox hay đổi sang ⌘ phải?" with default "Giữ mặc định VirtualBox cho người chuyển sang; onboarding cho đổi bằng một phím bấm."

### Requirement: Open question Q-04-05 — Is recording in P1?
Open question Q-04-05 (BRD-04) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-04-05
- **WHEN** a change implements behaviour covered by Q-04-05
- **THEN** it follows the confirmed answer, or, while Q-04-05 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No; P1 only has `FrameTap` and screenshots, an OS-encoder recorder comes in P2 (per RSK-00-01).
- **AND** the Vietnamese source reads "Recording có vào P1 không?" with default "Không; P1 chỉ có `FrameTap` và screenshot, recorder dùng encoder OS ở P2 (theo RSK-00-01)."

### Requirement: Open question Q-05-01 — What should closing the Console window do by default?
Open question Q-05-01 (BRD-05) SHALL be answered by its owner (UX lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-05-01
- **WHEN** a change implements behaviour covered by Q-05-01
- **THEN** it follows the confirmed answer, or, while Q-05-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Keep running in the background (chip in the Manager) with a one-time tip.
- **AND** the Vietnamese source reads "Đóng cửa sổ Console mặc định làm gì?" with default "Chạy tiếp ở nền (VM có chip trong Manager) kèm gợi ý một lần."

### Requirement: Open question Q-05-02 — Default ACPI timeout and escalation policy?
Open question Q-05-02 (BRD-05) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-05-02
- **WHEN** a change implements behaviour covered by Q-05-02
- **THEN** it follows the confirmed answer, or, while Q-05-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: 90 s then ask (interactive); 90 s then save (autostop).
- **AND** the Vietnamese source reads "Timeout ACPI và chính sách leo thang mặc định?" with default "90 s rồi hỏi (interactive); 90 s rồi save (autostop)."

### Requirement: Open question Q-05-03 — On host sleep: pause or save VMs?
Open question Q-05-03 (BRD-05) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-05-03
- **WHEN** a change implements behaviour covered by Q-05-03
- **THEN** it follows the confirmed answer, or, while Q-05-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Pause (fast); save when on battery < 10% or hibernating.
- **AND** the Vietnamese source reads "Khi host ngủ: pause hay save VM?" with default "Pause (nhanh); save nếu chạy pin < 10% hoặc ngủ đông (hibernate)."

### Requirement: Open question Q-05-04 — Should snapshots of running VMs default to live or paused?
Open question Q-05-04 (BRD-05) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-05-04
- **WHEN** a change implements behaviour covered by Q-05-04
- **THEN** it follows the confirmed answer, or, while Q-05-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Live (no interruption); paused is an Advanced option.
- **AND** the Vietnamese source reads "Snapshot mặc định live hay paused khi VM đang chạy?" with default "Live (không gián đoạn); paused là tuỳ chọn trong Advanced."

### Requirement: Open question Q-06-01 — Default disk format for new virtax-vmm VMs: qcow2 or sparse raw?
Open question Q-06-01 (BRD-06) SHALL be answered by its owner (Storage engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-06-01
- **WHEN** a change implements behaviour covered by Q-06-01
- **THEN** it follows the confirmed answer, or, while Q-06-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: qcow2 (overlays, snapshots, discard); raw for the 'Max performance' option and CI base images.
- **AND** the Vietnamese source reads "Định dạng disk mặc định cho VM virtax-vmm mới: qcow2 hay raw sparse?" with default "qcow2 (overlay, snapshot, discard); raw cho lựa chọn 'Max performance' và base image CI."

### Requirement: Open question Q-06-02 — On 'Move to virtax-vmm', convert VDI to qcow2 right away?
Open question Q-06-02 (BRD-06) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-06-02
- **WHEN** a change implements behaviour covered by Q-06-02
- **THEN** it follows the confirmed answer, or, while Q-06-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No; virtax-vmm reads/writes VDI in place for easy rollback; suggest a background convert after 7 stable days.
- **AND** the Vietnamese source reads "Khi 'Move to virtax-vmm', có convert VDI sang qcow2 ngay không?" with default "Không; virtax-vmm đọc/ghi VDI tại chỗ để rollback dễ; đề xuất convert nền sau 7 ngày chạy ổn."

### Requirement: Open question Q-06-03 — macOS: should guest FLUSH map to F_FULLFSYNC or F_BARRIERFSYNC by default?
Open question Q-06-03 (BRD-06) SHALL be answered by its owner (Storage engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-06-03
- **WHEN** a change implements behaviour covered by Q-06-03
- **THEN** it follows the confirmed answer, or, while Q-06-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: F_FULLFSYNC (truly durable); F_BARRIERFSYNC as an explained option.
- **AND** the Vietnamese source reads "macOS: guest FLUSH mặc định map sang F_FULLFSYNC hay F_BARRIERFSYNC?" with default "F_FULLFSYNC (bền vững thật); F_BARRIERFSYNC là tuỳ chọn có giải thích."

### Requirement: Open question Q-06-04 — Should the ISO library scan ~/Downloads automatically?
Open question Q-06-04 (BRD-06) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-06-04
- **WHEN** a change implements behaviour covered by Q-06-04
- **THEN** it follows the confirmed answer, or, while Q-06-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No scanning by default (privacy); ask once on first use 'Add Downloads to the library?'.
- **AND** the Vietnamese source reads "ISO library có tự quét ~/Downloads không?" with default "Không quét mặc định (quyền riêng tư); lần đầu hỏi một lần 'Thêm Downloads vào thư viện?'."

### Requirement: Open question Q-06-05 — Should the Phase-1 Plugins screen guide users to install Oracle's extension pack?
Open question Q-06-05 (BRD-06) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-06-05
- **WHEN** a change implements behaviour covered by Q-06-05
- **THEN** it follows the confirmed answer, or, while Q-06-05 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No; only show packs already installed in the legacy engine, with a neutral docs link; Virtax does not distribute PUEL packs.
- **AND** the Vietnamese source reads "Màn hình Plugins Pha 1 có hướng dẫn cài extension pack của Oracle không?" with default "Không; chỉ hiện gói đã cài trong engine legacy, có link tài liệu trung lập; Virtax không phân phối gói PUEL."

### Requirement: Open question Q-07-01 — Default Linux NAT backend in Phase 2: passt or libslirp?
Open question Q-07-01 (BRD-07) SHALL be answered by its owner (Network engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-07-01
- **WHEN** a change implements behaviour covered by Q-07-01
- **THEN** it follows the confirmed answer, or, while Q-07-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: passt when it meets the minimum version, otherwise libslirp; final call based on iperf3 benchmarks + IPv6 tests.
- **AND** the Vietnamese source reads "Backend NAT mặc định trên Linux ở Pha 2: passt hay libslirp?" with default "passt nếu đạt version tối thiểu, ngược lại libslirp; quyết định cuối dựa trên benchmark iperf3 + test IPv6."

### Requirement: Open question Q-07-02 — Windows bridged for virtax-vmm: HNS, our own signed LWF, or legacy engine only?
Open question Q-07-02 (BRD-07) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-07-02
- **WHEN** a change implements behaviour covered by Q-07-02
- **THEN** it follows the confirmed answer, or, while Q-07-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Legacy engine only until the HNS spike succeeds; no own driver shipped in Phase 2.
- **AND** the Vietnamese source reads "Bridged trên Windows cho virtax-vmm: HNS, LWF ký riêng, hay chỉ engine legacy?" with default "Chỉ engine legacy cho tới khi spike HNS thành công; không ship driver riêng ở Pha 2."

### Requirement: Open question Q-07-03 — Allow legacy and virtax-vmm VMs on the same NAT/internal network?
Open question Q-07-03 (BRD-07) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-07-03
- **WHEN** a change implements behaviour covered by Q-07-03
- **THEN** it follows the confirmed answer, or, while Q-07-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Not in Phase 2 (blocked in the UI with an explanation); revisit in Phase 3 when legacy becomes optional.
- **AND** the Vietnamese source reads "Cho phép VM legacy và virtax-vmm cùng một NAT/internal network không?" with default "Không ở Pha 2 (UI chặn kèm giải thích); Pha 3 xem lại khi legacy thành gói tuỳ chọn."

### Requirement: Open question Q-07-04 — Should new port forwards bind 127.0.0.1 by default, or all interfaces like VirtualBox?
Open question Q-07-04 (BRD-07) SHALL be answered by its owner (Security champion) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-07-04
- **WHEN** a change implements behaviour covered by Q-07-04
- **THEN** it follows the confirmed answer, or, while Q-07-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: 127.0.0.1; rules imported from .vbox keep their old value but show an 'Exposed to LAN' badge.
- **AND** the Vietnamese source reads "Port forward mới mặc định bind 127.0.0.1 hay mọi interface như VirtualBox?" with default "127.0.0.1; rule import từ .vbox giữ nguyên giá trị cũ nhưng hiện badge 'Mở ra LAN'."

### Requirement: Open question Q-07-05 — MAC prefix for new VMs: keep the 08:00:27 OUI for tool compatibility or use locally administered addresses?
Open question Q-07-05 (BRD-07) SHALL be answered by its owner (Network engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-07-05
- **WHEN** a change implements behaviour covered by Q-07-05
- **THEN** it follows the confirmed answer, or, while Q-07-05 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Random locally administered (bit 0x02) with a collision check; imported VMs keep their MACs.
- **AND** the Vietnamese source reads "Tiền tố MAC cho VM mới: giữ OUI 08:00:27 để tương thích tool cũ hay dùng địa chỉ locally administered?" with default "Locally administered (bit 0x02) ngẫu nhiên có kiểm trùng; VM import giữ MAC cũ."

### Requirement: Open question Q-08-01 — Agent protocol: our own protobuf or qemu-guest-agent compatible (JSON-RPC)?
Open question Q-08-01 (BRD-08) SHALL be answered by its owner (Integration engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-08-01
- **WHEN** a change implements behaviour covered by Q-08-01
- **THEN** it follows the confirmed answer, or, while Q-08-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Our own versioned protobuf (prost) with capabilities; no qemu-ga compatibility in Phase 2.
- **AND** the Vietnamese source reads "Protocol agent: protobuf riêng hay tương thích qemu-guest-agent (JSON-RPC)?" with default "Protobuf riêng (prost) có version và capability; không tương thích qemu-ga ở Pha 2."

### Requirement: Open question Q-08-02 — Allow exec without guest credentials?
Open question Q-08-02 (BRD-08) SHALL be answered by its owner (Security champion) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-08-02
- **WHEN** a change implements behaviour covered by Q-08-02
- **THEN** it follows the confirmed answer, or, while Q-08-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Yes for Virtax-created VMs with an Virtax-installed agent; imported VMs must opt in; always audited.
- **AND** the Vietnamese source reads "Cho phép exec không cần mật khẩu guest không?" with default "Có với VM do Virtax tạo và agent do Virtax cài; VM import phải bật tay; luôn có audit log."

### Requirement: Open question Q-08-03 — Bundle virtio-win in the installer or download on demand?
Open question Q-08-03 (BRD-08) SHALL be answered by its owner (Release engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-08-03
- **WHEN** a change implements behaviour covered by Q-08-03
- **THEN** it follows the confirmed answer, or, while Q-08-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Download on demand, pinned version + SHA-256, cached in the ISO library.
- **AND** the Vietnamese source reads "Đóng gói virtio-win trong installer hay tải khi cần?" with default "Tải khi cần, pin version + SHA-256, cache trong ISO library."

### Requirement: Open question Q-08-04 — Should virtax-vmm emulate VMMDev to keep Guest Additions working?
Open question Q-08-04 (BRD-08) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-08-04
- **WHEN** a change implements behaviour covered by Q-08-04
- **THEN** it follows the confirmed answer, or, while Q-08-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No; 'Move to virtax-vmm' installs the agent first and rollback keeps GA on legacy.
- **AND** the Vietnamese source reads "virtax-vmm có giả lập VMMDev để giữ Guest Additions chạy được không?" with default "Không; 'Move to virtax-vmm' cài agent trước, rollback giữ GA trên legacy."

### Requirement: Open question Q-08-05 — Default clipboard for new VMs: bidirectional?
Open question Q-08-05 (BRD-08) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-08-05
- **WHEN** a change implements behaviour covered by Q-08-05
- **THEN** it follows the confirmed answer, or, while Q-08-05 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Bidirectional text + image for Virtax-created VMs, files off; imported VMs keep their value.
- **AND** the Vietnamese source reads "Mặc định clipboard cho VM mới: hai chiều?" with default "Hai chiều text + ảnh cho VM do Virtax tạo, file tắt; VM import giữ giá trị cũ."

### Requirement: Open question Q-09-01 — Should the daemon exit after a long idle period?
Open question Q-09-01 (BRD-09) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-09-01
- **WHEN** a change implements behaviour covered by Q-09-01
- **THEN** it follows the confirmed answer, or, while Q-09-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Desktop: stay while the user is logged in (≈ zero cost); server: socket activation, exit after 10 minutes with no VMs/clients.
- **AND** the Vietnamese source reads "Daemon có tự thoát khi rảnh lâu không?" with default "Desktop: ở lại khi user đăng nhập (chi phí ≈ 0); server: socket activation, thoát sau 10 phút không VM/không client."

### Requirement: Open question Q-09-02 — Ship the Python SDK already in P1?
Open question Q-09-02 (BRD-09) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-09-02
- **WHEN** a change implements behaviour covered by Q-09-02
- **THEN** it follows the confirmed answer, or, while Q-09-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Yes at beta (generated), marked preview until v1 stabilises.
- **AND** the Vietnamese source reads "Có phát hành SDK Python ngay P1 không?" with default "Có ở beta (sinh tự động), đánh dấu preview tới khi v1 ổn định."

### Requirement: Open question Q-09-03 — Default port and enablement for the Prometheus exporter?
Open question Q-09-03 (BRD-09) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-09-03
- **WHEN** a change implements behaviour covered by Q-09-03
- **THEN** it follows the confirmed answer, or, while Q-09-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Off by default; enabled via `virtaxctl metrics serve` or config, 127.0.0.1 only.
- **AND** the Vietnamese source reads "Cổng mặc định và cách bật Prometheus exporter?" with default "Tắt mặc định; bật bằng `virtaxctl metrics serve` hoặc cấu hình, chỉ 127.0.0.1."

### Requirement: Open question Q-09-04 — Where should the crash-report endpoint live?
Open question Q-09-04 (BRD-09) SHALL be answered by its owner (Sponsor) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-09-04
- **WHEN** a change implements behaviour covered by Q-09-04
- **THEN** it follows the confirmed answer, or, while Q-09-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: A project-hosted open-source server with 90-day retention; only after opt-in.
- **AND** the Vietnamese source reads "Endpoint nhận crash report đặt ở đâu?" with default "Server tự host của dự án (open source), giữ 90 ngày; chỉ sau opt-in."

### Requirement: Open question Q-10-01 — Depend on OpenVMM crates (git rev) or fork the whole tree?
Open question Q-10-01 (BRD-10) SHALL be answered by its owner (VMM lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-10-01
- **WHEN** a change implements behaviour covered by Q-10-01
- **THEN** it follows the confirmed answer, or, while Q-10-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Depend on pinned git revs; fork only the crates that need patches and upstream them.
- **AND** the Vietnamese source reads "Phụ thuộc crate OpenVMM (git rev) hay fork toàn bộ?" with default "Phụ thuộc theo git rev đã pin; chỉ fork crate cần patch và gửi upstream."

### Requirement: Open question Q-10-02 — Support `virtax-vmm` on macOS 14 (userspace GIC)?
Open question Q-10-02 (BRD-10) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-10-02
- **WHEN** a change implements behaviour covered by Q-10-02
- **THEN** it follows the confirmed answer, or, while Q-10-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Not in Phase 2; macOS 14 uses the legacy engine.
- **AND** the Vietnamese source reads "Có hỗ trợ `virtax-vmm` trên macOS 14 (GIC userspace) không?" with default "Không ở Pha 2; macOS 14 dùng engine legacy."

### Requirement: Open question Q-10-03 — Windows-guest 3D: port VMSVGA to Rust or wait for virtio-gpu/GPU-PV options?
Open question Q-10-03 (BRD-10) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-10-03
- **WHEN** a change implements behaviour covered by Q-10-03
- **THEN** it follows the confirmed answer, or, while Q-10-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No port in Phase 2; keep the legacy engine; reassess in Phase 3 with VFIO/GPU-PV.
- **AND** the Vietnamese source reads "3D cho guest Windows: port VMSVGA sang Rust hay chờ giải pháp virtio-gpu/GPU-PV?" with default "Không port ở Pha 2; giữ engine legacy; đánh giá lại ở Pha 3 cùng VFIO/GPU-PV."

### Requirement: Open question Q-10-04 — Saved-state format: protobuf or serde (postcard)?
Open question Q-10-04 (BRD-10) SHALL be answered by its owner (VMM lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-10-04
- **WHEN** a change implements behaviour covered by Q-10-04
- **THEN** it follows the confirmed answer, or, while Q-10-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: postcard + per-unit version tags, with a JSON dump command for debugging.
- **AND** the Vietnamese source reads "Định dạng saved state: protobuf hay serde (postcard)?" with default "postcard + tag version cho từng unit, có lệnh dump JSON để debug."

### Requirement: Open question Q-10-05 — Should the Engine Pack ship HM (attestation-signed VBoxSup) for Windows x64 without Hyper-V?
Open question Q-10-05 (BRD-10) SHALL be answered by its owner (Release engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-10-05
- **WHEN** a change implements behaviour covered by Q-10-05
- **THEN** it follows the confirmed answer, or, while Q-10-05 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: No; such machines use BYO official VirtualBox.
- **AND** the Vietnamese source reads "Engine Pack có ship HM (VBoxSup ký attestation) cho Windows x64 không bật Hyper-V không?" with default "Không; máy đó dùng BYO VirtualBox chính chủ."

### Requirement: Open question Q-11-01 — Signature scheme: minisign only, or add Sigstore keyless?
Open question Q-11-01 (BRD-11) SHALL be answered by its owner (Release engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-11-01
- **WHEN** a change implements behaviour covered by Q-11-01
- **THEN** it follows the confirmed answer, or, while Q-11-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: minisign (offline root) for manifests/plugins + GitHub artifact attestations for provenance.
- **AND** the Vietnamese source reads "Lược đồ chữ ký: chỉ minisign hay thêm Sigstore keyless?" with default "minisign (root offline) cho manifest/plugin + GitHub artifact attestation cho provenance."

### Requirement: Open question Q-11-02 — Secure Boot PK: generated per VM or one project PK?
Open question Q-11-02 (BRD-11) SHALL be answered by its owner (Security champion) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-11-02
- **WHEN** a change implements behaviour covered by Q-11-02
- **THEN** it follows the confirmed answer, or, while Q-11-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Per VM, with the private key discarded after enrollment (no one can sign new KEKs except via reset).
- **AND** the Vietnamese source reads "PK Secure Boot: sinh riêng mỗi VM hay một PK của dự án?" with default "Sinh riêng mỗi VM, private key bỏ sau khi enroll (không ai ký được KEK mới ngoài reset)."

### Requirement: Open question Q-11-03 — Timing and scope of the external security audit?
Open question Q-11-03 (BRD-11) SHALL be answered by its owner (Sponsor) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-11-03
- **WHEN** a change implements behaviour covered by Q-11-03
- **THEN** it follows the confirmed answer, or, while Q-11-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Helper + IPC + updater, two months after Beta, before GA.
- **AND** the Vietnamese source reads "Thời điểm và phạm vi audit bảo mật bên ngoài?" with default "Helper + IPC + updater, 2 tháng sau Beta, trước GA."

### Requirement: Open question Q-11-04 — Install updates automatically by default?
Open question Q-11-04 (BRD-11) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-11-04
- **WHEN** a change implements behaviour covered by Q-11-04
- **THEN** it follows the confirmed answer, or, while Q-11-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Auto-check and auto-download on stable; install on user click or app restart; security releases get a badge.
- **AND** the Vietnamese source reads "Mặc định tự động cài cập nhật?" with default "Tự kiểm và tự tải trên stable; cài khi người dùng bấm hoặc khi app restart; bản security có badge."

### Requirement: Open question Q-11-05 — Plugin distribution: is there a central index?
Open question Q-11-05 (BRD-11) SHALL be answered by its owner (Backend lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-11-05
- **WHEN** a change implements behaviour covered by Q-11-05
- **THEN** it follows the confirmed answer, or, while Q-11-05 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: A curated, signed GitHub-hosted index; side-loading allowed with a warning.
- **AND** the Vietnamese source reads "Phân phối plugin: có index trung tâm không?" with default "Index tuyển chọn trên GitHub có chữ ký; cho side-load kèm cảnh báo."

### Requirement: Open question Q-12-01 — macOS: one universal2 build or two per-arch builds?
Open question Q-12-01 (BRD-12) SHALL be answered by its owner (Release engineer) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-12-01
- **WHEN** a change implements behaviour covered by Q-12-01
- **THEN** it follows the confirmed answer, or, while Q-12-01 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: universal2 for Manager/daemon (x64 best-effort); the Engine Pack split per arch.
- **AND** the Vietnamese source reads "macOS: một bản universal2 hay hai bản theo arch?" with default "universal2 cho Manager/daemon (x64 best-effort); Engine Pack tách theo arch."

### Requirement: Open question Q-12-02 — Windows signing: self-managed OV/EV or Azure Trusted Signing?
Open question Q-12-02 (BRD-12) SHALL be answered by its owner (Sponsor) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-12-02
- **WHEN** a change implements behaviour covered by Q-12-02
- **THEN** it follows the confirmed answer, or, while Q-12-02 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Azure Trusted Signing for EXE/MSI/MSIX; buy EV only if VBoxSup will ship.
- **AND** the Vietnamese source reads "Ký Windows: OV/EV tự quản hay Azure Trusted Signing?" with default "Azure Trusted Signing cho EXE/MSI/MSIX; chỉ mua EV nếu quyết định ship VBoxSup."

### Requirement: Open question Q-12-03 — Support Snap?
Open question Q-12-03 (BRD-12) SHALL be answered by its owner (Product owner) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-12-03
- **WHEN** a change implements behaviour covered by Q-12-03
- **THEN** it follows the confirmed answer, or, while Q-12-03 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Not in the MVP; Flatpak + AppImage + .deb/.rpm are enough.
- **AND** the Vietnamese source reads "Có hỗ trợ Snap không?" with default "Không ở MVP; Flatpak + AppImage + .deb/.rpm là đủ."

### Requirement: Open question Q-12-04 — Self-hosted HVF/WHP runners: buy or rent?
Open question Q-12-04 (BRD-12) SHALL be answered by its owner (Tech lead) before the behaviour it covers ships. Status: **TO BE CONFIRMED**.

#### Scenario: Work that depends on Q-12-04
- **WHEN** a change implements behaviour covered by Q-12-04
- **THEN** it follows the confirmed answer, or, while Q-12-04 is TO BE CONFIRMED, the proposed default, and says so in its proposal
- **AND** the proposed default (not confirmed) is: Buy 3-4 machines (~$4-6k, estimate) per vi-03; revisit when GitHub offers nested-virt runners.
- **AND** the Vietnamese source reads "Runner tự host cho HVF/WHP: tự mua hay thuê?" with default "Mua 3-4 máy (~$4-6k, ước tính) như vi-03; xem lại khi GitHub có runner nested-virt."
