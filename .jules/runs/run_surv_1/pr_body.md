## 💡 Summary
Fix workspace dependency consistency by replacing inline path/version definitions for internal crates (`tokmd-scan`, `tokmd-format`, `tokmd-model`) with `workspace = true` in their respective `dev-dependencies`.

## 🎯 Why
Internal crates within the tokmd workspace must use `workspace = true` to rely on the version boundaries defined in the root workspace `Cargo.toml`. Certain crates had inline definitions (`{ path = "../tokmd-scan", version = ">=1.9, <2" }`), which caused dependency resolution and versions to drift away from the central source of truth.

## 🔎 Evidence
- `crates/tokmd-types/Cargo.toml`
- `crates/tokmd-scan/Cargo.toml`
- `crates/tokmd-analysis-types/Cargo.toml`
- Checked `cargo tree --workspace --edges dev` and found inconsistent handling.

## 🧭 Options considered
### Option A (recommended)
- Use `workspace = true` for internal crate dev-dependencies.
- why it fits this repo and shard: It consolidates version management at the workspace level, improving architectural hygiene and boundary clarity.
- trade-offs: Structure (improved, strict compliance with cargo workspace pattern), Velocity (neutral/improved due to centralized bumps), Governance (aligns version-consistency).

### Option B
- Ignore the inline versions or wait until a major release bump.
- when to choose it instead: If the crates deliberately need separate dependency boundaries or are intended to be independently published before workspace extraction.
- trade-offs: Divergence in versioning, higher drift risk.

## ✅ Decision
Option A was selected. Consistent workspace dependencies improve the health of the project, especially during cross-crate feature validation.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/Cargo.toml`
- `crates/tokmd-scan/Cargo.toml`
- `crates/tokmd-types/Cargo.toml`

## 🧪 Verification receipts
```text
{"cmd": "cargo tree --workspace --edges normal"}
{"cmd": "git diff"}
{"cmd": "cargo check --workspace"}
{"cmd": "cargo fmt -- --check"}
{"cmd": "cargo clippy --workspace -- -D warnings"}
{"cmd": "cargo test -p tokmd-types -p tokmd-scan -p tokmd-analysis-types"}
```

## 🧭 Telemetry
- Change shape: Workspace-wide refactor of dev-dependencies configuration
- Blast radius: Compilation boundaries / dependencies
- Risk class + why: Low, pure dependency metadata change, no production code changed.
- Rollback: Revert the Cargo.toml changes
- Gates run: `cargo check`, `cargo fmt`, `cargo clippy`, `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/run_surv_1/envelope.json`
- `.jules/runs/run_surv_1/decision.md`
- `.jules/runs/run_surv_1/receipts.jsonl`
- `.jules/runs/run_surv_1/result.json`
- `.jules/runs/run_surv_1/pr_body.md`

## 🔜 Follow-ups
None.
