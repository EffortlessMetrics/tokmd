## 💡 Summary
Removed the redundant `rt-multi-thread` feature from the `tokio` dependency in `tokmd-node`'s `Cargo.toml`.

## 🎯 Why
In `napi-rs` bindings, the `napi` dependency's `async` feature automatically pulls in `tokio` with the `rt-multi-thread` feature via `tokio_rt`. Explicitly requesting `features = ["rt-multi-thread"]` on a direct `tokio` dependency is redundant and unnecessarily constrains the build matrix, potentially causing cross-platform compatibility issues where multi-threading may not be available or preferred.

## 🔎 Evidence
- `crates/tokmd-node/Cargo.toml`
- Found: `tokio = { version = "1", features = ["rt-multi-thread"] }`
- Memory explicitly warned against this practice: "In `napi-rs` bindings (e.g., `tokmd-node`), the `napi` dependency's `async` feature automatically pulls in `tokio` with the `rt-multi-thread` feature via `tokio_rt`. Explicitly requesting `features = ["rt-multi-thread"]` on a direct `tokio` dependency is redundant and can be safely tightened to `tokio = "1"`."

## 🧭 Options considered
### Option A (recommended)
- Remove the `"rt-multi-thread"` feature constraint from the `tokio` dependency in `tokmd-node`.
- Why it fits: It adheres to the target rule of addressing binding feature interactions, resolving an over-constrained dependency that `napi-rs` already safely manages dynamically.
- Trade-offs:
  - Structure: High. Fixes an over-constrained dependency boundary.
  - Velocity: Neutral.
  - Governance: High. Reduces dependency feature bloat.

### Option B
- Keep the explicit `"rt-multi-thread"` feature and create a learning PR documenting the redundancy.
- When to choose: If the removal causes build or test failures in `tokmd-node`.
- Trade-offs: Leaves redundant configuration in place despite known best practices for `napi-rs` modules.

## ✅ Decision
Option A. It directly addresses a known feature interaction issue in `tokmd-node` binding dependencies. It correctly applies the `napi-rs` best practices and aligns with the persona's goal to fix binding feature interactions.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Adjusted `tokio` dependency configuration to remove `features = ["rt-multi-thread"]`.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-node --no-default-features
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

$ cargo test -p tokmd-node --all-features
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

$ cargo check -p tokmd-node --all-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

$ cargo fmt -- --check

$ cargo clippy -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.94s
```

## 🧭 Telemetry
- Change shape: Dependency tightening
- Blast radius: `tokmd-node` bindings
- Risk class: Low
- Rollback: Revert `Cargo.toml`
- Gates run: `compat-matrix` (`cargo test --all-features`, `cargo test --no-default-features`, `cargo build`, `cargo clippy`, `cargo fmt`)

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None.
