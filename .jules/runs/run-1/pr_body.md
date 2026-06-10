## 💡 Summary
Consolidated explicit path and version declarations for internal crates to rely on workspace dependency resolution. This aligns crates like `tokmd-types`, and `tokmd-wasm` with the rest of the workspace.

## 🎯 Why
Several `dev-dependencies` and `dependencies` blocks across internal crates were manually specifying `path` and `version` blocks instead of utilizing `{ workspace = true }`. This causes internal drift and increases maintenance overhead when making version bumps or structural refactors across the project.

## 🔎 Evidence
- `crates/tokmd-types/Cargo.toml` manually specified `path = "../tokmd-scan", version = ">=1.9, <2"`.
- Workspace resolution (`cargo tree`) succeeds and confirms proper linking after the change.

## 🧭 Options considered
### Option A (recommended)
- Use `{ workspace = true }` for all internal crates already defined in the root `Cargo.toml` under `[workspace.dependencies]` where possible.
- Fits this repo and shard as it directly improves workspace structure and dependency hygiene.
- Trade-offs: Requires a targeted refactor of `Cargo.toml` files, but increases Structural cohesion and Velocity for future version bumps.

### Option B
- Keep the hardcoded paths.
- Trade-offs: Preserves current state, but allows dependency graphs to fragment and requires redundant version updates across the codebase.

## ✅ Decision
Proceed with Option A. The workspace structure is cleaner and less prone to publish errors when centralized in the root `Cargo.toml`. `tokmd-cockpit` was left as-is because Cargo workspace dependencies cannot override default-features yet, so the explicit path is maintained.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/Cargo.toml`: Migrated `tokmd-scan` to workspace.
- `crates/tokmd-envelope/Cargo.toml`: Migrated `tokmd-core` to workspace.
- `crates/tokmd-scan/Cargo.toml`: Migrated `tokmd-model` to workspace.
- `crates/tokmd-types/Cargo.toml`: Migrated `tokmd-scan`, `tokmd-format`, and `tokmd-model` to workspace.
- `crates/tokmd-wasm/Cargo.toml`: Migrated `tokmd-types` to workspace.

## 🧪 Verification receipts
```text
cargo build --verbose
CI=true cargo test --verbose -p tokmd-cockpit -p tokmd-analysis-types -p tokmd-envelope -p tokmd-scan -p tokmd-types -p tokmd-wasm
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Structural refactor.
- Blast radius: Internal API / build resolution. No runtime impact.
- Risk class: Low. Cargo verifies workspace validity at build time.
- Rollback: Revert `Cargo.toml` changes.
- Gates run: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/run-1/envelope.json`
- `.jules/runs/run-1/decision.md`
- `.jules/runs/run-1/receipts.jsonl`
- `.jules/runs/run-1/result.json`
- `.jules/runs/run-1/pr_body.md`

## 🔜 Follow-ups
None.
