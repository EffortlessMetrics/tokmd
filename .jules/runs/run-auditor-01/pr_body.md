## 💡 Summary
Replaced the unmanaged standalone `tokio` multi-thread runtime in the `tokmd-node` native addon with the managed runtime provided by the Node.js host environment. Moved `tokio` to `[dev-dependencies]` to support integration tests.

## 🎯 Why
Native Node.js addons using `napi-rs` should not spin up their own unmanaged multi-threaded `tokio` runtimes when the `async` feature is enabled. The `napi` crate provides a host-managed runtime via `napi::tokio` that integrates properly with the Node.js event loop and prevents rogue thread pools and redundant dependency resolution.

## 🔎 Evidence
- `crates/tokmd-node/Cargo.toml` previously imported `tokio` explicitly with the `rt-multi-thread` feature for use in the addon.
- `crates/tokmd-node/src/lib.rs` explicitly used `tokio::task::spawn_blocking` instead of `napi::tokio::task::spawn_blocking`.
- `cargo tree -e features -p tokmd-node` confirmed that the explicit `tokio` dependency and its features were included in the addon build.

## 🧭 Options considered
### Option A (recommended)
- Remove the explicit `tokio` dependency for the addon, enable the `tokio_rt` feature in `napi`, use `napi::tokio::task::spawn_blocking` in source, and move `tokio` to `dev-dependencies`.
- Fits the memory directive explicitly instructing us to use `napi::tokio` in `crates/tokmd-node` to avoid parallel unmanaged runtimes.
- Trade-offs: Structure improves by relying on the host-managed pool. Governance improves by reducing the production dependency tree.

### Option B
- Investigate `proptest` removal in `tokmd-python` or `wasm-bindgen-test` cleanups in `tokmd-wasm`.
- When to choose: If fixing the `tokmd-node` runtime caused untractable test failures.
- Trade-offs: Lower impact than removing an unmanaged runtime from a native module.

## ✅ Decision
Option A. It explicitly aligns with the persona mission, improves dependency hygiene, and fixes a real structural/runtime integration flaw in the native bindings.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Replaced `tokio` dependency with `napi` `tokio_rt` feature; moved `tokio` to `[dev-dependencies]`.
- `crates/tokmd-node/src/lib.rs`: Changed `tokio::task::spawn_blocking` to `napi::tokio::task::spawn_blocking`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-node
...
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

cargo check -p tokmd-node
...
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## 🧭 Telemetry
- Change shape: Dependency hygiene improvement and correct host runtime integration.
- Blast radius: Node.js bindings (`tokmd-node`).
- Risk class: Low. The runtime was correctly transitioned, and all integration tests continue to pass.
- Rollback: Revert `Cargo.toml` and `src/lib.rs` to explicit `tokio` multi-thread imports.
- Gates run: `cargo build --verbose`, `cargo test -p tokmd-node`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo deny --all-features check`.

## 🗂️ .jules artifacts
- `.jules/runs/run-auditor-01/envelope.json`
- `.jules/runs/run-auditor-01/decision.md`
- `.jules/runs/run-auditor-01/receipts.jsonl`
- `.jules/runs/run-auditor-01/result.json`
- `.jules/runs/run-auditor-01/pr_body.md`

## 🔜 Follow-ups
None.
