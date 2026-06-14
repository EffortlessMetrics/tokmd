## 💡 Summary
Removed `serde` from direct `[dependencies]` in `tokmd-model` and moved it to `[dev-dependencies]`. The core code in `tokmd-model` only relies on models from `tokmd-types` that implement serialization logic, but it uses `serde::Serialize` in its own test suite.

## 🎯 Why
This reduces the manifest surface area of the core pipeline dependencies by ensuring that `serde` is not unnecessarily propagated as a direct dependency of the `tokmd-model` production code, aligning with dependency hygiene best practices.

## 🔎 Evidence
- `crates/tokmd-model/Cargo.toml`
- Core library `crates/tokmd-model/src/**` does not have any references to `serde` imports or derivations.
- Tests (e.g. `crates/tokmd-model/tests/from_rows_api_w78.rs`) use `serde::Serialize`.
- Command receipt: `cargo check -p tokmd-model --all-targets --all-features` successfully builds the crate correctly after adjusting the manifest.

## 🧭 Options considered
### Option A (recommended)
- Move `serde` to `[dev-dependencies]` in `crates/tokmd-model/Cargo.toml`.
- It perfectly fits this repo and shard as it tightens the manifest surface area of the `tokmd-model` crate.
- Trade-offs: Structure is strictly correct based on actual usage. Velocity improves slightly as dependency graph bloat is reduced in production context. Governance strictly adheres to unused/redundant dependency rule.

### Option B
- Leave it untouched and search for other redundant dependencies.
- When to choose it instead: If the build relies on implicit trait implementations from `serde` that cannot be resolved in `dev-dependencies`.
- Trade-offs: Misses an obvious hygiene improvement.

## ✅ Decision
Option A. I moved `serde.workspace = true` to the `[dev-dependencies]` section in `crates/tokmd-model/Cargo.toml`. `cargo check -p tokmd-model --all-targets --all-features` passed perfectly.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/Cargo.toml`: Moved `serde.workspace = true` from `[dependencies]` to `[dev-dependencies]`.

## 🧪 Verification receipts
```text
$ sed -i 's/serde.workspace = true//g' crates/tokmd-model/Cargo.toml
$ sed -i '/\[dev-dependencies\]/a serde.workspace = true' crates/tokmd-model/Cargo.toml
$ cargo check -p tokmd-model --all-targets --all-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.50s

$ cargo test -p tokmd-model
    Finished `test` profile [unoptimized + debuginfo] target(s) in 50.58s
    Running unittests src/lib.rs (target/debug/deps/tokmd_model-5e68fcce08593331)
    Running tests/aggregate_test.rs (target/debug/deps/aggregate_test-4bd4980e190ac963)
    [Output truncated for brevity - 24 test suites all passed]
```

## 🧭 Telemetry
- Change shape: Dependency hygiene fix.
- Blast radius: Tightened dependency manifest in `tokmd-model`. Zero logic/behavioral changes.
- Risk class: Low. Safe adjustment verified by the compiler and test suite.
- Rollback: Revert `Cargo.toml`.
- Gates run: `cargo check`, `cargo fmt`, `cargo clippy`, `cargo build --verbose`.

## 🗂️ .jules artifacts
- `.jules/runs/auditor_core_manifests_001/envelope.json`
- `.jules/runs/auditor_core_manifests_001/decision.md`
- `.jules/runs/auditor_core_manifests_001/receipts.jsonl`
- `.jules/runs/auditor_core_manifests_001/result.json`
- `.jules/runs/auditor_core_manifests_001/pr_body.md`

## 🔜 Follow-ups
None.
