## 💡 Summary
Learning PR: The intended performance improvement for `build_integrity_report` was superseded by an existing PR (#1608). Recording learning artifacts and friction items.

## 🎯 Why
Attempted to optimize `compare_integrity_rows` by replacing `format!` allocations. However, PR review identified this was superseded by #1608, which already addressed the issue and avoided provenance path collisions.

## 🔎 Evidence
- Review comment: "Superseded by #1608. This older integrity-comparator allocation patch duplicated the keeper concept and also collided with another generated Bolt provenance path."

## 🧭 Options considered
### Option A (recommended)
- Record a Learning PR.
- Fits the scenario where a patch is identified as redundant/superseded during execution.
- Trade-offs: Structure (generates standard run artifacts without duplicate code), Velocity (fast conclusion), Governance (avoids merge conflicts and duplicated effort).

### Option B
- N/A - Path is blocked by superseded status.

## ✅ Decision
Option A was chosen. Aborting the redundant fix and creating a learning PR as instructed.

## 🧱 Changes made (SRP)
- Recorded friction item `.jules/friction/open/superseded_pr.md`.
- Reverted code changes to `crates/tokmd-analysis/src/derived/mod.rs`.

## 🧪 Verification receipts
```text
`mkdir -p .jules/runs/bolt_analysis_stack_builder` (exit code 0)
`python3 replace.py && cargo test -p tokmd-analysis --test derived` (exit code 0)
`python3 replace.py && cargo clippy -- -D warnings` (exit code 0)
`cargo fmt && cargo clippy -- -D warnings && cargo test -p tokmd-analysis --all-features` (exit code 0)
```

## 🧭 Telemetry
- Change shape: Learning / Friction recording
- Blast radius: None (code changes reverted)
- Risk class: None
- Rollback: None
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`
- `.jules/friction/open/superseded_pr.md`

## 🔜 Follow-ups
- Ensure #1608 correctly addresses the `compare_integrity_rows` hotpath.
