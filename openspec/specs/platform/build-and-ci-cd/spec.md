# Build & CI/CD Specification

## Purpose
Cargo workspace, 3 OS × 2 arch native runner matrix, cargo-deny licensing, real VM boot tests in CI. Module `PL-01` (Build & CI/CD; Vietnamese: "Build & CI/CD") from BRD-12, phase P1, priority Must. Owner crates: workspace (Cargo), xtask. Depends on: none. VirtualBox reference: `kBuild, Config.kmk (9.8k lines), configure.py, 408 .kmk files`. Implementation status (2026-11-24): partial — Cargo workspace, `cargo xtask ci|e2e|schema|guest-kernel|kvm-test`, CI on Linux x64/arm64, macOS and a KVM job; no release pipeline or performance gates yet.

## Requirements

### Requirement: Cargo workspace and crate map (FR-PL01-001)
Virtax SHALL provide "Cargo workspace and crate map" as specified by FR-PL01-001 (BRD-12, priority Must, phase P1; Vietnamese: "Cargo workspace và bản đồ crate").

#### Scenario: Specified behaviour of FR-PL01-001
- **WHEN** the behaviour FR-PL01-001 covers is exercised
- **THEN** it behaves as BRD-12 specifies: One Cargo workspace monorepo per diagram 06: binaries (virtax, virtax-console, virtaxctl, virtaxd, virtax-helper, virtax-agent, virtax-vmm), core crates (api, model, store, engine, ops, ui-kit, frame), engine drivers (vbox-sys, vbox-adapter, virtax-vmm-core, virtax-vmm-hv). `#![forbid(unsafe_code)]` outside `*-sys` and `virtax-vmm-hv`.
- **AND** the VirtualBox reference for the current behaviour is `Config.kmk (9,796 lines); 408 .kmk files`

#### Scenario: Acceptance for FR-PL01-001
- **WHEN** the FR-PL01-001 acceptance check is run
- **THEN** A clean clone needing only Rust + the platform toolchain (Xcode CLT / MSVC Build Tools) builds in ≤ 15 min, incrementally ≤ 2 min; CI checks `forbid(unsafe_code)`.

### Requirement: Automation with xtask (FR-PL01-002)
Virtax SHALL provide "Automation with xtask" as specified by FR-PL01-002 (BRD-12, priority Must, phase P1; Vietnamese: "Tự động hoá bằng xtask").

#### Scenario: Specified behaviour of FR-PL01-002
- **WHEN** the behaviour FR-PL01-002 covers is exercised
- **THEN** it behaves as BRD-12 specifies: `cargo xtask` provides `ci`, `codegen` (proto, XIDL), `fw fetch`, `engine-pack fetch`, `bench`, `dist`, `sign`, `release`; it behaves the same on PowerShell and Unix shells; every CI job calls exactly one xtask command.
- **AND** the VirtualBox reference for the current behaviour is `Makefile.kmk (2,411 lines); configure.py (4,386 lines)`

#### Scenario: Acceptance for FR-PL01-002
- **WHEN** the FR-PL01-002 acceptance check is run
- **THEN** `cargo xtask ci --quick` on a dev machine matches CI results; no Makefiles or shell scripts beyond thin wrappers.

### Requirement: Pinned toolchains and tools (FR-PL01-003)
Virtax SHALL provide "Pinned toolchains and tools" as specified by FR-PL01-003 (BRD-12, priority Must, phase P1; Vietnamese: "Pin toolchain và công cụ").

#### Scenario: Specified behaviour of FR-PL01-003
- **WHEN** the behaviour FR-PL01-003 covers is exercised
- **THEN** it behaves as BRD-12 specifies: `rust-toolchain.toml` pins a stable release (MSRV = N-2) and the target list; tools (cargo-deny, nextest, zigbuild, xwin, cyclonedx) are version-pinned via a lockfile; no external `protoc` install; `Cargo.lock` is committed.
- **AND** the VirtualBox reference for the current behaviour is `Config.kmk:1934-1936,2202,2231,2777`

