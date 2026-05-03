## 💡 Summary
This is a learning PR. I investigated the workspace structure and discovered drift where `tokmd-config`, `vendor/home-0.5.12`, and `crates/tokmd/tests/data` believe they are in the workspace but are omitted from `[workspace.members]`.

## 🎯 Why
When running workspace-wide analysis tools like `cargo metadata` or `cargo machete`, these dangling crates cause errors (`current package believes it's in a workspace when it's not`). However, simply adding `tokmd-config` back into the workspace causes its test suite to fail, as it relies on outdated type paths that have since moved to `tokmd_settings` and `tokmd_types`. Fixing this requires a large generic cleanup of tests, so it is recorded as a friction item instead.

## 🔎 Evidence
- file paths: `Cargo.toml`, `crates/tokmd-config/Cargo.toml`
- observed behavior: `cargo machete --with-metadata` fails on workspace boundaries.
- receipt: `error when handling ./crates/tokmd-config/Cargo.toml: cargo metadata exited with an error: error: current package believes it's in a workspace when it's not`

## 🧭 Options considered
### Option A (recommended)
- what it is: Create a learning PR documenting the out-of-shard friction item.
- why it fits this repo and shard: Identifies the workspace structural boundary issue without getting bogged down in rewriting a deprecated crate's test suite.
- trade-offs: Structure (good recording), Velocity (fast), Governance (compliant with Surveyor prompt).

### Option B
- what it is: Fix workspace members and patch `tokmd-config` tests.
- when to choose it instead: If the tests only required minor import fixes.
- trade-offs: Touches dozens of test files in an older crate, causing a large diff of generic cleanup.

## ✅ Decision
Option A. The `tokmd-config` crate is a compatibility shim with broken tests. Documenting the friction is the honest Surveyor outcome.

## 🧱 Changes made (SRP)
- None (Learning PR)

## 🧪 Verification receipts
```text
cargo machete --with-metadata
error when handling ./crates/tokmd-config/Cargo.toml: `cargo metadata` exited with an error: error: current package believes it's in a workspace when it's not:
current:   /app/crates/tokmd-config/Cargo.toml
workspace: /app/Cargo.toml
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class: Low
- Rollback: N/A
- Gates run: cargo test -p tokmd-core --all-features (passed), cargo check --workspace --all-features (passed)

## 🗂️ .jules artifacts
- `.jules/runs/surveyor_workspace_01/envelope.json`
- `.jules/runs/surveyor_workspace_01/decision.md`
- `.jules/runs/surveyor_workspace_01/receipts.jsonl`
- `.jules/runs/surveyor_workspace_01/result.json`
- `.jules/runs/surveyor_workspace_01/pr_body.md`
- `.jules/friction/open/FRIC-20240503-001.md`

## 🔜 Follow-ups
- FRIC-20240503-001: Resolve dangling workspace crates (either delete `tokmd-config` or update its test suite).
