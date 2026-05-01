## 💡 Summary
Improved the "Path not found" error hints to better handle misspelled subcommands. When a user typos a command (like `moduel`), the CLI now only suggests "Did you mean the subcommand `module`?" instead of also outputting confusing path-related hints (e.g., "Verify the input path exists"). Also added `help` to the known commands list so `helpp` provides a suggestion.

## 🎯 Why
When a subcommand is misspelled, it falls back to being parsed as a path for the default `lang` subcommand, producing a "Path not found: moduel" error. While the `error_hints.rs` code attempts to detect typos and suggests "Did you mean the subcommand X?", it *also* appends generic file path hints like "Verify the input path exists". This is confusing DX because the user wasn't trying to supply a path.

## 🔎 Evidence
- **File**: `crates/tokmd/src/error_hints.rs`
- **Observed Behavior**: `cargo run --bin tokmd -- moduel` outputted path verification hints alongside the "Did you mean?" hint.
- **Improved Behavior**: It now only outputs the "Did you mean the subcommand `module`?" hint when it detects a likely command typo.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Update the logic in `error_hints.rs` to skip pushing generic file system hints if `did_you_mean` is triggered. Also add `help` to the list of known commands.
- **Why it fits this repo and shard**: Matches the Builder style of making a targeted, high-value DX fix within the existing `error_hints` facade without refactoring core clap logic or breaking default argument parsing.
- **Trade-offs**: Still technically says "Path not found" before the hint, but the hint is now laser-focused so the user immediately knows how to proceed.

### Option B
- **What it is**: Override the "Path not found" prefix completely by unwrapping and formatting the error from `error.rs` or `clap`.
- **When to choose it instead**: If we want to fully decouple command typos from path failures.
- **Trade-offs**: Requires more invasive changes into how errors are propagated and typed.

## ✅ Decision
Option A was chosen as it resolves the confusing user experience quickly and safely without breaking existing fallback logic.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/error_hints.rs`

## 🧪 Verification receipts
```text
{"command": "cargo test -p tokmd", "outcome": "Success, all tests passed"}
{"command": "cargo run --bin tokmd -- moduel || true", "outcome": "Error output only contains 'Did you mean the subcommand `module`?' instead of path hints"}
```

## 🧭 Telemetry
- **Change shape**: Patch
- **Blast radius**: API (error hints)
- **Risk class**: Low, only modifies hint string output.
- **Rollback**: Trivial revert.
- **Gates run**: `cargo check`, `cargo test -p tokmd`, `cargo clippy`, `cargo fmt --check`, `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo xtask publish --plan`

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None.
