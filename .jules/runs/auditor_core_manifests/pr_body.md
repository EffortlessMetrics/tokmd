## 💡 Summary
Removed the unused `serde` dependency from `crates/tokmd-model` to improve dependency hygiene within the `core-pipeline` shard.

## 🎯 Why
During a review of the dependency graph and manifest files for the core data and model pipeline, `serde` was identified as an unused direct dependency in `tokmd-model`. Although `tokmd-model` types implement Serde traits, the types themselves and their Serde annotations live in `tokmd-types`. The `tokmd-model` implementation code does not directly use `serde`, and tests only require `serde` to serialize/deserialize outputs using `serde_json`. Cleaning up this dependency tightens the manifest surface and reduces the build graph slightly for this crate.

## 🔎 Evidence
- `crates/tokmd-model/Cargo.toml`
- Searching `crates/tokmd-model/src` for `serde` yielded no results.
- Removing `serde` from `[dependencies]` and running `cargo check -p tokmd-model` was successful. (Note: `serde` was added to `[dev-dependencies]` as it is used by test utility code via `serde_json`).

## 🧭 Options considered
### Option A (recommended)
- Remove `serde` from `[dependencies]` in `tokmd-model` (moving it to `[dev-dependencies]` for test usage).
- Why it fits: Aligns perfectly with the Auditor persona's mandate for boring, high-signal dependency hygiene improvements, specifically targeting "remove an unused direct dependency".
- Trade-offs: Zero velocity or architectural trade-offs. Minor positive governance impact by tightening the dependency surface.

### Option B
- Investigate removing the unused dev-dependencies (`tokmd-scan`, `tokmd-format`, `tokmd-model`) listed in `tokmd-types`.
- When to choose: If they were truly unused. However, they are used by tests in `tokmd-types`, so removing them would break the test suite.
- Trade-offs: Negative impact on velocity as it would require migrating tests.

## ✅ Decision
Option A was selected. Removing the unused `serde` dependency from `tokmd-model` provides a concrete, safe hygiene improvement within the assigned shard.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/Cargo.toml`
  - Removed `serde.workspace = true` from `[dependencies]`.
  - Added `serde.workspace = true` to `[dev-dependencies]` for test-only serialization requirements.

## 🧪 Verification receipts
```text
$ cargo check -p tokmd-model
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s

$ CI=true cargo test -p tokmd-model
   ...
   Doc-tests tokmd_model
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.94s

$ cargo clippy -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 30.69s

$ cargo xtask publish --plan --verbose
=== Publish Plan ===

Workspace version: 1.13.1
...
To execute this plan:
  cargo xtask publish --yes --verbose
```

## 🧭 Telemetry
- Change shape: Manifest update
- Blast radius: Internal crate dependencies only. No API, IO, docs, schema, concurrency, or compatibility risks.
- Risk class: Low risk. Modifying crate dependencies. Build and test gates provide complete confidence.
- Rollback: Revert the `Cargo.toml` change.
- Gates run: `cargo check`, `cargo test`, `cargo fmt`, `cargo clippy`, `cargo build`, `cargo xtask publish --plan`

## 🗂️ .jules artifacts
- `.jules/runs/auditor_core_manifests/envelope.json`
- `.jules/runs/auditor_core_manifests/decision.md`
- `.jules/runs/auditor_core_manifests/receipts.jsonl`
- `.jules/runs/auditor_core_manifests/result.json`
- `.jules/runs/auditor_core_manifests/pr_body.md`

## 🔜 Follow-ups
None.
