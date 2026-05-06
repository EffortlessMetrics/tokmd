## 💡 Summary
This is a learning PR. The previously attempted structural optimization to reduce allocation overhead in derived report aggregations was superseded by PR #1608. The run has been gracefully aborted and converted into a learning outcome to document this workflow edge case.

## 🎯 Why
During execution, a reviewer noted that PR #1608 already folded the useful borrowed-key derived-report allocation reductions into the current main branch. As a result, the active patch is obsolete. According to the core constraint directives, when a fix is superseded during execution, the agent must gracefully abort and create a learning PR documenting the friction item.

## 🔎 Evidence
- file path: `crates/tokmd-analysis/src/derived/mod.rs` (aborted patch target)
- observed behavior: A PR review comment stated: `Superseded by #1608, which folded the useful borrowed-key derived-report allocation reductions into a current-main keeper.`
- command: `cargo test -p tokmd --test determinism_regression` (fallback gate verification passed before abort).

## 🧭 Options considered
### Option A
- Continue applying the patch.
- Overwrites external work and fails the review constraint.

### Option B (recommended)
- Gracefully abort the fix and create a learning PR.
- Adheres to explicit instructions regarding superseded PR workflows.

## ✅ Decision
Option B was chosen to prevent duplicating work and to properly document the workflow race condition in `.jules/friction/open/`.

## 🧱 Changes made (SRP)
- `.jules/friction/open/FRIC-20250430-001.md`

## 🧪 Verification receipts
```text
cargo test -p tokmd --test determinism_regression
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
```

## 🧭 Telemetry
- Change shape: Learning PR documentation.
- Blast radius: None (documentation only).
- Risk class: Low.
- Rollback: `rm .jules/friction/open/FRIC-20250430-001.md`.
- Gates run: `cargo xtask version-consistency`, `cargo test -p tokmd --test determinism_regression`.

## 🗂️ .jules artifacts
- `.jules/runs/bolt-derived-alloc-opt/envelope.json`
- `.jules/runs/bolt-derived-alloc-opt/decision.md`
- `.jules/runs/bolt-derived-alloc-opt/receipts.jsonl`
- `.jules/runs/bolt-derived-alloc-opt/result.json`
- `.jules/runs/bolt-derived-alloc-opt/pr_body.md`
- `.jules/friction/open/FRIC-20250430-001.md`

## 🔜 Follow-ups
None immediately.
