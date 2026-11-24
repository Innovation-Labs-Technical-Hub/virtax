# Security & encryption Specification

## Purpose
Least privilege, peer auth, OS keystore, disk/NVRAM encryption, TPM 2.0 + Secure Boot, threat model. Module `CP-12` (Security & encryption; Vietnamese: "Bảo mật & mã hoá") from BRD-11, phase P1, priority Must. Owner crates: virtaxd, virtax-helper. Depends on: `control-plane/control-plane-api-and-ipc`, `platform/packaging-signing-and-installers`. VirtualBox reference: `src/VBox/Main/src-all/CryptoUtils.cpp, Devices/Security (TPM), HostDrivers/Support (hardening)`. Implementation status (2026-11-24): not started.

## Requirements

### Requirement: Threat model and trust boundaries (FR-CP12-001)
Virtax SHALL provide "Threat model and trust boundaries" as specified by FR-CP12-001 (BRD-11, priority Must, phase P1; Vietnamese: "Threat model và ranh giới tin cậy").

#### Scenario: Specified behaviour of FR-CP12-001
- **WHEN** the behaviour FR-CP12-001 covers is exercised
- **THEN** it behaves as BRD-11 specifies: A STRIDE threat model with data-flow diagrams for the boundaries user ↔ `virtaxd` (same UID) ↔ `virtax-helper` (elevated) ↔ VM process (hardened VBoxHeadless / `virtax-vmm`) ↔ guest (untrusted), plus plugins and the update channel. Every new IPC declares its boundary; reviewed each release.
- **AND** the VirtualBox reference for the current behaviour is `new; HostDrivers/Support/SUPR3HardenedMain.cpp (old model)`

#### Scenario: Acceptance for FR-CP12-001
- **WHEN** the FR-CP12-001 acceptance check is run
- **THEN** Threat model in the repo with a PR-template checklist; external review before Beta and GA; 0 open High findings at GA.

### Requirement: Peer-credential authentication on every socket (FR-CP12-002)
Virtax SHALL provide "Peer-credential authentication on every socket" as specified by FR-CP12-002 (BRD-11, priority Must, phase P1; Vietnamese: "Xác thực peer-credential trên mọi socket").

#### Scenario: Specified behaviour of FR-CP12-002
- **WHEN** the behaviour FR-CP12-002 covers is exercised
- **THEN** it behaves as BRD-11 specifies: 0600 UDS in `$XDG_RUNTIME_DIR` or `~/Library/Application Support/Virtax` (never /tmp) checked with `SO_PEERCRED`/`LOCAL_PEERCRED`; named pipes with a DACL for the user SID only + `GetNamedPipeClientProcessId`. Remote TCP is off by default and only allowed with mTLS.
- **AND** the VirtualBox reference for the current behaviour is `ipcd/shared/src/ipcConfig.cpp:50-51; webservice/vboxweb.cpp:911`

#### Scenario: Acceptance for FR-CP12-002
- **WHEN** the FR-CP12-002 acceptance check is run
- **THEN** Another local user connecting → VTX-SEC-001 and an audit entry; a pentest finds no world-accessible socket/pipe (`ls -l`, `accesschk`).

### Requirement: Capability tokens for the data plane (FR-CP12-003)
Virtax SHALL provide "Capability tokens for the data plane" as specified by FR-CP12-003 (BRD-11, priority Must, phase P1; Vietnamese: "Capability token cho data plane").