#### Scenario: Acceptance for FR-PL01-003
- **WHEN** the FR-PL01-003 acceptance check is run
- **THEN** Two clean builds a month apart on the same commit use the same toolchain and lockfile; a toolchain bump is one PR with green CI.

### Requirement: CI matrix on native runners (FR-PL01-004)
Virtax SHALL provide "CI matrix on native runners" as specified by FR-PL01-004 (BRD-12, priority Must, phase P1; Vietnamese: "CI matrix trên runner native").

#### Scenario: Specified behaviour of FR-PL01-004
- **WHEN** the behaviour FR-PL01-004 covers is exercised
- **THEN** it behaves as BRD-12 specifies: GitHub Actions: windows-x64 (windows-2025), windows-arm64 (windows-11-arm), macos-arm64 (macos-15), ubuntu-x64 (24.04), ubuntu-arm64 (24.04-arm); macOS x64 cross-built, tested best-effort. Jobs: fmt, clippy `-D warnings`, cargo-deny, nextest + proptest + insta, build; sccache caching.
- **AND** the VirtualBox reference for the current behaviour is `.github/workflows/question-stale.yml (no build CI)`

#### Scenario: Acceptance for FR-PL01-004
- **WHEN** the FR-PL01-004 acceptance check is run
- **THEN** PR CI p90 ≤ 20 min; 5/5 runners are required checks; flaky-test rate ≤ 1% (measured weekly).

### Requirement: Cross-compilation where useful (FR-PL01-005)
Virtax SHALL provide "Cross-compilation where useful" as specified by FR-PL01-005 (BRD-12, priority Should, phase P1; Vietnamese: "Cross-compile khi có lợi").

#### Scenario: Specified behaviour of FR-PL01-005
- **WHEN** the behaviour FR-PL01-005 covers is exercised
- **THEN** it behaves as BRD-12 specifies: `cargo-zigbuild` builds Linux x64/arm64 against a glibc 2.28 baseline from one runner; `cargo-xwin` gives fast Windows check jobs on Linux; macOS universal2 via `lipo`. Release builds still build and test on native runners for signing.
- **AND** the VirtualBox reference for the current behaviour is `configure.py:2816-2818; Config.kmk:1310-1323`

#### Scenario: Acceptance for FR-PL01-005
- **WHEN** the FR-PL01-005 acceptance check is run
- **THEN** Linux binaries run on Debian 11/RHEL 8 (glibc 2.28) and Ubuntu 22.04/24.04; the `cargo xwin check` job takes ≤ 8 min.

### Requirement: Integration tests that boot real VMs (FR-PL01-006)
Virtax SHALL provide "Integration tests that boot real VMs" as specified by FR-PL01-006 (BRD-12, priority Must, phase P1; Vietnamese: "Test tích hợp boot VM thật").

#### Scenario: Specified behaviour of FR-PL01-006
- **WHEN** the behaviour FR-PL01-006 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Boot Alpine/Ubuntu cloud images headless with cloud-init: P1 on the legacy engine (VirtualBox 7.2 on the runner), P2 on `virtax-vmm` with KVM (Linux runners with `/dev/kvm`), HVF (self-hosted Mac mini) and WHP (self-hosted Windows). Scenario: create → start → guest ready → snapshot → restore → stop; nightly Windows 11 boot.
- **AND** the VirtualBox reference for the current behaviour is `new (ValidationKit not in public CI)`

#### Scenario: Acceptance for FR-PL01-006
- **WHEN** the FR-PL01-006 acceptance check is run
- **THEN** Per-PR smoke on ubuntu-x64 ≤ 10 min; nightly boot suite green on 6 combos (BG-01); artifacts keep serial logs and screenshots.

### Requirement: Performance regression gates (FR-PL01-007)
Virtax SHALL provide "Performance regression gates" as specified by FR-PL01-007 (BRD-12, priority Must, phase P1; Vietnamese: "Cổng hồi quy hiệu năng").

