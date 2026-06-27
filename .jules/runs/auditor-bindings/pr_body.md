## 💡 Summary
Removed the explicit multi-threaded `tokio` dependency from `tokmd-node` in favor of using `napi::tokio`. This simplifies the build graph and prevents creating parallel, unmanaged runtimes in Node.js addons.

## 🎯 Why
In Node.js native addons using `napi-rs` (e.g., `crates/tokmd-node`), declaring an explicit multi-threaded `tokio` dependency is an anti-pattern. Instead, we should use the managed `tokio` runtime provided by the host environment via `napi::tokio` (when the `napi` crate's `async` feature is enabled) to prevent parallel unmanaged runtimes and redundant dependencies.

## 🔎 Evidence
- `crates/tokmd-node/Cargo.toml` explicitly depended on `tokio = { version = "1", features = ["rt-multi-thread"] }`.
- `crates/tokmd-node/src/lib.rs` instantiated its own runtime with `tokio::runtime::Builder::new_multi_thread()`.
- Tests pass after migrating to `napi::tokio`.

## 🧭 Options considered
### Option A (recommended)
- Remove the explicit `tokio` dependency in `crates/tokmd-node` and instead use the managed `tokio` runtime provided by the host environment via `napi::tokio` (when the `napi` crate's `async` feature is enabled).
- **Why it fits:** This aligns directly with the `deps-hygiene` gate profile and the Auditor persona mission.
- **Trade-offs:**
  - Structure: Improves consistency and prevents multi-runtime collisions in Node environments.
  - Velocity: High; the change is straightforward and local.
  - Governance: High signal for the Auditor role; removes a duplicate dependency footprint.

### Option B
- Tighten `napi` features to reduce compile time.
- **When to choose it:** If removing the entire explicit tokio dependency isn't possible.
- **Trade-offs:** Requires deep analysis of napi usage, and doesn't solve the core issue of multiple tokio runtimes.

## ✅ Decision
Option A. Removed the explicit `tokio` dependency and switched to using the `napi::tokio` re-export, which natively provides managed async functionality inside napi contexts.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Removed explicit `tokio` dependency.
- `crates/tokmd-node/src/lib.rs`: Swapped `tokio::` paths to `napi::tokio::`.

## 🧪 Verification receipts
```text
$ cd crates/tokmd-node && CI=true cargo test
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cd crates/tokmd-node && cargo fmt -- --check && cargo clippy -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## 🧭 Telemetry
- Change shape: Dependency removal and code migration
- Blast radius: API/dependencies (Node.js target only)
- Risk class: Low (covered by napi bindings tests)
- Rollback: Revert `Cargo.toml` and `lib.rs`
- Gates run: `cargo build`, `CI=true cargo test`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/auditor-bindings/envelope.json`
- `.jules/runs/auditor-bindings/decision.md`
- `.jules/runs/auditor-bindings/receipts.jsonl`
- `.jules/runs/auditor-bindings/result.json`
- `.jules/runs/auditor-bindings/pr_body.md`

## 🔜 Follow-ups
None.
