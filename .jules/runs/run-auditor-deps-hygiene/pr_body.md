## 💡 Summary
This is a learning PR. Attempted to standardize the `tokmd-types` dependency in `crates/tokmd-wasm/Cargo.toml` to use the workspace definition, but was blocked by the CI LEM budget ceiling.

## 🎯 Why
During a dependency hygiene sweep of the bindings targets, I identified that `tokmd-wasm` manually defined the path and version for `tokmd-types` instead of using `workspace = true`. Removing this redundancy is a standard Auditor improvement. However, modifying `crates/tokmd-wasm/Cargo.toml` causes `ci-plan` to estimate a 139 LEM cost (based on historical actuals + static WASM build cost), which exceeds the 125 LEM hard ceiling. Because autonomous agents cannot apply the `ci-budget-override` GitHub label, the trivial fix is hard-blocked.

## 🔎 Evidence
- File: `crates/tokmd-wasm/Cargo.toml`
- Observed behavior: `ci-plan` rejects the change with `PR plan estimated 139 LEM (>125 hard ceiling)`.
- Command run: `cargo xtask ci-plan --base origin/main --head HEAD --labels-json '[]' ...` failed with the override requirement.

## 🧭 Options considered
### Option A (recommended)
Revert the Cargo.toml change and submit a learning PR highlighting the friction.
- What it is: Do not force a fake fix; instead, document the structural friction preventing small hygiene fixes in WASM.
- Why it fits this repo: Honors the hard constraint to avoid blocking and not to force a fix when legitimately blocked.
- Trade-offs:
  - Structure: Leaves the redundant manifest entry in place for now.
  - Velocity: Avoids CI failures and highlights a process improvement.
  - Governance: Follows agent protocol by recording a friction item.

### Option B
Remove `napi-build` from `tokmd-node`'s `build-dependencies`.
- What it is: Follow `cargo-machete`'s hint that `napi-build` is unused.
- When to choose it instead: If it were actually true.
- Trade-offs: This would break the Node.js native extension build since `napi-build` is explicitly invoked in `crates/tokmd-node/build.rs`.

## ✅ Decision
Option A. I reverted the patch to `crates/tokmd-wasm/Cargo.toml` and am submitting this learning PR with a friction item detailing the CI budget constraint on leaf crates.

## 🧱 Changes made (SRP)
- (None - learning PR only)

## 🧪 Verification receipts
```text
cargo check -p tokmd-wasm
cargo fmt -- --check
cargo clippy -p tokmd-wasm -- -D warnings
```

## 🧭 Telemetry
- Change shape: Learning PR / Friction Recording
- Blast radius: None
- Risk class: Low
- Rollback: None needed
- Gates run: `cargo check`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/run-auditor-deps-hygiene/envelope.json`
- `.jules/runs/run-auditor-deps-hygiene/decision.md`
- `.jules/runs/run-auditor-deps-hygiene/receipts.jsonl`
- `.jules/runs/run-auditor-deps-hygiene/result.json`
- `.jules/runs/run-auditor-deps-hygiene/pr_body.md`
- `.jules/friction/open/ci-budget-blocks-trivial-wasm-manifest-fix.md`

## 🔜 Follow-ups
See `.jules/friction/open/ci-budget-blocks-trivial-wasm-manifest-fix.md` regarding the CI budget ceiling.
