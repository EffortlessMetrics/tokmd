## 💡 Summary
Removed the redundant multi-threaded `tokio` runtime dependency from the `tokmd-node` bindings crate. The crate now cleanly delegates blocking tasks to `napi::tokio`, aligning properly with the Node.js native addon execution model.

## 🎯 Why
The `tokmd-node` crate previously declared an explicit dependency on `tokio` with the `rt-multi-thread` feature. Since it's a native Node.js addon building via `napi-rs`, pulling in a standalone multi-threaded async runtime is a significant anti-pattern. Node.js extensions should rely on the host's event loop or the managed `napi::tokio` integration. Removing this direct dependency eliminates duplicate declarations, tightens the feature graph (dropping `rt-multi-thread`), and ensures we don't accidentally spin up parallel unmanaged runtimes.

## 🔎 Evidence
- `crates/tokmd-node/Cargo.toml`
- `crates/tokmd-node/src/lib.rs`
- Cargo tree confirmed the direct `tokio` dependency with `rt-multi-thread`:
```text
├── tokio feature "default" (*)
├── tokio feature "rt-multi-thread" (*)
```

## 🧭 Options considered
### Option A (recommended)
- Remove `tokio` dependency completely from `Cargo.toml` and use `napi::tokio::task::spawn_blocking` and `napi::tokio::runtime::Builder` directly.
- why it fits this repo and shard: It removes redundant code from the bindings-targets shard while tightening manifest dependencies, fitting exactly what the Auditor persona requires.
- trade-offs: Structure / Velocity / Governance: Improves structure by matching native execution boundaries. Very minor velocity gain locally. Perfect fit for governance rules against duplicate/bloated manifests.

### Option B
- Keep `tokio` but strip the `rt-multi-thread` feature flag.
- when to choose it instead: If `napi` didn't re-export its own `tokio` wrapper and we just needed standard threading.
- trade-offs: Leaves a duplicative explicit dependency in `Cargo.toml`.

## ✅ Decision
Implemented Option A because it fully removes an unnecessary top-level explicit dependency and prevents conflicting runtime declarations.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Removed the direct `tokio` dependency.
- `crates/tokmd-node/src/lib.rs`: Switched `tokio::task::spawn_blocking` and `tokio::runtime::Builder` to `napi::tokio::...`.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-node
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

## 🧭 Telemetry
- Change shape: Dependency reduction / Code update
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Dependencies (napi-internal tokio is used now instead of explicitly building rt-multi-thread)
- Risk class + why: Low risk. We are using the equivalent `napi::tokio` functionality inside the exact same `run_blocking` wrappers, guaranteeing the thread pool behavior is NAPI-approved.
- Rollback: Revert the PR
- Gates run: `cargo build`, `cargo test -p tokmd-node`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo xtask version-consistency`

## 🗂️ .jules artifacts
- `.jules/runs/auditor_bindings_manifests/envelope.json`
- `.jules/runs/auditor_bindings_manifests/decision.md`
- `.jules/runs/auditor_bindings_manifests/receipts.jsonl`
- `.jules/runs/auditor_bindings_manifests/result.json`
- `.jules/runs/auditor_bindings_manifests/pr_body.md`

## 🔜 Follow-ups
None.
