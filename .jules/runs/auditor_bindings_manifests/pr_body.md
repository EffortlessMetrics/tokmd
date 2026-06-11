## 💡 Summary
Tightened explicit `tokio` feature requests in the Node bindings crate.

## 🎯 Why
Dependency declarations should be as tight and boring as possible to minimize explicit compilation surfaces. `tokmd-node` explicitly requested a multi-threaded Tokio runtime (`rt-multi-thread`) that it did not strictly require to build and pass its tests natively, introducing unneeded explicit compilation bloat constraints in the manifest.

## 🔎 Evidence
- `crates/tokmd-node/Cargo.toml` explicitly requested `rt-multi-thread` for `tokio`.
- Compiling and running tests against `tokio="1"` succeeds without regressions because the feature is not strictly directly required by its own code natively.

## 🧭 Options considered
### Option A (recommended)
- Drop the unneeded `tokio` feature flag in Node manifest.
- Trade-offs: Structure (cleaner manifests), Velocity (fits within CI budget of 125 LEM), Governance (aligns with `deps-hygiene` rules).

### Option B
- Modify additional Wasm manifests.
- Trade-offs: Triggers excessive CI risk packs exceeding the 125 LEM hard limit.

## ✅ Decision
Option A. Keeps it boring, tightens local constraints, and fits within PR budget constraints.

## 🧱 Changes made (SRP)
- `crates/tokmd-node/Cargo.toml`: Replaced `tokio = { version = "1", features = ["rt-multi-thread"] }` with `tokio = "1"`.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-node
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
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