#### Scenario: Specified behaviour of FR-PL01-007
- **WHEN** the behaviour FR-PL01-007 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Budgets live in `perf-budgets.toml` and are measured nightly: idle CPU/wakeups of Manager and daemon, idle-VM CPU/wakeups (powertop/ETW/powermetrics), Console frame time p95, fio 4k QD32 and iperf3 (P2), criterion benchmarks. A budget breach or > 5% regression blocks the release.
- **AND** the VirtualBox reference for the current behaviour is `TMR3.cpp:688; SUPDrvGip.cpp:2105`

#### Scenario: Acceptance for FR-PL01-007
- **WHEN** the FR-PL01-007 acceptance check is run
- **THEN** Deliberately adding a 1 ms timer to the daemon turns the next nightly red; a per-commit dashboard; budgets come from NFR-00-xx.

### Requirement: Separate Engine Pack pipeline (FR-PL01-008)
Virtax SHALL provide "Separate Engine Pack pipeline" as specified by FR-PL01-008 (BRD-12, priority Should, phase P1; Vietnamese: "Pipeline Engine Pack tách riêng").

#### Scenario: Specified behaviour of FR-PL01-008
- **WHEN** the behaviour FR-PL01-008 covers is exercised
- **THEN** it behaves as BRD-12 specifies: A separate pipeline pins a VirtualBox 7.2.x tag + the Virtax patch queue (git am), builds with kBuild on native runners for 5 driverless combos, and emits signed tarballs + SBOM + hashes; the Cargo workspace only consumes them via `xtask engine-pack fetch` and never builds C++ in per-PR CI.
- **AND** the VirtualBox reference for the current behaviour is `Config.kmk:567-579; Installer/win/Makefile.kmk:312-327`

#### Scenario: Acceptance for FR-PL01-008
- **WHEN** the FR-PL01-008 acceptance check is run
- **THEN** An Engine Pack rebuild takes ≤ 4 h per combo; a new upstream 7.2.x → an Engine Pack candidate within 5 working days; Linux hashes reproduce.

### Requirement: Pinned prebuilt firmware and C components (FR-PL01-009)
Virtax SHALL provide "Pinned prebuilt firmware and C components" as specified by FR-PL01-009 (BRD-12, priority Should, phase P2; Vietnamese: "Firmware và thành phần C dùng bản prebuilt có pin").

#### Scenario: Specified behaviour of FR-PL01-009
- **WHEN** the behaviour FR-PL01-009 covers is exercised
- **THEN** it behaves as BRD-12 specifies: OVMF/AAVMF, libtpms, virglrenderer and MoltenVK are fetched at pinned versions with sha256 (or built in a separate pipeline), never compiled in per-PR CI — unlike VBox building 3M lines of edk2 in-tree.
- **AND** the VirtualBox reference for the current behaviour is `Devices/EFI/Firmware (3.03M LOC)`

#### Scenario: Acceptance for FR-PL01-009
- **WHEN** the FR-PL01-009 acceptance check is run
- **THEN** `xtask fw fetch` verifies hashes; changing firmware = one manifest line + a boot-suite run.

### Requirement: Contributor experience (FR-PL01-010)
Virtax SHALL provide "Contributor experience" as specified by FR-PL01-010 (BRD-12, priority Should, phase P1; Vietnamese: "Trải nghiệm contributor").

#### Scenario: Specified behaviour of FR-PL01-010
- **WHEN** the behaviour FR-PL01-010 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Only Rust + the platform toolchain are required; optional devcontainer and Nix flake; CONTRIBUTING, an architecture map, good-first-issue labels; CI runs on forks (except signing); fmt/clippy pre-commit hooks.
- **AND** the VirtualBox reference for the current behaviour is `configure.py:3443-3577 (36 library + 19 tool checks)`

