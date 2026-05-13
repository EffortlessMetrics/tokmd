## 💡 Summary
Moved the `source_complexity` module from `tokmd-analysis` to `tokmd-cockpit`. This heuristic is only used for PR metric complexity gates, so it now lives directly under the cockpit's `gates` module.

## 🎯 Why
The `source_complexity` logic was sitting in `tokmd-analysis` but was solely consumed by `tokmd-cockpit::gates::complexity`. According to the architecture guidelines, `tokmd-analysis` orchestrates presets and derived metrics; it should not own internal gate heuristics used exclusively by the cockpit tier. This cleans up the crate boundary and ensures that cockpit logic remains in the cockpit.

## 🔎 Evidence
- `tokmd-analysis/src/source_complexity.rs` was exclusively imported by `tokmd-cockpit/src/gates/complexity.rs`.
- `cargo test -p tokmd-cockpit -p tokmd-analysis` ran successfully after the refactor.

## 🧭 Options considered
### Option A (recommended)
- what it is: Move `source_complexity` to `tokmd-cockpit/src/gates/`.
- why it fits this repo and shard: Directly addresses a tier boundary violation. The PR cockpit metrics should own their own heuristics.
- trade-offs: Structure is improved, no velocity hit, aligns perfectly with the governance tier model.

### Option B
- what it is: Keep it in `tokmd-analysis` and expose it as a derived metric.
- when to choose it instead: If other workflows needed the raw function-level cyclomatic complexity heuristics.
- trade-offs: Bloats the analysis crate with gate-specific code that no other consumer uses.

## ✅ Decision
Option A. The `source_complexity` module was a clear tier boundary violation and is much better situated inside `tokmd-cockpit` where it is actually used. The `SourceAnalyzer` trait was also removed since it was an unused abstraction that only added noise to the gate logic.

## 🧱 Changes made (SRP)
- Moved `crates/tokmd-analysis/src/source_complexity/mask.rs` to `crates/tokmd-cockpit/src/gates/source_complexity/mask.rs`.
- Moved `crates/tokmd-analysis/src/source_complexity.rs` to `crates/tokmd-cockpit/src/gates/source_complexity/mod.rs`.
- Updated `crates/tokmd-cockpit/src/gates.rs` and `crates/tokmd-cockpit/src/gates/complexity.rs` to use the new local module.
- Removed `pub mod source_complexity;` from `crates/tokmd-analysis/src/lib.rs`.
- Removed the unused `SourceAnalyzer` trait from the moved module.

## 🧪 Verification receipts
```text
cargo fmt
cargo test -p tokmd-cockpit -p tokmd-analysis
cargo clippy -p tokmd-cockpit -p tokmd-analysis -- -D warnings
```

## 🧭 Telemetry
- Change shape: Refactoring
- Blast radius: `tokmd-analysis`, `tokmd-cockpit`
- Risk class: Low
- Rollback: Revert the commit.
- Gates run: `cargo test -p tokmd-cockpit -p tokmd-analysis`, `cargo clippy -p tokmd-cockpit -p tokmd-analysis -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/run-surveyor-workspace/envelope.json`
- `.jules/runs/run-surveyor-workspace/decision.md`
- `.jules/runs/run-surveyor-workspace/receipts.jsonl`
- `.jules/runs/run-surveyor-workspace/result.json`
- `.jules/runs/run-surveyor-workspace/pr_body.md`

## 🔜 Follow-ups
None.
