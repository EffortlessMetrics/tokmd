## 💡 Summary
Removed the `js` feature from the `uuid` dependency in `tokmd-format`. This tightens the feature flags to reduce the compile surface and avoids unnecessary WebAssembly/JavaScript transitive dependencies for the native build. Re-added `js` feature selectively for `wasm32` to fix CI builds.

## 🎯 Why
The `js` feature on the `uuid` dependency is intended for WebAssembly environments to bind to JS APIs (like `crypto.getRandomValues`). Native applications and CLIs (which `tokmd-format` is part of) do not need this feature, and leaving it enabled introduces unnecessary transitive dependencies to the native compile graph. For `wasm32-unknown-unknown` target, `js` feature is strictly required for generating v4 UUIDs since there is no native randomness source available without it.

## 🔎 Evidence
- `crates/tokmd-format/Cargo.toml` previously had `uuid = { version = "1.22", features = ["v4", "js"] }`.
- Cargo test run: `cargo test -p tokmd-format` executed 302 tests seamlessly without the `js` feature, confirming its redundancy for native targets.
- WASM CI tests failed without it but pass again when `uuid` with `js` feature is added conditionally for `wasm32`.

## 🧭 Options considered
### Option A (recommended)
- Remove `js` from `uuid` features in `tokmd-format`, and re-add conditionally for `wasm32`.
- Why it fits this repo and shard: It directly targets manifest dependency hygiene within the `core-pipeline` shard.
- Trade-offs: Structure is improved by isolation; Velocity unaffected; Governance improved by lowering surface area natively.

### Option B
- Ignore this dependency and seek others.
- When to choose: If the crate were exclusively targeting WASM.
- Trade-offs: Retains a known anti-pattern and unnecessary dependencies.

## ✅ Decision
Option A was selected. It implements a concrete dependency constraint tightening natively while keeping the WASM target working seamlessly.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/Cargo.toml`: Removed the `js` feature from `uuid` dependency under general `[dependencies]` and added it exclusively under `[target.'cfg(target_arch = "wasm32")'.dependencies]`.

## 🧪 Verification receipts
```text
cargo check -p tokmd-format
> Success

cargo check -p tokmd-wasm --target wasm32-unknown-unknown
> Success

cargo test -p tokmd-format
> ok. 23 passed (format_tests)
> ok. 35 passed (snapshot_deep)
> ok. 34 passed (snapshots)
> ... Total 302 tests passed natively

cargo clippy -p tokmd-format -- -D warnings
> Success
```

## 🧭 Telemetry
- Change shape: Manifest modification
- Blast radius: dependencies
- Risk class + why: low, native UUID generation does not use the JS APIs, and Wasm is configured explicitly to use it. Verified by the test suite pass.
- Rollback: Revert `crates/tokmd-format/Cargo.toml`
- Gates run: `cargo check`, `cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/run-auditor-core-manifests/envelope.json`
- `.jules/runs/run-auditor-core-manifests/decision.md`
- `.jules/runs/run-auditor-core-manifests/receipts.jsonl`
- `.jules/runs/run-auditor-core-manifests/result.json`
- `.jules/runs/run-auditor-core-manifests/pr_body.md`

## 🔜 Follow-ups
None.
