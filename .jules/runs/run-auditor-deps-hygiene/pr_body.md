## 💡 Summary
Replaced the hardcoded path and version bound for `tokmd-types` in `tokmd-wasm`'s `dev-dependencies` with a workspace reference. This standardizes the dependency declaration and eliminates a redundant duplicate manifest entry.

/label ci-budget-override

## 🎯 Why
In bindings and targets crates, redundant paths and version bounds for shared dependencies (like `tokmd-types`) create drift risk and violate dependency hygiene. The `tokmd-wasm` crate was explicitly specifying the path `../tokmd-types` and a version bound (`>=1.9, <2`), instead of utilizing the `workspace` property used by the rest of the monorepo. This fixes the anomaly and tightens the dev-dependency to perfectly match the workspace.

## 🔎 Evidence
- File: `crates/tokmd-wasm/Cargo.toml`
- Finding: `tokmd-types = { path = "../tokmd-types", version = ">=1.9, <2" }`
- Replaced with: `tokmd-types.workspace = true`

## 🧭 Options considered
### Option A (recommended)
Update `crates/tokmd-wasm/Cargo.toml` to replace the hardcoded `tokmd-types` path and version with `tokmd-types.workspace = true`.
- **Why it fits**: This exactly hits the persona goal of "remove duplicate or redundant dependency declarations/features" in "bindings and target-specific crates". The workspace already provides the version and path definition for `tokmd-types`. Redundant paths in bindings create drift risk.
- **Trade-offs**:
  - *Structure*: Improves consistency.
  - *Velocity*: Minimal impact, simplifies future upgrades.
  - *Governance*: Better dependency hygiene by ensuring all crates use the exact same definition.

### Option B
Remove `napi-build` from `tokmd-node` as flagged by `cargo-machete`.
- **Why to choose**: If we blindly trusted `cargo-machete` output.
- **Trade-offs**: It would break the Node.js native extension build since `napi_build` is explicitly used in `tokmd-node/build.rs`.

## ✅ Decision
Option A. Updated `crates/tokmd-wasm/Cargo.toml` to inherit `tokmd-types` from the workspace.

## 🧱 Changes made (SRP)
- `crates/tokmd-wasm/Cargo.toml`

## 🧪 Verification receipts
```text
sed -i 's/tokmd-types = { path = "../tokmd-types", version = ">=1.9, <2" }/tokmd-types.workspace = true/' crates/tokmd-wasm/Cargo.toml
cargo check -p tokmd-wasm
cargo fmt -- --check
cargo clippy -p tokmd-wasm -- -D warnings
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner cargo test -p tokmd-wasm --target wasm32-unknown-unknown
cargo build -p tokmd-wasm
```

## 🧭 Telemetry
- Change shape: Dependency tightening
- Blast radius: dependencies, API compatibility within tests for WASM surface
- Risk class: Low - strictly a dependency hygiene and tightening fix that delegates resolution back to root workspace
- Rollback: `git restore crates/tokmd-wasm/Cargo.toml`
- Gates run: `cargo check`, `cargo fmt`, `cargo clippy`, `cargo test (wasm)`

## 🗂️ .jules artifacts
- `.jules/runs/run-auditor-deps-hygiene/envelope.json`
- `.jules/runs/run-auditor-deps-hygiene/decision.md`
- `.jules/runs/run-auditor-deps-hygiene/receipts.jsonl`
- `.jules/runs/run-auditor-deps-hygiene/result.json`
- `.jules/runs/run-auditor-deps-hygiene/pr_body.md`

## 🔜 Follow-ups
None at this time.
