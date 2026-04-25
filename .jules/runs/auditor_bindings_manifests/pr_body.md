## 💡 Summary
Removed the unused `pyo3-build-config` build dependency from the `tokmd-python` bindings crate. This tightens the compile surface by dropping an unnecessary crate dependency.

## 🎯 Why
The `tokmd-python` crate did not contain a `build.rs` script, yet declared `pyo3-build-config` as a build-dependency. Removing it cleans up manifest hygiene with zero impact on the final product.

## 🔎 Evidence
- File path: `crates/tokmd-python/Cargo.toml`
- Observed behavior: `ls crates/tokmd-python/build.rs` fails (file does not exist).
- Receipt: `grep -r "pyo3-build-config" crates/tokmd-python/` showed it was only present in `Cargo.toml`.

## 🧭 Options considered
### Option A (recommended)
- Remove `pyo3-build-config` from `crates/tokmd-python/Cargo.toml`.
- Fits the repo/shard by reducing duplicate/unused dependencies.
- Trade-offs: Zero risk to functionality since there is no build script using it.

### Option B
- Investigate removing `js-sys` from `crates/tokmd-wasm/Cargo.toml`.
- When to choose it instead: If the python cleanup was invalid.
- Trade-offs: `js-sys` is actually used in `src/lib.rs` and would require replacing functionality or verifying implicit wasm-bindgen requirements.

## ✅ Decision
Option A was chosen because it's a straightforward, completely risk-free reduction of dependency declarations.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd-python/Cargo.toml` to remove `[build-dependencies]` and `pyo3-build-config`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-python
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

cargo xtask version-consistency
Checking version consistency against workspace version 1.9.0

  ✓ Cargo crate versions match 1.9.0.
  ✓ Cargo workspace dependency versions match 1.9.0.
  ✓ Node package manifest versions match 1.9.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.
```

## 🧭 Telemetry
- Change shape: Dependency removal
- Blast radius: Internal build config for Python bindings only.
- Risk class: Low - build dependency removed where no build script exists.
- Rollback: Revert `Cargo.toml`.
- Gates run: `cargo test -p tokmd-python`, `cargo check --workspace`, `cargo xtask version-consistency`.

## 🗂️ .jules artifacts
- `.jules/runs/auditor_bindings_manifests/envelope.json`
- `.jules/runs/auditor_bindings_manifests/decision.md`
- `.jules/runs/auditor_bindings_manifests/receipts.jsonl`
- `.jules/runs/auditor_bindings_manifests/result.json`
- `.jules/runs/auditor_bindings_manifests/pr_body.md`

## 🔜 Follow-ups
None.
