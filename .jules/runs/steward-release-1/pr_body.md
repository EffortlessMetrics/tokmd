## 💡 Summary
Optimizes string allocations in `xtask` version consistency checks and removes an unused build dependency from `tokmd-python`.

## 🎯 Why
The `xtask` tool's case-insensitive collision checker unconditionally allocated new `String` instances with `.to_lowercase()` for `BTreeMap` lookups, which was inefficient. Additionally, `tokmd-python` declared `pyo3-build-config` as a build-dependency but has no `build.rs` script, violating dependency hygiene.

## 🔎 Evidence
- `xtask/src/tasks/version_consistency.rs` was using `.entry(path.to_lowercase()).or_default().push(path)`.
- `crates/tokmd-python/Cargo.toml` had `pyo3-build-config = "0.28.3"` under `[build-dependencies]`, but `ls -l crates/tokmd-python/build.rs` showed no such file.
- `cargo test -p xtask` and `cargo xtask version-consistency` passed after changes.

## 🧭 Options considered
### Option A (recommended)
- Fix `xtask` allocations using `Cow` and remove `pyo3-build-config` from `tokmd-python`.
- Fits the `tooling-governance` shard by improving release check performance and cleaning up manifest metadata.
- Trade-offs: Minor logic restructuring in `xtask`.

### Option B
- Only remove the unused build dependency.
- Could choose if the `xtask` change was deemed too complex.
- Trade-offs: Leaves known allocation inefficiencies in release tooling.

## ✅ Decision
Option A was chosen to improve both performance and dependency hygiene in a single coherent release-safety patch.

## 🧱 Changes made (SRP)
- `xtask/src/tasks/version_consistency.rs`: Replaced `.entry(path.to_lowercase()).or_default().push(path)` with `Cow<'_, str>` based lookups and inserts to avoid unnecessary allocations.
- `crates/tokmd-python/Cargo.toml`: Removed the unused `pyo3-build-config` from `[build-dependencies]`.

## 🧪 Verification receipts
```text
cargo test -p xtask
test result: ok. 156 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.2s

cargo xtask version-consistency
Checking version consistency against workspace version 1.9.0
  ✓ Cargo crate versions match 1.9.0.
  ✓ Cargo workspace dependency versions match 1.9.0.
  ✓ Node package manifest versions match 1.9.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

cargo clippy -p xtask -- -D warnings
cargo clippy -p tokmd-python -- -D warnings
cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Optimization + Manifest Cleanup
- Blast radius: `xtask` tooling and `tokmd-python` build metadata
- Risk class: Low - `xtask` logic is thoroughly unit tested, and removing unused build-deps is structurally safe.
- Rollback: Revert the PR
- Gates run: `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo test -p xtask`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/steward-release-1/envelope.json`
- `.jules/runs/steward-release-1/decision.md`
- `.jules/runs/steward-release-1/receipts.jsonl`
- `.jules/runs/steward-release-1/result.json`
- `.jules/runs/steward-release-1/pr_body.md`

## 🔜 Follow-ups
None.
