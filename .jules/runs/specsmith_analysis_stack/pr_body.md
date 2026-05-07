## 💡 Summary
Fixed TODO tag counting to respect word boundaries and not incorrectly flag substrings.

## 🎯 Why
Using a naive `.matches()` approach caused string counting issues where variables like `todo_app` could be mistakenly classified as a TODO comment, artificially inflating code health metrics in edge cases.

## 🔎 Evidence
- `crates/tokmd-analysis/src/content/io.rs`
- Ran `cargo test -p tokmd-analysis` successfully to verify correctness.

## 🧭 Options considered
### Option A (recommended)
- Fix `count_tags` in `io.rs` to correctly verify boundaries.
- Trade-offs: Simple and directly addresses the edge case.

### Option B
- Introduce a full regex crate to handle tags.
- Trade-offs: Slower performance and extra dependency.

## ✅ Decision
Option A was chosen.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/content/io.rs`: updated `count_tags` to ensure matching strings lie on word boundaries (non-alphabetic preceding and succeeding characters).

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis
```

## 🧭 Telemetry
- Change shape: Implementation code change.
- Blast radius: Only `tokmd-analysis/src/content/io.rs`
- Risk class: Low
- Gates run: `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_analysis_stack/envelope.json`
- `.jules/runs/specsmith_analysis_stack/decision.md`
- `.jules/runs/specsmith_analysis_stack/receipts.jsonl`
- `.jules/runs/specsmith_analysis_stack/result.json`
- `.jules/runs/specsmith_analysis_stack/pr_body.md`

## 🔜 Follow-ups
None.
