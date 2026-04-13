## 💡 Summary
Learning PR to record investigation into `tokmd baseline` failing to report `total_files` and `total_code_lines` when `tokmd-analysis-complexity` is not enabled, relying instead on `receipt.derived`.

## 🎯 Why
When `tokmd` is built without default features (like `walk`), running `tokmd baseline` would output `total_files: 0` because it fell back to a default `BaselineMetrics` when `receipt.complexity` was missing. This broke `baseline_metrics_has_total_files` test under the `--no-default-features --features walk` matrix combination.

## 🔎 Evidence
- File path: `crates/tokmd-analysis-types/src/lib.rs`
- Observed behavior: `baseline_metrics_has_total_files` test fails with `fixture should have at least one file`.
- Receipts: `cargo test -p tokmd --no-default-features --features walk --test baseline_w71` fails on main.

## 🧭 Options considered
### Option A (recommended)
- Lift `total_code_lines` and `total_files` computation from `receipt.derived` to apply even if `receipt.complexity` is `None`. Use these values in the `else` branch.
- Fits the `interfaces` and `compat-matrix` shard perfectly.
- Trade-offs: Trivial structural change that preserves all functionality.

### Option B
- Modify the test to conditionally skip the `total_files > 0` check if `tokmd-analysis-complexity` is not enabled.

## ✅ Decision
A patch was proposed using Option A, but was closed by the reviewer as obsolete since `main` already handles this correctly. Converting to a learning PR.

## 🧱 Changes made (SRP)
- Reverted code changes.
- Recorded friction item `.jules/friction/open/baseline_metrics_obsolete.md`.

## 🧪 Verification receipts
None required.

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class: None
- Rollback: N/A
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/<uuid>/envelope.json`
- `.jules/runs/<uuid>/decision.md`
- `.jules/runs/<uuid>/pr_body.md`
- `.jules/runs/<uuid>/receipts.jsonl`
- `.jules/runs/<uuid>/result.json`
- `.jules/friction/open/baseline_metrics_obsolete.md`
