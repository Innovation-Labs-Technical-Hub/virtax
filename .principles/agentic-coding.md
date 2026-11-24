# Agentic coding principles

Scope: work done in this repository by AI coding agents (Claude Code and others reading `AGENTS.md`), and by
people reviewing it.

## Principles

1. **Spec first, then code.** Behaviour changes start as an OpenSpec change that cites its requirement IDs; the
   change's tasks drive the code. An agent never invents a requirement or resolves a business decision.
2. **Evidence over confidence.** "Done" means a command ran and its output was read: tests, `cargo xtask ci`, a
   booted guest. A claim without that evidence is reported as unverified.
3. **Small, reviewable steps.** One behaviour per commit, each with its tests, so a reviewer can follow the
   reasoning and revert precisely.
4. **The workspace is shared.** Other people and agents edit the same files; an agent changes only what its task
   needs and leaves everything else as it found it.
5. **Stop when stuck, don't improvise on real data.** A surprising result is investigated (reproduce, isolate,
   prove the cause) before anything else is changed.

## Rules

1. MUST read the relevant `openspec/specs/...` capability and its `.principles` file before changing behaviour,
   and MUST work through an OpenSpec change (`openspec-propose`, then `openspec-apply-change`) for anything that
   changes externally visible behaviour.
2. MUST NOT resolve a decision or open question marked TO BE CONFIRMED in
   `openspec/specs/governance/decisions-and-open-questions/spec.md`; follow its proposed default and say so in the
   proposal and the final report.
3. MUST verify before claiming: run the narrowest test first, then `cargo xtask ci`, and `cargo xtask kvm-test`
   for engine or VMM changes. Report failures with their output; never weaken or skip a test to make it pass.
4. MUST prove a bug's cause before fixing it: reproduce it, change one variable at a time (for example boot with
   `acpi=off` or `noapic`), and keep only the fix that the evidence supports.
5. MUST stage explicit paths (never `git add -A` or `git add .`) and read `git diff --cached` before committing;
   edits the agent did not make stay out of its commits and are reported to the user.
6. MUST use conventional commit messages without AI attribution, and MUST NOT put plan IDs, phase numbers or
   finding codes in code, test names or commit messages; describe the behaviour instead.
7. MUST confirm with the user before outward-facing or hard-to-reverse actions not already requested: pushing,
   merging, changing repository settings, deleting branches or data, publishing artifacts.
8. MUST clean up every process it starts (daemons, VMs, watchers) and verify none are left with `pgrep`; tests
   that spawn processes kill them on drop.
9. MUST re-read a file after formatting or concurrent edits before patching it, and patch against its current
   text; a scripted edit that fails to match must stop, not half-apply.
10. SHOULD keep an implementation plan for multi-step work in `plans/<date>-<slug>/plan.md` with its acceptance
    criteria, and mark it done only when every criterion was verified.
11. SHOULD lead reports with the outcome, state what was verified and how, and list what was not done.

## Enforcement

| Rule | Check |
|---|---|
| 1, 2 | `openspec validate --all --strict` (`cargo xtask spec`, CI job `openspec validate`); review of the change's proposal for cited IDs and decisions |
| 3 | CI jobs `ci` and `kvm boot tests` on every pull request |
| 4–11 | Review of the pull request, its commits and the agent's report |

## Anti-patterns

- **Guessing a name.** A request said "termax" in a repository named `virtax`, and a separate Termax product
  exists. Fix: check the evidence, then ask one clear question before a repository-wide rename.
- **Placeholder facts in a report.** A report once named a commit hash that did not exist. Fix: quote hashes,
  counts and test results from command output only.
- **Half-applied scripted edits.** A multi-part patch script failed its third assertion after rustfmt reflowed the
  file, and the first two parts had already been written. Fix: write the file only after every replacement
  matched, then re-run the tests.
- **Fixes that stay after failing.** An ACPI interrupt override that did not fix lost serial output stayed until
  it was tested on its own and removed. Fix: rule 4.

## References

- `openspec/AGENTS.md` (spec workflow and conventions), root `AGENTS.md` (repository map and commands).
- The other `.principles` files, which the agent applies to the code it touches.
