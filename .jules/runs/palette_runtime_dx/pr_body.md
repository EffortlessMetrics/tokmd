## 💡 Summary
This patch improves the developer experience of the `tokmd analyze` and `tokmd badge` commands by rejecting `0` as an input for `--max-commits` and `--max-commit-files`. This prevents silent degenerated behavior.

## 🎯 Why
The `context` subcommand already explicitly validates that `--max-commits` and `--max-commit-files` are strictly positive integers by using `super::validate::positive_usize`. However, the `analyze` and `badge` subcommands omitted this validation. Passing `0` led to unhelpful and silent failure states later in the execution. Applying this validation at the CLI level fails fast with a clear error.

## 🔎 Evidence
- `crates/tokmd/src/cli/parser/badge.rs` allowed `0` for `max_commits` and `max_commit_files`.
- `crates/tokmd/src/cli/parser/analysis.rs` allowed `0` for `max_commits` and `max_commit_files`.
- Running `tokmd analyze --max-commits 0` without this fix proceeds without failing at the parse step.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add clap value_parser validation (`super::validate::positive_usize`) to the numeric flags `max_commits` and `max_commit_files` in the `analyze` and `badge` CLI subcommands.
- why it fits this repo and shard: It aligns with the existing validation of these same parameters in the `context` subcommand (found in `crates/tokmd/src/cli/parser/context.rs`), fixing a sharp edge that allowed `0` as an invalid value which would cause silent degenerate behavior or fail ungracefully, aligning perfectly with the Palette persona's runtime DX focus inside the `interfaces` shard.
- trade-offs: Structure (low, reuses existing validation fn) / Velocity (high, clear fix) / Governance (none).

### Option B
- what it is: Let `clap` parse `0` and then handle the error gracefully within the command logic itself (in `run.rs`, `analyze.rs`, etc).
- when to choose it instead: If the value `0` had some valid semantic meaning for those commands that differs from other commands, or if delayed evaluation of the parameter was necessary.
- trade-offs: Increases boilerplate and decreases consistency since we'd duplicate validation logic inside command implementations rather than doing it uniformly in the parser, which `context.rs` already does.

## ✅ Decision
Option A was chosen to maximize consistency with `context.rs` and leverage the existing `super::validate::positive_usize` parser. It fails early and provides a consistent error message across all subcommands.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser/badge.rs`: Added `value_parser = super::validate::positive_usize` to `max_commits` and `max_commit_files`.
- `crates/tokmd/src/cli/parser/analysis.rs`: Added `value_parser = super::validate::positive_usize` to `max_commits` and `max_commit_files`.
- `crates/tokmd/tests/cli_errors_w66.rs`: Added integration tests `analyze_zero_max_commits_fails` and `badge_zero_max_commits_fails`.

## 🧪 Verification receipts
```text
{"cmd": "replace_with_git_merge_diff crates/tokmd/src/cli/parser/badge.rs", "status": "success"}
{"cmd": "replace_with_git_merge_diff crates/tokmd/src/cli/parser/analysis.rs", "status": "success"}
{"cmd": "replace_with_git_merge_diff crates/tokmd/tests/cli_errors_w66.rs", "status": "success"}
{"cmd": "cargo build --verbose", "status": "success"}
{"cmd": "bash -c 'CI=true cargo test -p tokmd --verbose'", "status": "success"}
{"cmd": "cargo fmt -- --check", "status": "success"}
{"cmd": "cargo clippy -- -D warnings", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Runtime input validation (clap configuration)
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): CLI parser configuration only
- Risk class + why: low, purely preventative check adding no new business logic
- Rollback: git revert
- Gates run: `cargo build --verbose`, `CI=true cargo test -p tokmd --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None.
