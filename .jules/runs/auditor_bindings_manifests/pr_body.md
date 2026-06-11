## 💡 Summary
Tightened explicit feature requests and cleaned up redundant dependency declarations in the Node and Wasm bindings crates.

## 🎯 Why
Dependency declarations should be as tight and boring as possible to minimize compile surface and keep manifests readable. `tokmd-wasm` duplicated version strings instead of relying on workspace inheritance, and `tokmd-node` explicitly requested a multi-threaded Tokio runtime it did not strictly require directly.

## 🔎 Evidence
- `crates/tokmd-wasm/Cargo.toml` had a path and version specifier for `tokmd-types` despite workspace configuration.
- `crates/tokmd-node/Cargo.toml` explicitly requested `rt-multi-thread` for `tokio`, but compiling and running tests against `tokio="1"` succeeds since the feature is not directly required by its own code.

## 🧭 Options considered
### Option A (recommended)
- Consolidate Wasm dependencies to `workspace = true` and drop the unneeded `tokio` feature flag in Node.
- Trade-offs: Structure (cleaner manifests), Velocity (no regressions), Governance (aligns with `deps-hygiene` rules).

### Option B
- Ignore them because other transitive dependencies might pull them anyway.
- Trade-offs: Leaves messy manifests behind and violates dependency hygiene guidelines.

## ✅ Decision
Option A. Keeps it boring, tightens local constraints, and removes redundancy.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Replaced `tokio = { version = "1", features = ["rt-multi-thread"] }` with `tokio = "1"`.
- `crates/tokmd-wasm/Cargo.toml`: Replaced explicit `tokmd-types` path/version dependency with `workspace = true`.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-node
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

$ cargo test -p tokmd-wasm
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: Manifest cleanups
- Blast radius: Compilation/Dependencies
- Risk class: Low - tests verify the bindings still compile and behave correctly.
- Rollback: Revert Cargo.toml
- Gates run: targeted cargo test

## 🗂️ .jules artifacts
- `.jules/runs/auditor_bindings_manifests/envelope.json`
- `.jules/runs/auditor_bindings_manifests/decision.md`
- `.jules/runs/auditor_bindings_manifests/receipts.jsonl`
- `.jules/runs/auditor_bindings_manifests/result.json`
- `.jules/runs/auditor_bindings_manifests/pr_body.md`

## 🔜 Follow-ups
None.
