## 💡 Summary
This is a learning PR. The intended patch extracted inline sorting logic from `tokmd-model`'s core into public deterministic sort functions and updated `determinism_w66.rs` to use them directly to close a testing gap. However, this work was found to be superseded by PR #1584. The patch was aborted gracefully.

## 🎯 Why
The `determinism_w66.rs` test suite previously redefined `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows`. This means the test suite was asserting determinism against its *own* logic, rather than the library's actual behavior. Any mutation in the production library's inline sorting closures would go unnoticed. The work was meant to close this mutation coverage gap.

## 🔎 Evidence
- File: `crates/tokmd-model/tests/determinism_w66.rs`
- Finding: The patch was verified locally and tests passed, but it was noted in a PR comment that it was superseded by #1584 which aligned row-sorting extraction while keeping the helpers private instead of expanding the public API.

## 🧭 Options considered
### Option A (recommended)
- Stop work, abort the redundant fix gracefully, and create a learning PR documenting the workflow edge case.
- Fits the rule: "If an intended patch is found to be superseded by another merged PR during execution, gracefully abort the redundant fix and create a 'learning PR'."

### Option B
- N/A

## ✅ Decision
Option A. Acknowledging that the work has been superseded, aborting the patch, and creating a learning PR.

## 🧱 Changes made (SRP)
- `.jules/friction/open/mutant_high_value_superseded.md`

## 🧪 Verification receipts
```text
None (Learning PR)
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: N/A
- Risk class: N/A
- Rollback: N/A
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`
- `.jules/friction/open/mutant_high_value_superseded.md`

## 🔜 Follow-ups
None