#### Scenario: Specified behaviour of FR-CP12-003
- **WHEN** the behaviour FR-CP12-003 covers is exercised
- **THEN** it behaves as BRD-11 specifies: `OpenDisplay` returns a single-use token, redeemable within 60 s, bound to machine/screen/Console PID and scoped (`display:read`, `input:write`). The input socket and SHM open only with a token; tokens are revoked when the VM stops or the Console closes.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-client/DisplayImpl.cpp:925-1006`

#### Scenario: Acceptance for FR-CP12-003
- **WHEN** the FR-CP12-003 acceptance check is run
- **THEN** A replayed token is rejected; a token from another PID is rejected; a token older than 60 s expires (automated tests).

### Requirement: Least-privilege virtax-helper (FR-CP12-004)
Virtax SHALL provide "Least-privilege virtax-helper" as specified by FR-CP12-004 (BRD-11, priority Must, phase P1; Vietnamese: "virtax-helper đặc quyền tối thiểu").

#### Scenario: Specified behaviour of FR-CP12-004
- **WHEN** the behaviour FR-CP12-004 covers is exercised
- **THEN** it behaves as BRD-11 specifies: The only privileged component: ≤ 3k LOC, a narrow typed API (≤ 15 methods), validated arguments (interface-name patterns, owner UID), no shell-outs, an audit log of every action, started on demand and exiting when idle. Every method has a fuzz target; external audit before GA.
- **AND** the VirtualBox reference for the current behaviour is `HostDrivers/adpctl/VBoxNetAdpCtl.cpp:89-90,359,732`

#### Scenario: Acceptance for FR-CP12-004
- **WHEN** the FR-CP12-004 acceptance check is run
- **THEN** `tokei` ≤ 3k LOC; 100% of methods fuzzed; the helper exits after 60 s idle; the audit report has 0 open High findings.

### Requirement: No setuid, no permanently elevated process (FR-CP12-005)
Virtax SHALL provide "No setuid, no permanently elevated process" as specified by FR-CP12-005 (BRD-11, priority Must, phase P1; Vietnamese: "Không setuid, không process elevated thường trực").

#### Scenario: Specified behaviour of FR-CP12-005
- **WHEN** the behaviour FR-CP12-005 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Virtax ships no setuid binaries; Manager, Console, `virtaxd` and `virtax-vmm` run as the user; Linux reaches `/dev/kvm` through the `kvm` group. Any Phase-1 Engine Pack exception is listed in the SBOM with a reason and removed in Phase 2.
- **AND** the VirtualBox reference for the current behaviour is `Installer/linux/install.sh:296-313; Support/win/VBoxSup.inf:76`

#### Scenario: Acceptance for FR-CP12-005
- **WHEN** the FR-CP12-005 acceptance check is run
- **THEN** After install, `find / -perm -4000` shows no Virtax files (except declared P1 exceptions, 0 in P2); no service auto-starts with admin rights.

### Requirement: Sandboxing for the virtax-vmm process (FR-CP12-006)
Virtax SHALL provide "Sandboxing for the virtax-vmm process" as specified by FR-CP12-006 (BRD-11, priority Must, phase P2; Vietnamese: "Sandbox cho process virtax-vmm").

#### Scenario: Specified behaviour of FR-CP12-006
- **WHEN** the behaviour FR-CP12-006 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Linux: seccomp-bpf (seccompiler) per thread class, `no_new_privs`, landlock limited to the VM folder and granted ISOs. macOS: hardened runtime + App Sandbox with `com.apple.security.hypervisor`, files via security-scoped bookmarks. Windows: AppContainer or low integrity + job object (no child processes) + mitigation policies (CIG/ACG).
- **AND** the VirtualBox reference for the current behaviour is `SUPR3HardenedMain-win.cpp:84-90,4777-4830`

#### Scenario: Acceptance for FR-CP12-006
- **WHEN** the FR-CP12-006 acceptance check is run
- **THEN** Escape tests: `virtax-vmm` cannot open files outside granted folders and `exec` is blocked; a seccomp violation kills the process and reports VTX-SEC-010.

### Requirement: Fuzz every guest-facing parser (FR-CP12-007)
Virtax SHALL provide "Fuzz every guest-facing parser" as specified by FR-CP12-007 (BRD-11, priority Must, phase P2; Vietnamese: "Fuzz mọi parser hướng guest").

#### Scenario: Specified behaviour of FR-CP12-007
- **WHEN** the behaviour FR-CP12-007 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Fuzz targets (cargo-fuzz/libFuzzer) for virtqueue descriptor chains, virtio-net headers, virtio-gpu commands, NVMe, xHCI TRBs, fw_cfg, TPM CRB, the agent vsock protocol and the helper API; nightly runs and ≥ 24 CPU-hours per release; crashes block the release.
- **AND** the VirtualBox reference for the current behaviour is `Devices/VirtIO/VirtioCore.cpp:180-245,875-883`

#### Scenario: Acceptance for FR-CP12-007
- **WHEN** the FR-CP12-007 acceptance check is run
- **THEN** ≥ 70% line coverage in parsers; 0 open crashes at release; the regression corpus is kept in the repo.

### Requirement: Disk, NVRAM and state encryption with OS keystores (FR-CP12-008)
Virtax SHALL provide "Disk, NVRAM and state encryption with OS keystores" as specified by FR-CP12-008 (BRD-11, priority Should, phase P2; Vietnamese: "Mã hoá disk, NVRAM và state với keystore OS").

#### Scenario: Specified behaviour of FR-CP12-008
- **WHEN** the behaviour FR-CP12-008 covers is exercised
- **THEN** it behaves as BRD-11 specifies: `virtax-vmm` encrypts images with AES-XTS-256 per whole request (AES-NI/ARMv8-CE) off the I/O thread; NVRAM, TPM state and saved state share the DEK set. The DEK is wrapped by a KEK in Keychain, DPAPI/Credential Manager or Secret Service, or by a passphrase (Argon2id); keys are zeroized.
- **AND** the VirtualBox reference for the current behaviour is `Storage/VDFilterCrypt.cpp:379-440; NvramStoreImpl.cpp:93-96,659`

#### Scenario: Acceptance for FR-CP12-008
- **WHEN** the FR-CP12-008 acceptance check is run
- **THEN** fio on an encrypted VM is ≤ 15% slower than unencrypted; no plaintext key on disk (grep test); one unlock per session, Touch ID/Windows Hello supported.

### Requirement: Compatibility with VirtualBox AES-XTS encrypted VMs (FR-CP12-009)
Virtax SHALL provide "Compatibility with VirtualBox AES-XTS encrypted VMs" as specified by FR-CP12-009 (BRD-11, priority Must, phase P1; Vietnamese: "Tương thích VM mã hoá AES-XTS của VirtualBox").

#### Scenario: Specified behaviour of FR-CP12-009
- **WHEN** the behaviour FR-CP12-009 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Phase 1 via the legacy engine: detect encrypted VMs/disks (AES-XTS128/256-PLAIN64, PBKDF2-SHA*), ask for the password in an in-app sheet (replacing R-17) and call `IConsole.addEncryptionPassword`; optional "remember in keystore". Phase 2: import reads the VDKeyStore format.
- **AND** the VirtualBox reference for the current behaviour is `MachineImpl.cpp:16360,16579; VDKeyStore.cpp:165-184`

#### Scenario: Acceptance for FR-CP12-009
- **WHEN** the FR-CP12-009 acceptance check is run
- **THEN** A VM encrypted by VirtualBox 7.2 runs in Virtax after the password; a wrong password → VTX-SEC-020 inline; "remember" survives a reboot.

### Requirement: TPM 2.0 + Secure Boot policy and key management (FR-CP12-010)
Virtax SHALL provide "TPM 2.0 + Secure Boot policy and key management" as specified by FR-CP12-010 (BRD-11, priority Must, phase P1; Vietnamese: "Chính sách TPM 2.0 + Secure Boot và quản lý khoá").

#### Scenario: Specified behaviour of FR-CP12-010
- **WHEN** the behaviour FR-CP12-010 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Defaults per GuestOSType: Windows 11 → TPM 2.0 + Secure Boot + Microsoft keys; Linux → Secure Boot with the Microsoft UEFI CA + shim. A per-VM PK, never Oracle's. A Security card in Settings shows PK/KEK/db/dbx, resets defaults and enrolls MOKs; P1 goes through `IUefiVariableStore`.
- **AND** the VirtualBox reference for the current behaviour is `UefiVariableStoreImpl.cpp:509-649; Global.cpp:213-216`

#### Scenario: Acceptance for FR-CP12-010
- **WHEN** the FR-CP12-010 acceptance check is run
- **THEN** A new Windows 11 VM passes PC Health Check; Secure Boot state shows in Settings; key reset takes ≤ 2 clicks; the default store has no Oracle PK.

### Requirement: Supply-chain gates: cargo-deny, cargo-vet, cargo-audit (FR-CP12-011)
Virtax SHALL provide "Supply-chain gates: cargo-deny, cargo-vet, cargo-audit" as specified by FR-CP12-011 (BRD-11, priority Must, phase P1; Vietnamese: "Cổng supply chain: cargo-deny, cargo-vet, cargo-audit").

#### Scenario: Specified behaviour of FR-CP12-011
- **WHEN** the behaviour FR-CP12-011 covers is exercised
- **THEN** it behaves as BRD-11 specifies: CI gates: cargo-deny (GPLv3-compatible license allowlist, bans, sources, advisories), daily cargo-audit, cargo-vet with imported Mozilla/Google/Bytecode Alliance audits; `Cargo.lock` committed; no git dependency without a rev.
- **AND** the VirtualBox reference for the current behaviour is `src/libs (20 bundled libraries)`

#### Scenario: Acceptance for FR-CP12-011
- **WHEN** the FR-CP12-011 acceptance check is run
- **THEN** A PR adding an SSPL/BUSL dependency fails CI; a new RUSTSEC advisory → issue filed within 24 h; releases block on unvetted crates.

### Requirement: SBOM, provenance, reproducible builds, signed releases (FR-CP12-012)
Virtax SHALL provide "SBOM, provenance, reproducible builds, signed releases" as specified by FR-CP12-012 (BRD-11, priority Must, phase P1; Vietnamese: "SBOM, provenance, build tái lập, release có chữ ký").

#### Scenario: Specified behaviour of FR-CP12-012
- **WHEN** the behaviour FR-CP12-012 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Every artifact has a CycloneDX SBOM (Rust + bundled C: libtpms, OVMF, Engine Pack) and SLSA provenance via GitHub artifact attestations; Linux builds are bit-for-bit reproducible; every artifact is signed (Authenticode, codesign + notarisation, GPG, minisign — mechanics in BRD-12).
- **AND** the VirtualBox reference for the current behaviour is `Installer/win/Makefile.kmk:562-601`

#### Scenario: Acceptance for FR-CP12-012
- **WHEN** the FR-CP12-012 acceptance check is run
- **THEN** `gh attestation verify` succeeds for every artifact; a rebuild on a second runner gives identical hashes (.deb, AppImage); every release carries its SBOM.

### Requirement: Guided Windows hardening errors (Phase 1) (FR-CP12-013)
Virtax SHALL provide "Guided Windows hardening errors (Phase 1)" as specified by FR-CP12-013 (BRD-11, priority Must, phase P1; Vietnamese: "Lỗi hardening Windows có hướng dẫn (Pha 1)").

#### Scenario: Specified behaviour of FR-CP12-013
- **WHEN** the behaviour FR-CP12-013 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Group the 82 `VERR_SUP_VP_*` codes into a few VTX-SEC-03x families with an explanation, the suspected AV (from the 23 adversary flags), the `VBoxHardening.log` path and fix steps (AV exclusion, remove injected DLL). Virtax never injects into hardened processes.
- **AND** the VirtualBox reference for the current behaviour is `SUPR3HardenedMain-win.cpp:413-454; err.h:2902-2997`

#### Scenario: Acceptance for FR-CP12-013
- **WHEN** the FR-CP12-013 acceptance check is run
- **THEN** 100% of `VERR_SUP_VP_*` codes are mapped; UX test: ≥ 80% of users fix common AV conflicts unaided.

### Requirement: Replace the Windows hardening model (Phase 2) (FR-CP12-014)
Virtax SHALL provide "Replace the Windows hardening model (Phase 2)" as specified by FR-CP12-014 (BRD-11, priority Must, phase P2; Vietnamese: "Thay mô hình hardening Windows (Pha 2)").

#### Scenario: Specified behaviour of FR-CP12-014
- **WHEN** the behaviour FR-CP12-014 covers is exercised
- **THEN** it behaves as BRD-11 specifies: `virtax-vmm` has no own driver, so no respawn chain, purification or AV sleep loops are needed. Protection comes from OS mitigations (`SetProcessMitigationPolicy`: CIG, ACG, signed-binaries-only), the FR-CP12-006 sandbox and a one-time signature check at install.
- **AND** the VirtualBox reference for the current behaviour is `SUPR3HardenedMain-win.cpp:4777-4830,6243-6281`

#### Scenario: Acceptance for FR-CP12-014
- **WHEN** the FR-CP12-014 acceptance check is run
- **THEN** Windows with Defender + one third-party AV: security overhead from spawn to first guest instruction ≤ 50 ms (≥ 640 ms today); 0 fixed sleep loops in code.

### Requirement: Secrets hygiene (FR-CP12-015)
Virtax SHALL provide "Secrets hygiene" as specified by FR-CP12-015 (BRD-11, priority Should, phase P1; Vietnamese: "Vệ sinh bí mật").

#### Scenario: Specified behaviour of FR-CP12-015
- **WHEN** the behaviour FR-CP12-015 covers is exercised
- **THEN** it behaves as BRD-11 specifies: Unattended passwords, encryption keys and plugin tokens never reach logs, diagnostic bundles or `machine.toml`; a `Secret` type zeroizes and redacts in `Debug`; redaction tests run on seeded data.
- **AND** the VirtualBox reference for the current behaviour is `Main/src-server/UnattendedImpl.cpp; Frontends/VBoxBugReport`

#### Scenario: Acceptance for FR-CP12-015
- **WHEN** the FR-CP12-015 acceptance check is run
- **THEN** Scanning an `virtaxctl diag bundle` finds 0 seeded secrets; JSON logs contain no values of `Secret` fields.
