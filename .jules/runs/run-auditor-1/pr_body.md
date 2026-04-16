## 💡 Summary
Removed the unused direct dependency on `tokmd-analysis-types` from `tokmd-wasm` by re-exporting `ANALYSIS_SCHEMA_VERSION` in `tokmd-core`.

## 🎯 Why
The `tokmd-wasm` crate directly depended on `tokmd-analysis-types` only to read `ANALYSIS_SCHEMA_VERSION`. Because `tokmd-wasm` already relies heavily on `tokmd-core`, we can re-export this constant from `tokmd-core` and simplify the dependency tree for the wasm target, improving dependency hygiene.

## 🔎 Evidence
File: `crates/tokmd-wasm/Cargo.toml` directly declared:
```toml
tokmd-analysis-types = { workspace = true, optional = true }
```
File: `crates/tokmd-wasm/src/lib.rs` imported it directly:
```rust
use tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION;
```
Receipts demonstrate this is cleanly resolvable through `tokmd-core`.

## 🧭 Options considered

### Option A (recommended)
- Remove `tokmd-analysis-types` from `tokmd-wasm`'s `Cargo.toml`.
- Re-export `ANALYSIS_SCHEMA_VERSION` in `tokmd-core`.
- Update `tokmd-wasm/src/lib.rs` to import from `tokmd-core`.
- This tightens the dependency graph in the wasm surface.

### Option B
- Leave the dependency as is.
- No risk of churn, but leaves an unnecessary direct edge in the manifest.

## ✅ Decision
Option A. It's a boring, high-signal cleanup of the bindings/wasm manifest that perfectly aligns with the deps-hygiene gate profile.

## 🧱 Changes made (SRP)
* Modified `crates/tokmd-core/src/lib.rs` to expose `ANALYSIS_SCHEMA_VERSION`.
* Modified `crates/tokmd-wasm/Cargo.toml` to drop `tokmd-analysis-types`.
* Modified `crates/tokmd-wasm/src/lib.rs` to import the schema version from `tokmd-core`.

## 🧪 Verification receipts
```text
$ cargo check -p tokmd-wasm --all-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.60s

$ cd web/runner && npm test
...
# pass 40
# fail 0

$ cargo xtask version-consistency
Checking version consistency against workspace version 1.9.0
  ✓ Cargo crate versions match 1.9.0.
  ✓ Cargo workspace dependency versions match 1.9.0.
  ✓ Node package manifest versions match 1.9.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.
```

## 🧭 Telemetry
- Change shape: Dependency removal and re-export.
- Blast radius: Internal API surface for wasm bindings.
- Risk class: Low, verified by compiler, cargo deny, and wasm runtime tests.
- Rollback: Revert `Cargo.toml` and import paths.
- Gates run: `cargo check`, `cargo test`, `cargo xtask version-consistency`, `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/run-auditor-1/envelope.json`
- `.jules/runs/run-auditor-1/result.json`
- `.jules/runs/run-auditor-1/pr_body.md`

## 🔜 Follow-ups
None.
