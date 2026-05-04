## 💡 Summary
Restored `tokmd-config` to the root workspace members, fixed its bitrotted test suite, and excluded non-workspace folders to resolve `cargo metadata` boundaries issues.

## 🎯 Why
`cargo machete --with-metadata` revealed that `crates/tokmd-config`, `crates/tokmd/tests/data`, and `vendor/home-0.5.12` believed they were in the workspace but were neither explicitly included nor excluded. `tokmd-config` was effectively orphaned, meaning its extensive test suite wasn't running in CI. Adding it back revealed compilation failures from recent struct renames (e.g., `CliExportFormat` to `ExportFormat`), which have now been fixed.

## 🔎 Evidence
- Running `cargo machete --with-metadata` output: `error: current package believes it's in a workspace when it's not` for the three paths.
- Running `cargo test -p tokmd-config` resulted in `unresolved import` and `cannot find type` errors due to old enum prefixes (`Cli`).

## 🧭 Options considered
### Option A (recommended)
- Add `tokmd-config` back into the workspace members, fix the test suite compilation failures, and exclude non-workspace crates. Use `[package.metadata.cargo-machete]` to ignore dynamic features.
- Why it fits this repo and shard: Resolves structural hygiene issues spotted by tools, repairs broken tests in an orphaned crate, and solidifies boundaries.
- Trade-offs: Structure / Velocity / Governance: Improves structure and governance by ensuring test suites run; requires slight velocity hit to fix the bitrot.

### Option B
- Completely remove `tokmd-config`.
- when to choose it instead: If the crate tests were fully migrated to other crates.
- trade-offs: We would lose a significant amount of property-based and BDD scenario tests.

## ✅ Decision
Chose Option A to preserve the extensive test coverage in `tokmd-config` while fixing the workspace metadata errors.

## 🧱 Changes made (SRP)
- `Cargo.toml`: Added `crates/tokmd-config` to members and default-members; added `exclude` for `crates/tokmd/tests/data` and `vendor/home-0.5.12`.
- `crates/tokmd-config/Cargo.toml`: Added `tokmd-settings` and `tokmd-types` as dev-dependencies.
- `crates/tokmd-config/tests/*.rs`: Updated imports from `tokmd::settings` to `tokmd_settings` and removed `Cli` prefixes from enum variants to fix bitrot.
- `crates/tokmd/tests/data/Cargo.toml`: Added an empty `[workspace]` block to prevent inheritance issues.
- `vendor/home-0.5.12/Cargo.toml`: Added an empty `[workspace]` block.
- `fuzz/Cargo.toml`, `crates/tokmd-node/Cargo.toml`: Added `cargo-machete` ignores.
- `crates/tokmd/tests/snapshots/*`: Updated `insta` snapshots for tests reflecting the new `Cargo.toml` lines.

## 🧪 Verification receipts
```text
cargo test --workspace --exclude tokmd-python --all-targets --all-features
cargo machete --with-metadata
```

## 🧭 Telemetry
- Change shape: Workspace structure and test fixes
- Blast radius: Build configuration and test suites
- Risk class: Low - fixes isolated tests and metadata, no runtime behavior changes.
- Rollback: Revert `Cargo.toml` and test file updates.
- Gates run: `core-rust` (cargo check, cargo test)

## 🗂️ .jules artifacts
- `.jules/runs/surveyor_workspace/envelope.json`
- `.jules/runs/surveyor_workspace/decision.md`
- `.jules/runs/surveyor_workspace/receipts.jsonl`
- `.jules/runs/surveyor_workspace/result.json`
- `.jules/runs/surveyor_workspace/pr_body.md`

## 🔜 Follow-ups
None.
