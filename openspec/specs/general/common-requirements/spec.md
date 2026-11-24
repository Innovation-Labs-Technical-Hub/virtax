# Common requirements Specification

## Purpose
Requirements that apply to the whole product (BRD-00 §common requirements): bilingual UI, accessibility, privacy, platforms and other cross-cutting rules. Every other capability inherits these. Source: BRD-00. Implementation status (2026-11-24): see each requirement.

## Requirements

### Requirement: Bilingual VI/EN interface (FR-GEN-001)
Virtax SHALL provide "Bilingual VI/EN interface" as specified by FR-GEN-001 (BRD-00, priority Must, phase P1; Vietnamese: "Giao diện song ngữ VI/EN").

#### Scenario: Specified behaviour of FR-GEN-001
- **WHEN** the behaviour FR-GEN-001 covers is exercised
- **THEN** it behaves as BRD-00 specifies: All UI, error messages and CLI help exist in Vietnamese and English; follows the system locale and can be switched without restart.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/nls (40 .ts files)`

#### Scenario: Acceptance for FR-GEN-001
- **WHEN** the FR-GEN-001 acceptance check is run
- **THEN** Switching language in Preferences updates every screen within 1 s; 0 missing strings in the CI i18n report.

### Requirement: Supported platform matrix (FR-GEN-002)
Virtax SHALL provide "Supported platform matrix" as specified by FR-GEN-002 (BRD-00, priority Must, phase P1; Vietnamese: "Ma trận nền tảng được hỗ trợ").

#### Scenario: Specified behaviour of FR-GEN-002
- **WHEN** the behaviour FR-GEN-002 covers is exercised
- **THEN** it behaves as BRD-00 specifies: Virtax runs on 6 host combinations {Windows, macOS, Linux} × {x64, arm64}; macOS x64 is best-effort. The app detects the available hypervisor and explains clearly when one is missing.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/VMM/VMMR3/target-*/NEMR3Native-*.cpp`

#### Scenario: Acceptance for FR-GEN-002
- **WHEN** the FR-GEN-002 acceptance check is run
- **THEN** CI build + smoke tests green on all 6 combinations; a machine without VT-x/KVM shows how to enable virtualization.

### Requirement: Same-architecture guests only (FR-GEN-003)
Virtax SHALL provide "Same-architecture guests only" as specified by FR-GEN-003 (BRD-00, priority Must, phase P1; Vietnamese: "Chỉ guest cùng kiến trúc").

#### Scenario: Specified behaviour of FR-GEN-003
- **WHEN** the behaviour FR-GEN-003 covers is exercised
- **THEN** it behaves as BRD-00 specifies: Wizards and import only allow guests matching the host architecture; a mismatched ISO is blocked with an explanation and a hint to the right ISO.
- **AND** the VirtualBox reference for the current behaviour is `Config.kmk:593-598 (cross-arch hidden)`

#### Scenario: Acceptance for FR-GEN-003
- **WHEN** the FR-GEN-003 acceptance check is run
- **THEN** Dropping an amd64 ISO on an M-series Mac shows a message + link to the arm64 build; no VM is created.

### Requirement: Per-VM engine selection (FR-GEN-004)
Virtax SHALL provide "Per-VM engine selection" as specified by FR-GEN-004 (BRD-00, priority Must, phase P1; Vietnamese: "Chọn engine theo từng VM").

#### Scenario: Specified behaviour of FR-GEN-004
- **WHEN** the behaviour FR-GEN-004 covers is exercised
- **THEN** it behaves as BRD-00 specifies: Each VM has an engine attribute (vbox-legacy | virtax-vmm). Phase 2 offers "Move to virtax-vmm" with a device-compatibility check and keeps the .vbox for rollback.
- **AND** the VirtualBox reference for the current behaviour is `new (strangler switch)`

#### Scenario: Acceptance for FR-GEN-004
- **WHEN** the FR-GEN-004 acceptance check is run
- **THEN** Machine.engine exists in the API from P1; in P2 the conversion reports incompatible devices and rolls back in one click.

### Requirement: Coexistence with VirtualBox (FR-GEN-005)
Virtax SHALL provide "Coexistence with VirtualBox" as specified by FR-GEN-005 (BRD-00, priority Must, phase P1; Vietnamese: "Cùng tồn tại với VirtualBox").

