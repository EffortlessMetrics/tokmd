## 💡 Summary
Improved path redaction and normalization logic to properly handle parent directory segments (`..`). The fix prevents directory structure leakage and ensures logically identical paths with parent traversals yield deterministic hashes.

## 🎯 Why
As highlighted by memory, path redaction and normalization logic (`clean_path`, `normalize_path`, `normalize_scan_input`) must correctly resolve `..` segments and unify path separators so that logically identical paths hash deterministically, preventing directory structure leakage.

## 🔎 Evidence
- `crates/tokmd-format/src/redact/mod.rs` (clean_path)
- `crates/tokmd-format/src/scan_args/mod.rs` (normalize_rel_path)
- `crates/tokmd-model/src/lib.rs` (normalize_path)
Path traversal vectors (`foo/../bar`) historically leaked layout info when hashed unmodified. Tested locally: `normalize_path(foo/../bar)` previously left `foo/../bar`. It now reduces to `bar`.

## 🧭 Options considered
### Option A (recommended)
- Resolve parent directory segments correctly by maintaining a part stack and popping on `..`.
- Fits the repo and shard: Directly hardens the deterministic input pipeline around config and parser interfaces.
- Trade-offs: Structure is improved, velocity hit is small, governance hygiene is strictly adhered to.

### Option B
- Only strip `..` blindly without checking parents.
- Choose when pure simplicity is required.
- Trade-offs: Unsound for deep traversals, can lead to invalid filesystem structures.

## ✅ Decision
Option A was chosen. It securely hardens the input redaction and path normalization routines while meeting the strict constraints.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`: Added `..` resolution loop and regression test to `clean_path`.
- `crates/tokmd-format/src/scan_args/mod.rs`: Added `..` resolution loop and regression test to `normalize_rel_path`.
- `crates/tokmd-model/src/lib.rs`: Added `..` resolution loop and regression test to `normalize_path`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-format normalize_scan_input (success)
cargo test -p tokmd-format clean_path (success)
cargo test -p tokmd-model normalize_path (success)
```

## 🧭 Telemetry
- Change shape: Logic updates in path string processing loops.
- Blast radius: Output metrics paths, analysis path inputs.
- Risk class: Low - deterministic behavior now correctly identifies the terminal file paths.
- Rollback: Revert the `Vec<&str>` stack logic.
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/run-001/envelope.json`
- `.jules/runs/run-001/decision.md`
- `.jules/runs/run-001/receipts.jsonl`
- `.jules/runs/run-001/result.json`
- `.jules/runs/run-001/pr_body.md`

## 🔜 Follow-ups
None
