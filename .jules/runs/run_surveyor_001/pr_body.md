## 💡 Summary
Updated internal dependency declarations in several crates that were bypassing the root workspace definition. This restores boundary hygiene and ensures version consistency.

## 🎯 Why
A few crates (`tokmd-analysis-types`, `tokmd-envelope`, `tokmd-scan`, `tokmd-types`, and `tokmd-wasm`) were explicitly setting `version = ">=1.9, <2"` for internal crates. This breaks workspace inheritance, creates a risk of drift, and constitutes an architectural seam problem.

## 🔎 Evidence
- `crates/tokmd-analysis-types/Cargo.toml`
- `crates/tokmd-envelope/Cargo.toml`
- `crates/tokmd-scan/Cargo.toml`
- `crates/tokmd-types/Cargo.toml`
- `crates/tokmd-wasm/Cargo.toml`
- Output of `grep -n "version = \">=\"" crates/*/Cargo.toml` demonstrating the explicitly bypassed versions.

## 🧭 Options considered
### Option A (recommended)
- Consolidate Dependency Management in Cargo Workspaces to use `workspace = true`.
- Fits the repo/shard because it resolves an explicit feature-boundary hygiene and workspace structure problem.
- Trade-offs: Structure is unified; Velocity improves as bumps happen in one place; Governance aligns with `cargo-workspace` norms.

### Option B
- Deep refactor of `tokmd-node` Async Multi-thread to remove a redundant feature flag.
- Choose when looking for narrow dependency footprint cleanup.
- Trade-offs: Very low blast radius, but smaller structural gain compared to workspace hygiene.

## ✅ Decision
Chose Option A. Unifying the internal dependency versions using `workspace = true` enforces structural coherence and boundary hygiene.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/Cargo.toml`
- `crates/tokmd-envelope/Cargo.toml`
- `crates/tokmd-scan/Cargo.toml`
- `crates/tokmd-types/Cargo.toml`
- `crates/tokmd-wasm/Cargo.toml`

## 🧪 Verification receipts
```text
$ cargo xtask version-consistency
Version consistency checks passed.

$ cargo check --workspace --all-features
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## 🧭 Telemetry
- Change shape: Structural refactor
- Blast radius: Configuration / Dependencies
- Risk class: Low
- Rollback: `git reset HEAD --hard`
- Gates run: core-rust fallbacks, xtask version-consistency

## 🗂️ .jules artifacts
- `.jules/runs/run_surveyor_001/envelope.json`
- `.jules/runs/run_surveyor_001/decision.md`
- `.jules/runs/run_surveyor_001/receipts.jsonl`
- `.jules/runs/run_surveyor_001/result.json`
- `.jules/runs/run_surveyor_001/pr_body.md`
- `.jules/friction/open/FRIC-20240612-001.md`

## 🔜 Follow-ups
- FRIC-20240612-001: Redundant `tokio` feature flag in `tokmd-node`.