#### Scenario: Acceptance for FR-PL01-010
- **WHEN** the FR-PL01-010 acceptance check is run
- **THEN** An onboarding test with 3 volunteers: build and run the Manager on MockEngine within 30 min; fork PRs run every non-secret job.

### Requirement: MockEngine for UI and e2e tests (FR-PL01-011)
Virtax SHALL provide "MockEngine for UI and e2e tests" as specified by FR-PL01-011 (BRD-12, priority Should, phase P1; Vietnamese: "MockEngine cho test UI và e2e").

#### Scenario: Specified behaviour of FR-PL01-011
- **WHEN** the behaviour FR-PL01-011 covers is exercised
- **THEN** it behaves as BRD-12 specifies: `virtax-engine` ships a deterministic `MockEngine` with scriptable latency and failures, so Manager/Console e2e tests run on every runner without a hypervisor; traces check the UI thread ≤ 50 ms.
- **AND** the VirtualBox reference for the current behaviour is `new (FR-GEN-006)`

#### Scenario: Acceptance for FR-PL01-011
- **WHEN** the FR-PL01-011 acceptance check is run
- **THEN** The UI e2e suite runs on macOS/Windows runners without VirtualBox; a 50 ms violation fails the test.

### Requirement: Release pipeline and provenance (FR-PL01-012)
Virtax SHALL provide "Release pipeline and provenance" as specified by FR-PL01-012 (BRD-12, priority Must, phase P1; Vietnamese: "Pipeline phát hành và provenance").

#### Scenario: Specified behaviour of FR-PL01-012
- **WHEN** the behaviour FR-PL01-012 covers is exercised
- **THEN** it behaves as BRD-12 specifies: Tag → build → test → sign → package → attest → GitHub Releases + update manifest (CP-14) per diagram 51; an automated release checklist; the nightly channel publishes daily from `main`; any red gate stops the release.
- **AND** the VirtualBox reference for the current behaviour is `Installer/win/Makefile.kmk:562-601`

#### Scenario: Acceptance for FR-PL01-012
- **WHEN** the FR-PL01-012 acceptance check is run
- **THEN** Tag to release ≤ 2 h with 0 manual steps; a nightly every day; every artifact has an SBOM and an attestation.

### Requirement: As built — Unsafe code policy
Every crate SHALL inherit the workspace lints that forbid `unsafe_code`, except the crates listed in `UNSAFE_ALLOWED` in `xtask/src/main.rs` (today only `crates/virtax-vmm-hv`), which SHALL deny `clippy::undocumented_unsafe_blocks` so every unsafe block carries a `SAFETY:` comment.

#### Scenario: Lint policy check
- **WHEN** `cargo xtask ci` runs
- **THEN** it fails if a crate does not inherit the workspace lints or an allowed crate does not deny undocumented unsafe blocks
- **AND** this is verified by `xtask/src/main.rs::check_lint_policy (run by `cargo xtask ci`)`

### Requirement: As built — Pinned KVM test guest
`cargo xtask guest-kernel` SHALL fetch a pinned Ubuntu kernel package, verify its SHA-256 before use and build a busybox initramfs; `cargo xtask kvm-test` SHALL run every `kvm_*` test, and CI SHALL run them in a job with `/dev/kvm` enabled.

#### Scenario: Initramfs format
- **WHEN** the initramfs archive is written
- **THEN** it is valid cpio newc padded to 512 bytes
- **AND** this is verified by `xtask/src/guest.rs::newc_layout`

### Requirement: As built — Developer recipes
The `justfile` SHALL define the toolchain and dev-loop recipes (build, lint, test, ci, guest, kvm-test, boot, dev-up/dev-down, ctl, events, demo) and the `Makefile` SHALL only forward targets to `just`. The dev daemon SHALL keep its data in `.cache/dev` and be tracked by a pidfile so recipes never start a second copy.

#### Scenario: Recipe list
- **WHEN** `just` is run with no arguments
- **THEN** it lists every recipe with its description
- **AND** this is verified by `manual check (`just --list`)`
