## 💡 Summary
Removed the `js` feature from the `uuid` dependency in `crates/tokmd-format/Cargo.toml`. This drops unnecessary WebAssembly/JavaScript transitive dependencies (`wasm-bindgen`, `js-sys`) from the native compile surface.

## 🎯 Why
The `js` feature flag on `uuid` is only required when compiling for WASM/JS targets. Including it in a native core crate like `tokmd-format` unnecessarily inflates the dependency tree with WASM-specific transitive dependencies (`wasm-bindgen`, `js-sys`). As documented in the provided memory: "In Rust native applications or CLIs (like those in `tokmd`), avoid enabling the `js` feature on the `uuid` dependency". Removing this feature flag reduces the native compile surface without affecting functionality, completely aligning with the Auditor persona's goal to tighten feature flags.

## 🔎 Evidence
File path: `crates/tokmd-format/Cargo.toml`
Observation:
Before change: `uuid = { version = "1.22", features = ["v4", "js"] }`

We observed the feature flag enabled via `cargo tree` and `grep`:
```text
$ grep -Hn "uuid" crates/tokmd-*/Cargo.toml
crates/tokmd-format/Cargo.toml:20:uuid = { version = "1.22", features = ["v4", "js"] }

$ cargo tree -e features -p tokmd-format
├── uuid feature "js"
│   └── uuid v1.22.0 (*)
```

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Remove the `js` feature from the `uuid` dependency in `crates/tokmd-format/Cargo.toml`.
- **Why it fits this repo and shard:** The `tokmd-format` crate is part of the `core-pipeline` shard. Removing this feature aligns strictly with dependency hygiene goals (target ranking #3: tighten feature flags to reduce compile surface) and project memory constraints for native applications.
- **Trade-offs:**
  - *Structure:* Cleaner dependency graph for native targets.
  - *Velocity:* Slightly faster builds due to reduced transitive dependencies.
  - *Governance:* Reduces risk of unintended JS/WASM ecosystem dependencies leaking into a native core crate.

### Option B
- **What it is:** Create a PR to remove unused `tempfile` dependencies.
- **When to choose it instead:** If there was a clear unused dependency across multiple crates that `cargo machete` or manual review confirmed was entirely unused.
- **Trade-offs:** `cargo machete` did not find any unused direct dependencies. Finding unused dependencies manually could be noisy.

## ✅ Decision
I chose **Option A**. The memory specifically calls out the `js` feature on the `uuid` dependency as an anti-pattern for this project's native compilation surface. It is a clean, boring, high-signal dependency hygiene improvement.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/Cargo.toml`: Removed `js` from the features list for `uuid`.

## 🧪 Verification receipts
```text
$ sed -i 's/"v4", "js"/"v4"/g' crates/tokmd-format/Cargo.toml

$ cargo check -p tokmd-format
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

$ cargo clippy -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.49s

$ cargo test -p tokmd-format
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo xtask version-consistency
Version consistency checks passed.
```

## 🧭 Telemetry
- Change shape: Feature flag reduction.
- Blast radius: Internal API / Dependencies. No external IO, schema, concurrency, or docs changes.
- Risk class: Low - it only affects the dependency graph compilation and the code does not rely on `js` specific `uuid` behavior.
- Rollback: Revert `crates/tokmd-format/Cargo.toml`.
- Gates run: `cargo check`, `cargo test`, `cargo clippy`, `cargo xtask version-consistency`, `cargo xtask publish --plan`, `cargo fmt`.

## 🗂️ .jules artifacts
- `.jules/runs/run-auditor-core-manifests/envelope.json`
- `.jules/runs/run-auditor-core-manifests/decision.md`
- `.jules/runs/run-auditor-core-manifests/receipts.jsonl`
- `.jules/runs/run-auditor-core-manifests/result.json`
- `.jules/runs/run-auditor-core-manifests/pr_body.md`

## 🔜 Follow-ups
None.
