## 💡 Summary
Moved the `source_complexity` heuristic directly into `tokmd-cockpit` where it belongs. This drops `tokmd-analysis` from `tokmd-cockpit`'s dependency tree and resolves a tiering violation.

## 🎯 Why
`tokmd-cockpit` was improperly pulling orchestration internals from an adjacent module (`tokmd-analysis`), violating the rule that higher-tier products must own their specific domain heuristics locally or use shared contracts.

## 🔎 Evidence
- `crates/tokmd-cockpit/src/gates/complexity.rs` was directly using `tokmd_analysis::source_complexity::analyze_rust_function_complexity`.
- `crates/tokmd-cockpit/Cargo.toml` had a hardcoded `path = "../tokmd-analysis", version = "1.11.0"` dependency.

## 🧭 Options considered
### Option A (recommended)
- what it is: Move `source_complexity` out of `tokmd-analysis` into `tokmd-cockpit` and remove the dependency.
- why it fits this repo and shard: It resolves a crate boundary layering violation and completely drops the `tokmd-analysis` crate dependency from `tokmd-cockpit`.
- trade-offs: Structure is improved, dependency tree is simpler.

### Option B
- what it is: Export `source_complexity` from `tokmd_analysis_types`.
- when to choose it instead: If it was truly a shared type definition.
- trade-offs: It violates the Tier 0 definition of `tokmd_analysis_types` which should have no business logic.

## ✅ Decision
Option A was chosen. It fits the Surveyor mission perfectly by fixing a structural coherence problem and respecting the Tier boundaries.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/source_complexity.rs` -> `crates/tokmd-cockpit/src/source_complexity.rs`
- `crates/tokmd-analysis/src/source_complexity/` -> `crates/tokmd-cockpit/src/source_complexity/`
- Updated `crates/tokmd-cockpit/Cargo.toml` to remove the `tokmd-analysis` dependency.
- Updated `crates/tokmd-cockpit/src/gates/complexity.rs` to point to the local `crate::source_complexity::analyze_rust_function_complexity`.
- Updated `crates/tokmd-analysis/src/lib.rs` and `crates/tokmd-cockpit/src/lib.rs` module bindings.

## 🧪 Verification receipts
```text
cargo test --verbose
cargo check
cargo clippy -- -D warnings
cargo fmt -- --check
```

## 🧭 Telemetry
- Change shape: Structural refactor
- Blast radius: dependencies
- Risk class + why: Low, code is simply moved without changes to logic. Tests confirm behavior remains identical.
- Rollback: Revert the PR
- Gates run: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/run_surveyor_workspace_1/envelope.json`
- `.jules/runs/run_surveyor_workspace_1/decision.md`
- `.jules/runs/run_surveyor_workspace_1/receipts.jsonl`
- `.jules/runs/run_surveyor_workspace_1/result.json`
- `.jules/runs/run_surveyor_workspace_1/pr_body.md`

## 🔜 Follow-ups
None
