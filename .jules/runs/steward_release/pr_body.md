## 💡 Summary
Standardized internal dependencies across the workspace to use `workspace = true` rather than hardcoded path versions. This resolves an issue where `cargo xtask version-consistency` missed drifting inline versions (such as `>=1.9, <2` and `1.11.0`) which could break publish plans.

## 🎯 Why
In the `tokmd` project, `cargo xtask version-consistency` only checks dependencies listed in the `[workspace.dependencies]` section of the root `Cargo.toml`. It does not detect version drift for hardcoded inline path dependencies inside individual crate `Cargo.toml` files. Several internal dependencies (like `tokmd-scan` inside `tokmd-types`) had exact version requirements that were drifting from the workspace version (`1.13.1`). By adopting workspace inheritance (`.workspace = true`), all internal crate versions are automatically locked to the workspace version, eliminating the risk of release mismatches and making future version bumps fully automated.

## 🔎 Evidence
- file paths: `crates/tokmd-analysis-types/Cargo.toml`, `crates/tokmd-cockpit/Cargo.toml`, `crates/tokmd-envelope/Cargo.toml`, `crates/tokmd-scan/Cargo.toml`, `crates/tokmd-types/Cargo.toml`, `crates/tokmd-wasm/Cargo.toml`
- observed behavior: `git grep "version =" crates/*/Cargo.toml | grep path` showed lingering versions like `>=1.9, <2` and `1.11.0` that didn't match `1.13.1`.
- command receipt: `cargo xtask version-consistency` didn't catch the drift initially but passes after moving to workspace dependencies.

## 🧭 Options considered
### Option A (recommended)
- Replace inline version blocks with `.workspace = true` (and `{ workspace = true, default-features = false }` where needed).
- Why it fits this repo and shard: It natively resolves the tooling gap by leveraging Cargo's built-in workspace inheritance, which is already used by the vast majority of internal dependencies in this repo.
- Trade-offs: Structure / Velocity / Governance - High governance win, standardizes dependency management across the board. No downside.

### Option B
- Manually bump the specific `version = "..."` lines to `1.13.1`.
- When to choose it instead: If the crates deliberately need different versions, which is not the case for internal mono-repo components.
- Trade-offs: Increases maintenance burden for every release.

## ✅ Decision
Option A was chosen to permanently fix the version consistency blind spot and align with existing workspace patterns.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/Cargo.toml`: Replaced `tokmd-scan` inline dependency with workspace inheritance.
- `crates/tokmd-cockpit/Cargo.toml`: Migrated `tokmd-analysis` to workspace inheritance.
- `crates/tokmd-envelope/Cargo.toml`: Replaced `tokmd-core` inline dependency with workspace inheritance.
- `crates/tokmd-scan/Cargo.toml`: Replaced `tokmd-model` inline dependency with workspace inheritance.
- `crates/tokmd-types/Cargo.toml`: Replaced `tokmd-scan`, `tokmd-format`, and `tokmd-model` dev-dependencies with workspace inheritance.
- `crates/tokmd-wasm/Cargo.toml`: Replaced `tokmd-types` dev-dependency with workspace inheritance.

## 🧪 Verification receipts
```text
$ cargo xtask version-consistency
Checking version consistency against workspace version 1.13.1
  ✓ Cargo crate versions match 1.13.1.
  ✓ Cargo workspace dependency versions match 1.13.1.
  ✓ Node package manifest versions match 1.13.1.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo xtask publish --plan
=== Publish Plan ===
Workspace version: 1.13.1
Publish order (16 crates):
...
```

## 🧭 Telemetry
- Change shape: Manifest updates
- Blast radius: Build configuration (compatible)
- Risk class: Low risk - no production code changed. Validation confirmed Cargo still builds correctly.
- Rollback: Revert the PR
- Gates run: `cargo xtask version-consistency`, `cargo xtask publish --plan --verbose`, `cargo xtask docs --check`, `cargo build --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
