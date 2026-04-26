## 💡 Summary
Removed the unused `tokio_rt` feature from the `napi` dependency in `tokmd-node`. The crate already manages its own `tokio` runtime directly, making the N-API global runtime feature redundant and an unnecessary compilation cost.

## 🎯 Why
The `napi` crate was configured with the `tokio_rt` feature, which integrates N-API with a global Tokio runtime. However, `tokmd-node` explicitly manages its own Tokio runtime via `tokio::task::spawn_blocking` and `tokio::runtime::Builder` directly using the `tokio` dependency. The `tokio_rt` feature is unused and redundant, adding unnecessary compilation overhead to the bindings target. Removing it improves dependency hygiene and tightens the feature matrix.

## 🔎 Evidence
- `crates/tokmd-node/Cargo.toml`
- `crates/tokmd-node/src/lib.rs`
- The `tokmd-node` crate compiles and passes tests cleanly after removing the feature.

## 🧭 Options considered
### Option A (recommended)
- Remove the unused `tokio_rt` feature flag from the `napi` dependency.
- **Why it fits:** Reduces compilation surface and aligns with the Auditor persona's goal of tightening features.
- **Trade-offs:**
  - *Structure:* Cleaner dependency graph, explicit runtime management.
  - *Velocity:* Slightly faster build times for the node target.
  - *Governance:* Aligns with the rule to remove unused/redundant features.

### Option B
- Remove other dependencies like `js-sys` or `serde`.
- **When to choose it instead:** If they were actually unused.
- **Trade-offs:** They are heavily used across the bindings and core surfaces. Removing them breaks compilation.

## ✅ Decision
Option A. I removed the `tokio_rt` feature from the `napi` dependency in `crates/tokmd-node/Cargo.toml` as it is demonstrably unused.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Removed `"tokio_rt"` from `napi` features list.

## 🧪 Verification receipts
```text
$ cargo check -p tokmd-node
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

$ cargo test -p tokmd-node
running 21 tests
...
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

## 🧭 Telemetry
- Change shape: Feature flag removal
- Blast radius: `tokmd-node` build configuration only. No API, IO, or schema changes.
- Risk class: Low - validated by successful compilation and test suite.
- Rollback: Re-add `"tokio_rt"` to `napi` features in `crates/tokmd-node/Cargo.toml`.
- Gates run: `cargo check`, `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/auditor_bindings_manifests/envelope.json`
- `.jules/runs/auditor_bindings_manifests/decision.md`
- `.jules/runs/auditor_bindings_manifests/receipts.jsonl`
- `.jules/runs/auditor_bindings_manifests/result.json`
- `.jules/runs/auditor_bindings_manifests/pr_body.md`

## 🔜 Follow-ups
None.
