## 💡 Summary
Removed the unnecessary `js` feature from the `uuid` dependency in `tokmd-format`. This cleanly drops JavaScript/WASM specific transitive dependencies (like `wasm-bindgen` and `js-sys`) from the native compile graph.

## 🎯 Why
The `tokmd-format` crate uses `uuid` to generate `v4` identifiers for some JSON export formats (specifically CycloneDX). Enabling the `js` feature natively provides WASM random number generator fallbacks, but introduces unused transitive dependencies when building natively, violating dependency hygiene goals.

## 🔎 Evidence
- File: `crates/tokmd-format/Cargo.toml`
- Issue: `uuid = { version = "1.22", features = ["v4", "js"] }`

## 🧭 Options considered
### Option A (recommended)
- What it is: Remove the `js` feature from `uuid` in `tokmd-format`.
- Why it fits this repo and shard: Directly tightens feature flags to reduce compile surface for a core crate.
- Trade-offs:
  - Structure: Minor, safe Cargo manifest change.
  - Velocity: Avoids pulling in `wasm-bindgen` build steps where not needed.
  - Governance: High alignment with hygiene guidelines.

### Option B
- What it is: Move `tempfile` to `[dev-dependencies]` in `tokmd-scan`.
- When to choose it instead: If `tempfile` was actually unused at runtime.
- Trade-offs: However, `tempfile` is actually used in `tokmd-scan` runtime code to hold mocked scans, so it cannot be removed from direct dependencies.

## ✅ Decision
Implemented Option A. It correctly and safely reduces dependency surface area without breaking functionality.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/Cargo.toml`: Changed `uuid` features from `["v4", "js"]` to `["v4"]`.

## 🧪 Verification receipts
```text
$ sed -i 's/features = \["v4", "js"\]/features = \["v4"\]/' crates/tokmd-format/Cargo.toml
$ cargo test -p tokmd-format
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## 🧭 Telemetry
- Change shape: Dependency hygiene, manifest tightening.
- Blast radius: Compilation only.
- Risk class + why: Low. Removing an unneeded WASM generator feature has no effect on native RNG.
- Rollback: Re-add the feature to Cargo.toml.
- Gates run: `cargo build -p tokmd-format`, `cargo test -p tokmd-format`.

## 🗂️ .jules artifacts
- `.jules/runs/auditor_core_manifests_1/envelope.json`
- `.jules/runs/auditor_core_manifests_1/decision.md`
- `.jules/runs/auditor_core_manifests_1/receipts.jsonl`
- `.jules/runs/auditor_core_manifests_1/result.json`
- `.jules/runs/auditor_core_manifests_1/pr_body.md`

## 🔜 Follow-ups
None.
