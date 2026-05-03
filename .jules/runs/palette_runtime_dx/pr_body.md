## 💡 Summary
Improved the CLI developer experience by properly throwing a `clap` "unrecognized subcommand" error when users typo a subcommand. Previously, typoed subcommands silently fell back to the implicit `lang` mode and resulted in a confusing "Path not found" error because they were interpreted as file paths.

## 🎯 Why
In `tokmd`, `clap` defaults to the `lang` subcommand when no valid subcommand is provided, parsing the remaining arguments as file paths (`CliLangArgs.paths`). Consequently, typing an invalid subcommand like `tokmd modlue` yielded `Path not found: modlue`. While `error_hints.rs` attempted to offer helpful suggestions, the primary error message was misleading. Catching this pattern and throwing a native clap error drastically improves the DX for users making typos.

## 🔎 Evidence
- Prior to fix: `tokmd anolyze` produced `Error: Path not found: anolyze`.
- After fix: `tokmd anolyze` correctly errors with `error: unrecognized subcommand 'anolyze'` along with clap's standard help and hints.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Add a validation step in `src/lib.rs` immediately after parsing arguments. If the command falls back to the default mode with a single path that doesn't exist and resembles a typoed command (no slashes/dots), manually trigger `clap`'s `InvalidSubcommand` error.
- **Why it fits this repo and shard:** Resolves the issue directly at the CLI interface level without removing the convenient `tokmd .` fallback behavior, keeping the interface robust while solving the precise error messaging.
- **Trade-offs:** Needs manual validation of the path string to ensure it looks like a subcommand, which is slightly heuristic but extremely accurate for typical usage.

### Option B
- **What it is:** Remove the `#[command(flatten)]` fallback in `Cli` entirely, forcing users to type `tokmd lang .`.
- **When to choose it instead:** If we decided the implicit `lang` subcommand was a bad architectural choice overall.
- **Trade-offs:** Significantly breaks backward compatibility and worsens DX for the common `tokmd` (no-args) use case.

## ✅ Decision
Option A was chosen. We intercept the fallback logic right after parsing. If the single parsed path does not exist on disk and looks like a bare word, we emit `clap`'s standard unrecognized subcommand error.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/lib.rs`: Intercept the parsed arguments to conditionally emit `ErrorKind::InvalidSubcommand` for typoed subcommands.
- `crates/tokmd/tests/cli_e2e_w65.rs`: Updated the integration test asserting the error output.

## 🧪 Verification receipts
```text
$ cargo run -- anolyze
error: unrecognized subcommand 'anolyze'

Usage: tokmd [OPTIONS] [PATH]... [COMMAND]

For more information, try '--help'.
```

## 🧭 Telemetry
- **Change shape:** Patch
- **Blast radius:** CLI parsing error path
- **Risk class:** Low (only alters terminal output on error)
- **Rollback:** Revert commit
- **Gates run:** `cargo test -p tokmd`

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None.
