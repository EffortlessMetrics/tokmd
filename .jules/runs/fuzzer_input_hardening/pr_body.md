## 💡 Summary
Created a learning PR because the intended fuzz-toolchain friction log was superseded by #1606. This PR documents the workflow edge case of encountering a superseded fix during execution.

## 🎯 Why
During the execution of this prompt, it was discovered via PR comments that the useful fuzz-toolchain blocker had already been consolidated into the current `.jules` friction rollup by PR #1606. To avoid redundant or conflicting patches, the original work was aborted and a new friction item was created to document this workflow edge case.

## 🔎 Evidence
- PR Comment from maintainer: "Superseded by #1606, which consolidated the useful fuzz-toolchain blocker into the current .jules friction rollup without carrying raw run packets."

## 🧭 Options considered
### Option A (recommended)
- Gracefully abort the original patch.
- Create a new friction item documenting the workflow edge case of a superseded PR.
- This fits the repo and shard by preventing duplicate work and recording the collision.
- Trade-offs: No new code changes are landed, but workflow hygiene is maintained.

### Option B
- Ignore the comment and force the original patch.
- When to choose it: Only if the prior PR was reverted or incorrect.
- Trade-offs: High risk of merge conflicts and maintainer frustration.

## ✅ Decision
Option A was chosen. I aborted the redundant fix and created a new friction item (`superseded_by_pr_workflow.md`) to document the edge case.

## 🧱 Changes made (SRP)
- `.jules/friction/open/superseded_by_pr_workflow.md` (created)
- `.jules/runs/fuzzer_input_hardening/` (created artifacts)

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.

$ cargo xtask publish --plan
Workspace version: 1.10.0-rc.1
Publish order (16 crates): [...]

$ cargo xtask version-consistency
Checking version consistency against workspace version 1.10.0-rc.1
  ✓ Cargo crate versions match 1.10.0-rc.1.
  ✓ Cargo workspace dependency versions match 1.10.0-rc.1.
  ✓ Node package manifest versions match 1.10.0-rc.1.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo fmt -- --check
[no output - passed]

$ cargo clippy -- -D warnings
[no output - passed]
```

## 🧭 Telemetry
- Change shape: scaffolding and friction documentation
- Blast radius: None (test/fuzz only)
- Risk class: Low
- Rollback: Revert PR
- Gates run: manual gate tests: `cargo xtask docs --check`, `cargo xtask publish --plan`, `cargo xtask version-consistency`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/friction/open/superseded_by_pr_workflow.md`

## 🔜 Follow-ups
- None.
