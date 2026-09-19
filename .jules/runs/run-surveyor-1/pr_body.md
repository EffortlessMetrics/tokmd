## 💡 Summary
Added two missing non-Rust files (`Dockerfile.release` and `scripts/publish-release-crates.sh`) to the file policy allowlist and fixed `workspace = true` usage in `dev-dependencies` across several crates.

## 🎯 Why
The `cargo xtask check-file-policy` command fails (or emits advisory warnings) because the release Dockerfile and publish script were missing from the allowlist. Additionally, the publish integration tests (`xtask/tests/publish_w71.rs`) fail because some internal `dev-dependencies` used `workspace = true` instead of a loose local path requirement (`">=1.9, <2"`). This fixes workspace hygiene for file tracking and publication boundaries.

## 🔎 Evidence
```
file-policy findings (2):
  - file Dockerfile.release does not match any non-Rust allowlist glob
  - file scripts/publish-release-crates.sh does not match any non-Rust allowlist glob
```

```
thread 'publishable_internal_dev_dependencies_use_loose_versions' (25167) panicked at xtask/tests/publish_w71.rs:341:5:
publishable internal dev-dependencies must not require unpublished same-version crates:
  - tokmd-analysis-types dev-dependency tokmd-scan must not use workspace = true
  - tokmd-wasm dev-dependency tokmd-types must not use workspace = true
  ...
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Add the missing non-Rust files to the allowlist and fix the `workspace = true` errors in `dev-dependencies` for internal crates.
- why it fits this repo and shard: Surveyor Persona (workspace-wide shard). Fixes broken release policy boundaries and publish task rules that affect workspace structure and test hygiene.
- trade-offs: Structure / Velocity / Governance: Improves overall workspace structure and publication constraints, ensuring tests pass.

### Option B
- what it is: Ignore the file policy check since it's advisory, and only fix the failing tests.
- when to choose it instead: If the non-rust policy is being completely revamped.
- trade-offs: Leaves known drift in the codebase which will fail under strict checking.

## ✅ Decision
Option A was chosen. It directly fixes both the file policy drift and the publication dependency boundary issues.

## 🧱 Changes made (SRP)
- `policy/non-rust-allowlist.toml`
- `crates/tokmd-analysis-types/Cargo.toml`
- `crates/tokmd-wasm/Cargo.toml`
- `crates/tokmd-types/Cargo.toml`
- `crates/tokmd-scan/Cargo.toml`
- `crates/tokmd-envelope/Cargo.toml`

## 🧪 Verification receipts
```text
file-policy OK: 97 entries, 1240 non-Rust files covered, 1348 Rust files skipped
```

```text
test publishable_internal_dev_dependencies_use_loose_versions ... ok
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.13s
```

## 🧭 Telemetry
- Change shape: Config adjustments
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Low risk config/publish adjustments.
- Risk class + why: Low, fixes existing test failures and policy drift.
- Rollback: Revert the commits.
- Gates run: `cargo check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `CI=true cargo test -p xtask --verbose`, `cargo xtask check-file-policy --strict`

## 🗂️ .jules artifacts
- `.jules/runs/run-surveyor-1/envelope.json`
- `.jules/runs/run-surveyor-1/decision.md`
- `.jules/runs/run-surveyor-1/receipts.jsonl`
- `.jules/runs/run-surveyor-1/result.json`
- `.jules/runs/run-surveyor-1/pr_body.md`

## 🔜 Follow-ups
None.
