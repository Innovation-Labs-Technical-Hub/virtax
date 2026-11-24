# OpenSpec in Virtax

`openspec/` holds the behaviour contract for Virtax: what the product must do, where each requirement came from,
and what is still undecided. Code and tests implement it; changes to behaviour start here.

## Layout

- `specs/<layer>/<module>/spec.md`: one capability per module of the design set's module catalogue, in the layers
  `ui`, `control-plane`, `engine` and `platform`. Each Purpose names the module ID, its owner crates, its
  dependencies and its implementation status.
- `specs/general/common-requirements`: rules every capability inherits (BRD-00).
- `specs/quality/non-functional-requirements`: measurable targets (every NFR from BRD-00 to BRD-12).
- `specs/governance/decisions-and-open-questions`: decisions D1–D6 and every BRD open question, with owner,
  proposed default and status.
- `changes/<name>/`: in-flight changes (proposal, delta specs, design, tasks); `changes/archive/` holds finished ones.
- `config.yaml`: project context and artifact rules that OpenSpec passes to agents.

The specs were extracted once from the design set "OxBox Rust Rebuild" v1.0, which lives outside this repository.
From now on they are the source of truth: edit them through changes, not by regenerating them.

## Commands

The CLI is pinned to 2.1.1; `just spec` runs the validation.

```sh
bunx @fission-ai/openspec@2.1.1 list --specs                 # capabilities
bunx @fission-ai/openspec@2.1.1 show <capability> --type spec  # one capability
bunx @fission-ai/openspec@2.1.1 new change <kebab-name>     # start a change
bunx @fission-ai/openspec@2.1.1 status --change <name>      # what is left to write
bunx @fission-ai/openspec@2.1.1 validate --all --strict     # must pass before merging
bunx @fission-ai/openspec@2.1.1 archive <name>              # merge a finished change into the specs
```

In Claude Code the same workflow is available as skills (`openspec-propose`, `openspec-apply-change`,
`openspec-archive-change` and others) in `.agents/skills/`, linked from `.claude/skills/`.

## Working on a change

1. **Propose:** create the change, then write proposal, delta specs, design (when the change is cross-cutting or
   adds dependencies) and tasks, in the order `openspec status` gives. Planning does not touch code.
2. **Apply:** implement the tasks in order and tick each one only when its verification passes.
3. **Archive:** after review, `openspec archive <name>` moves the delta requirements into the main specs.

Every change must cite the IDs it implements (FR-…, NFR-…, Q-…, D1–D6) and the capabilities it touches.

## How specs map to code and tests

| Spec element | Where it lives |
|---|---|
| Capability | The owner crates named in its Purpose (for example `control-plane/registry-and-configuration-store` → `crates/virtax-store`, `crates/virtax-model`) |
| API requirements | `crates/virtax-api/proto/virtax/v1/virtax.proto` |
| Manifest requirements | `crates/virtax-model/src/manifest.rs` and the generated `schemas/virtax.machine.v1.json` |
| Error codes (`VTX-…`) | `crates/virtax-model/src/errors.rs` |
| Scenario | A test: unit tests next to the code, end-to-end tests in `apps/virtaxctl/tests/e2e.rs`, and `kvm_*` tests run by `cargo xtask kvm-test` |

Requirements quoted from the BRDs carry their ID and an "Acceptance" scenario taken from the BRD. "As built"
requirements describe implemented behaviour the BRDs do not cover or that diverges from them; each scenario names
the test that verifies it (`- **AND** this is verified by …`). A new scenario without a test is a gap to close,
not an implemented behaviour.

## Confirmation markers

- **TO BE CONFIRMED** marks a business decision the project owner has not made. Never resolve one by guessing.
  Follow the proposed default, and say in the proposal that the change depends on it.
- **CONFIRMED (date)** records a decision the owner made; the requirement states the decision.
- In the Vietnamese source text, "cần xác nhận" usually describes a confirmation dialog before a dangerous action
  (a UI behaviour), not an open decision.

## Rules for requirements

- Write requirements in English with SHALL or MUST and keep each description under 500 characters; put detail
  and examples in scenarios. Every requirement needs at least one WHEN/THEN scenario.
- Keep the source citation on every requirement you add or modify.
- Use Virtax names (`virtaxd`, `virtaxctl`, `virtax-vmm`, `VTX-…`); the design set's `oxbox…` and `OXE-…` names
  map to them.
