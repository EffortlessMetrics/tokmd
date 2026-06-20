## 💡 Summary
Fixed misleading and broken doctests in `tokmd`'s CLI config resolution logic (`export`, `lang`, and `module`). Corrected path scopes and import collisions so the documented examples are factually accurate and executable.

## 🎯 Why
The doctests inside `crates/tokmd/src/config/resolve/` referenced fake import paths like `tokmd::cli::Profile` and missing root functions like `tokmd::resolve_export_with_config`. Because they were out of sync with the actual crate structure, they served as a misleading reference for developers. Fixing them ensures `docs-executable` gate guarantees and acts as a better developer proof point.

## 🔎 Evidence
File paths:
- `crates/tokmd/src/config/resolve/export.rs`
- `crates/tokmd/src/config/resolve/lang.rs`
- `crates/tokmd/src/config/resolve/module.rs`

Observed behavior:
- Missing `tokmd_settings::Profile` import, aliasing instead to an invalid path.
- `resolve_*_with_config` functions were mapped from the crate root instead of `tokmd::config::*`.
- `ExportFormat` collision between `tokmd::cli::ExportFormat` and `tokmd_types::ExportFormat`.

Receipts:
`cargo test -p tokmd --doc`

## 🧭 Options considered
### Option A (recommended)
- Fix the `crates/tokmd/src/config/resolve/` doctests to import correct types.
- Fixes `use tokmd::cli::Profile` to correctly use `tokmd_settings::Profile`.
- Fixes paths to `ConfigContext` and CLI-resolving functions (e.g. `use tokmd::config::resolve_export_with_config` instead of `use tokmd::resolve_export_with_config`).
- Trade-offs: Structure / Velocity / Governance: Highly aligned with "Prover" persona, very quick to execute, zero runtime risk.

### Option B
- Add executable doctests to `crates/tokmd-core/src/lib.rs`.
- Target `tokmd-core` instead of `tokmd::config::resolve`.
- Trade-offs: Slower to find meaningful broken examples, doesn't address the concrete friction of the broken resolve doctests.

## ✅ Decision
Option A. It explicitly patches drifted/broken doctests inside `tokmd/src/config/resolve/`, fixing the documented usage so it compiles against the real types and correctly imports `tokmd_settings::Profile` instead of a nonexistent alias in `tokmd::cli`.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/config/resolve/export.rs`
- `crates/tokmd/src/config/resolve/lang.rs`
- `crates/tokmd/src/config/resolve/module.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd --doc
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: Documentation and test compilation patches
- Blast radius: API docs only (no runtime effect)
- Risk class: Low
- Rollback: Revert the PR
- Gates run: `cargo test -p tokmd --doc`, `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/run-librarian-api/envelope.json`
- `.jules/runs/run-librarian-api/decision.md`
- `.jules/runs/run-librarian-api/receipts.jsonl`
- `.jules/runs/run-librarian-api/result.json`
- `.jules/runs/run-librarian-api/pr_body.md`

## 🔜 Follow-ups
None.
