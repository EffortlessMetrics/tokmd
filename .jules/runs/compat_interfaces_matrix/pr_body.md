## 💡 Summary
Fixed dead code warnings when compiling `tokmd` without default features. The `ExportMetaLite` and `ExportBundle` types and associated load functions were triggering `#[warn(dead_code)]`.

## 🎯 Why
When the `tokmd` crate is built with `--no-default-features`, it triggers a cluster of `#[warn(dead_code)]` warnings on `ExportMetaLite`, `ExportBundle`, and several loading functions in `export_bundle.rs`. Addressing these warnings ensures cleaner builds in restricted toolchain environments or specific feature selections.

## 🔎 Evidence
- `cargo check -p tokmd --no-default-features` produced multiple `dead_code` warnings.
- The functions and types in `crates/tokmd/src/export_bundle.rs` are internal tools that may be bypassed by feature selection.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add `#[allow(dead_code)]` attributes to `ExportMetaLite`, `ExportBundle`, and the parsing functions in `export_bundle.rs`.
- why it fits this repo and shard: It gracefully handles internal dead code without ripping out functionality that may be required by other feature sets.
- trade-offs: Structure is preserved without large refactoring. Velocity is high. Governance expectations are met by quieting the compiler warnings.

### Option B
- what it is: Remove the unused structs and methods.
- when to choose it instead: If the code were truly orphaned across all features.
- trade-offs: Higher risk of breaking active features.

## ✅ Decision
Option A. It's the safest and most robust path for resolving dead code warnings tied to feature flags.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/export_bundle.rs`

## 🧪 Verification receipts
```text
cargo check -p tokmd --no-default-features
cargo test -p tokmd --no-default-features
cargo xtask version-consistency
cargo fmt -- --check
cargo check
```

## 🧭 Telemetry
- Change shape: Added `#[allow(dead_code)]`
- Blast radius: Internal compilation only. No runtime behavioral changes.
- Risk class + why: Very low risk.
- Rollback: Revert changes to `export_bundle.rs`.
- Gates run: `--no-default-features` checks, tests, formatting.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`

## 🔜 Follow-ups
None.
