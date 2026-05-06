## 💡 Summary
This is a learning PR. The attempt to optimize map reporting logic in `crates/tokmd-analysis/src/derived/mod.rs` was superseded by #1608, which successfully merged the measured derived-analysis allocation keeper. This draft is now stale.

## 🎯 Why
During execution, it was found that the intended patch (and the initial draft learning packet) was superseded by #1608. Following strict governance, we gracefully abort the redundant fix and capture this workflow edge case in a learning PR.

## 🔎 Evidence
- **Observed behavior**: Pull Request #1608 merged the allocation keeper changes.
- **Receipt**: PR comment stating "Superseded by #1608... This draft learning packet is stale now."

## 🧭 Options considered
### Option A (recommended)
- Halt the patch and file a Learning PR.
- Prevents useless code drift and gracefully handles supersession by documenting the workflow edge case as a friction item.
- Trade-offs: Structure/Velocity/Governance: Perfectly aligned with strict governance rules for superseded patches.

### Option B
- Ignore the supersession and push redundant changes.
- Trade-offs: Degrades trust, creates merge conflicts, and duplicates work already merged in #1608.

## ✅ Decision
Option A. I am abandoning the stale patch and finalizing this run as a Learning PR because the work was superseded by PR #1608.

## 🧱 Changes made (SRP)
- None.

## 🧪 Verification receipts
```text
$ cargo check -p tokmd-analysis
    Finished dev [unoptimized + debuginfo] target(s) in 0.46s
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None.
- Risk class: No risk.
- Rollback: N/A
- Gates run: `cargo check`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`
- `.jules/friction/open/bolt-hot-path-fake-out.md`
- `.jules/friction/open/superseded-by-1608.md`

## 🔜 Follow-ups
Created a friction item documenting the workflow edge case where this work was superseded by #1608.
