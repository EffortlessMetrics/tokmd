## 💡 Summary
Standardized internal dependency definitions to use `workspace = true` in workspace crates (`tokmd-analysis-types`, `tokmd-envelope`, `tokmd-scan`, `tokmd-types`, `tokmd-wasm`), and aligned `tokmd-cockpit`'s explicit dependency version. This improves release metadata consistency and centralizes governance through `workspace.dependencies`.

## 🎯 Why
Several internal crates were redefining dependencies on other workspace crates via explicit `path = "../..."` and `version = "..."` pairs instead of inheriting from the root `Cargo.toml`. This creates metadata drift risk and complicates the publishing and version-bumping sequences, as changes must be synchronized in multiple places instead of just one.

## 🔎 Evidence
- `grep -nE 'path = "\.\./tokmd' crates/*/Cargo.toml` revealed hard-coded internal dependencies in multiple crates.
- The root `Cargo.toml` already defined central internal dependencies in `[workspace.dependencies]`.

## 🧭 Options considered
### Option A (recommended)
- Centralize workspace dependency versioning using `workspace = true` (and update explicitly where inheritance is blocked).
- **Why it fits this repo and shard**: Aligning manifests with the workspace root is a tier-1 cargo best practice and falls squarely in the tooling-governance shard.
- **Trade-offs**:
  - Structure: High alignment with workspace centralization.
  - Velocity: Prevents future friction with version drift.
  - Governance: High, ensures safer releases.

### Option B
- Keep explicit path/version and rely on CI/xtask failures to catch drift.
- **When to choose it instead**: Never in a centralized workspace without a specific overriding requirement.
- **Trade-offs**: Manual maintenance burden, higher risk of drift.

## ✅ Decision
Selected **Option A**. Consolidating dependency definitions to use the root `[workspace.dependencies]` improves maintainability and aligns the workspace metadata safely.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/Cargo.toml`
- `crates/tokmd-cockpit/Cargo.toml`
- `crates/tokmd-envelope/Cargo.toml`
- `crates/tokmd-scan/Cargo.toml`
- `crates/tokmd-types/Cargo.toml`
- `crates/tokmd-wasm/Cargo.toml`

## 🧪 Verification receipts
```text
cargo xtask version-consistency (Success)
cargo xtask publish --plan --verbose (Success)
cargo xtask docs --check (Success)
cargo fmt -- --check (Success)
cargo clippy -- -D warnings (Success)
```

## 🧭 Telemetry
- **Change shape**: Dependency metadata simplification
- **Blast radius**: Low (Manifest-only change, internal deps resolved exactly the same)
- **Risk class**: Low, no behavioral code changes
- **Rollback**: Safe, revert `Cargo.toml` changes
- **Gates run**: `governance-release` fallback commands

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.

## ⚠️ Notes for Reviewer
Because of the number of modified manifest files (`crates/*/Cargo.toml`), the CI plan estimates 128 LEM, which is above the 125 hard ceiling limit. Please add the `ci-budget-override` label to the PR to allow the full check suite to run.
