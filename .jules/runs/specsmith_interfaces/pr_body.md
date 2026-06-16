## 💡 Summary
Fixed CLI error messages for non-existent paths across `check-ignore`, `diff`, and `init` to use standard "Path not found: <path>" format. This correctly triggers the global `error_hints` machinery.

## 🎯 Why
Some subcommands were emitting "does not exist" or "Directory does not exist", bypassing the CLI's `error_hints` system which provides actionable advice for missing paths.

## 🔎 Evidence
Tests failed: `init_into_nonexistent_dir_fails_gracefully` expected "does not exist" but actually needed standard error handling to match correctly or trigger hints. The memory specifically notes: "standard CLI error formatting for missing paths should always use the format `Error: Path not found: <path>` ... as this exact string prefix automatically triggers the CLI's global error hint machinery".

## 🧭 Options considered
### Option A (recommended)
- Update error strings in `check_ignore.rs`, `diff.rs`, and `tokeignore/mod.rs` to output standard "Path not found: {}"
- why it fits this repo and shard: It adheres to the specific memory instruction about error formats and fits the interfaces shard/polish mission.
- trade-offs: Structure / Velocity / Governance: Fixes bugs but required updating several test assertions.

### Option B
- Add a hack to `error_hints.rs` to match all custom "does not exist" formats.
- when to choose it instead: If modifying error strings would break external contracts.
- trade-offs: Leaves inconsistent error formats scattered throughout the codebase.

## ✅ Decision
Option A. Fixed the error formats in the code, updated the assertions in the tests, and verified everything passes.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/commands/check_ignore.rs`
- `crates/tokmd/src/commands/diff.rs`
- `crates/tokmd/tests/init_cli_w76.rs`
- `crates/tokmd-scan/src/tokeignore/mod.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd --test init_cli_w76
cargo test -p tokmd --test cli_error_paths_w51
CI=true cargo test -p tokmd --verbose
cargo clippy -- -D warnings
cargo fmt -- --check
cargo build --verbose
```

## 🧭 Telemetry
- Change shape: Bug fix
- Blast radius: CLI error messages
- Risk class + why: Low, only error messages
- Rollback: Revert PR
- Gates run: core-rust fallback

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces/envelope.json`
- `.jules/runs/specsmith_interfaces/decision.md`
- `.jules/runs/specsmith_interfaces/receipts.jsonl`
- `.jules/runs/specsmith_interfaces/result.json`
- `.jules/runs/specsmith_interfaces/pr_body.md`

## 🔜 Follow-ups
None.
