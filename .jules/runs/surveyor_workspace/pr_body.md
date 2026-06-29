## 💡 Summary
Centralized internal workspace dependency versions in `Cargo.toml` manifests. This standardizes all internal crates to point directly to their workspace definitions, avoiding duplicated versions that could cause drift.

## 🎯 Why
Internal workspace dependencies had hardcoded version strings in sub-crate `Cargo.toml` files (e.g. `tokmd-model = { path = "../tokmd-model", version = ">=1.9, <2" }`). This causes version drift. Centralizing them using `workspace = true` enforces consistency through the root metadata. Note that `tokmd-cockpit`'s reference to `tokmd-analysis` uses `default-features = false` and so retains explicit path and version per Cargo rules.

## 🔎 Evidence
- File path: `crates/*/Cargo.toml`
- Finding: Internal dependency references used hardcoded version ranges like `version = ">=1.9, <2"`.
- Receipt: Output of `cargo xtask version-consistency` confirms correctness after centralizing.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `Cargo.toml` to replace explicit dependencies with `.workspace = true`.
- why it fits this repo and shard: Surveyor optimizes for architecture and structure. Centralizing workspace dependency definitions ensures single-source-of-truth and prevents drift.
- trade-offs: Structure improves. Velocity increases because it's easier to bump versions. Governance improves. No significant drawbacks.

### Option B
- what it is: Attempt to change the `tokmd-analysis` / `tokmd-cockpit` boundary.
- when to choose it instead: If the current dependency graph shows true architectural leakage rather than simple metadata debt.
- trade-offs: Higher blast radius, more chance of breaking things.

## ✅ Decision
Chosen Option A. It addresses a clear workspace structure and version-management problem.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/Cargo.toml`
- `crates/tokmd-cockpit/Cargo.toml`
- `crates/tokmd-envelope/Cargo.toml`
- `crates/tokmd-scan/Cargo.toml`
- `crates/tokmd-types/Cargo.toml`
- `crates/tokmd-wasm/Cargo.toml`

## 🧪 Verification receipts
```text
cargo xtask version-consistency
cargo build --verbose
CI=true cargo test -p tokmd-cockpit -p tokmd-envelope -p tokmd-types -p tokmd-scan -p tokmd-wasm -p tokmd-analysis-types
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Metadata updates
- Blast radius: Internal structural refactor, isolated to build tooling and metadata.
- Risk class: Low - build tooling.
- Rollback: Revert to exact hardcoded versions.
- Gates run: `core-rust`

## 🗂️ .jules artifacts
- `.jules/runs/surveyor_workspace/envelope.json`
- `.jules/runs/surveyor_workspace/decision.md`
- `.jules/runs/surveyor_workspace/receipts.jsonl`
- `.jules/runs/surveyor_workspace/result.json`
- `.jules/runs/surveyor_workspace/pr_body.md`

## 🔜 Follow-ups
None.
