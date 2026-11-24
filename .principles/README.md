# Engineering principles

Project-specific rules for Virtax, each grounded in the architecture document, an ADR, an OpenSpec requirement
or something that went wrong in this repository. Every file lists how a violation is caught; rules marked
"not yet enforced" are review items until their check exists.

| Area | File | Applies to | Owner (role) | Enforced by |
|---|---|---|---|---|
| Rust conventions | [rust.md](rust.md) | every crate | Backend lead | CI `ci`: `cargo xtask ci` (fmt, lint policy, clippy `-D warnings`, tests) |
| Control plane | [control-plane.md](control-plane.md) | `virtaxd`, `virtaxctl`, `virtax-api`, `virtax-model`, `virtax-store`, `virtax-engine` | Backend lead | CI `ci`: unit and e2e tests |
| VMM | [vmm.md](vmm.md) | `virtax-vmm`, `virtax-vmm-core`, `virtax-vmm-hv` | VMM lead | CI `ci` lint policy; CI `kvm boot tests` |
| Security | [security.md](security.md) | whole repository | Backend lead with Platform/Release | CI `ci` refusal-path tests; checksum check in `cargo xtask guest-kernel` |
| Build and CI | [build-and-ci.md](build-and-ci.md) | `xtask`, `justfile`, `Makefile`, `.github/workflows` | Platform/Release | CI `ci` and `kvm boot tests` |
| Agentic coding | [agentic-coding.md](agentic-coding.md) | agents and reviewers | Tech lead | `just spec` (OpenSpec strict validation); pull-request review |

Owner roles come from the design set's MVP staffing (UI lead, Backend lead, Console engineer, Platform/Release
engineer) and the Phase 2 VMM lead.

## Not yet enforced anywhere

These rules appear in the files above without an automated check yet. Each names the requirement that will add one:

- `cargo-deny` (licences and advisories), `cargo-audit` and an SBOM per release (NFR-00-13, FR-PL01-004).
- `buf breaking` against the previous protocol release (FR-CP01-010).
- `cargo-fuzz` targets for guest-facing parsers and seccomp filters for `virtax-vmm` (architecture §"Security").
- A second-OS-user test for the socket's peer check (FR-CP01-002).
- OpenSpec validation and performance gates in CI (FR-PL01-007).

## Changing a principle

Change a rule in the same pull request as the code or check that motivates it, and keep its reference. A rule
nobody can check or explain is removed, not kept "for later".