#### Scenario: Specified behaviour of FR-GEN-005
- **WHEN** the behaviour FR-GEN-005 covers is exercised
- **THEN** it behaves as BRD-00 specifies: Virtax never breaks an installed VirtualBox; Phase 1 offers a BYO-engine mode using the official VirtualBox 7.2.x and never writes GUI/* keys into .vbox files.
- **AND** the VirtualBox reference for the current behaviour is `Frontends/VirtualBox/src/extradata/UIExtraDataDefs.cpp (151 GUI/* keys)`

#### Scenario: Acceptance for FR-GEN-005
- **WHEN** the FR-GEN-005 acceptance check is run
- **THEN** Opening the same VM in the VirtualBox GUI after Virtax works without errors or lost settings.

### Requirement: Every long action is a cancellable Operation with progress (FR-GEN-006)
Virtax SHALL provide "Every long action is a cancellable Operation with progress" as specified by FR-GEN-006 (BRD-00, priority Must, phase P1; Vietnamese: "Mọi thao tác dài là Operation có tiến độ & huỷ").

#### Scenario: Specified behaviour of FR-GEN-006
- **WHEN** the behaviour FR-GEN-006 covers is exercised
- **THEN** it behaves as BRD-00 specifies: Create, clone, import/export, snapshot, ISO detection, merge… are all Operations with %, ETA, cancel and history in the Task center; the UI never blocks.
- **AND** the VirtualBox reference for the current behaviour is `UnattendedImpl.cpp:575-625; UIWizardNewVM.cpp:628`

#### Scenario: Acceptance for FR-GEN-006
- **WHEN** the FR-GEN-006 acceptance check is run
- **THEN** No action holds the UI thread for > 50 ms (measured with tracing in CI e2e).

### Requirement: Human errors with fixes (FR-GEN-007)
Virtax SHALL provide "Human errors with fixes" as specified by FR-GEN-007 (BRD-00, priority Must, phase P1; Vietnamese: "Lỗi dễ hiểu có hướng dẫn sửa").

#### Scenario: Specified behaviour of FR-GEN-007
- **WHEN** the behaviour FR-GEN-007 covers is exercised
- **THEN** it behaves as BRD-00 specifies: Every error has an VTX-xxx code, a bilingual explanation, a quick-fix button when possible and "Copy technical details" with the original VBox/OS code.
- **AND** the VirtualBox reference for the current behaviour is `src/VBox/Frontends/VirtualBox/src/globals/UIErrorString.cpp; notificationcenter (208 messages)`

#### Scenario: Acceptance for FR-GEN-007
- **WHEN** the FR-GEN-007 acceptance check is run
- **THEN** The top 50 VBox errors (VERR_*) are mapped; UX test: 80% of users understand the cause.

### Requirement: No telemetry by default (FR-GEN-008)
Virtax SHALL provide "No telemetry by default" as specified by FR-GEN-008 (BRD-00, priority Must, phase P1; Vietnamese: "Không telemetry mặc định").

#### Scenario: Specified behaviour of FR-GEN-008
- **WHEN** the behaviour FR-GEN-008 covers is exercised
- **THEN** it behaves as BRD-00 specifies: Nothing leaves the machine until the user opts in; crash reports and anonymous metrics are optional and previewable.
- **AND** the VirtualBox reference for the current behaviour is `new`

#### Scenario: Acceptance for FR-GEN-008
- **WHEN** the FR-GEN-008 acceptance check is run
- **THEN** Egress check on a fresh install: only the update check (which can be disabled).

### Requirement: As built — Product identifiers
Virtax SHALL use the product name Virtax and the identifiers derived from it: crates `virtax-*`, binaries `virtaxd`, `virtaxctl` and `virtax-vmm`, protocol package `virtax.v1`, manifest schema `virtax.machine/1`, error codes `VTX-*` and environment variables `VIRTAX_*`. The design set's provisional codename "OxBox" (and `oxboxd`, `oxctl`, `oxvmm`, `OXE-*`) maps to these names (decision D5, confirmed 2026-11-24).

#### Scenario: Error codes use the VTX prefix
- **WHEN** the error catalogue is checked
- **THEN** every code starts with `VTX-` and has English and Vietnamese text
- **AND** this is verified by `crates/virtax-model/src/errors.rs::catalogue_codes_are_unique_and_bilingual`
