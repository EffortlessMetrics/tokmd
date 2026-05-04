## 💡 Summary
Improved the CLI error UX by ensuring that unrecognized subcommands produce an "Unrecognized subcommand" error instead of a confusing "Path not found" error. This addresses the sharp edge where `clap`'s fallback to the implicit `lang` subcommand parses mistyped commands as file paths.

## 🎯 Why
When users mistakenly type `tokmd frobnicate` or `tokmd blabla`, the CLI would default to the implicit `lang` subcommand and attempt to scan the path `frobnicate`. It would then fail with `Path not found: frobnicate`. While there was a hint at the bottom suggesting it might be an unrecognized subcommand, the primary error message was misleading.

## 🔎 Evidence
- **File:** `crates/tokmd/src/error_hints.rs`
- **Observed behavior:** `cargo run -- frobnicate` produced `Error: Path not found: frobnicate`.
- **Receipt:** Before fix, the test `frobnicate_unknown_subcommand_has_stable_error_output` expected `Error: Path not found: frobnicate`.

## 🧭 Options considered
### Option A (recommended)
Fix the CLI error reporting in `crates/tokmd/src/error_hints.rs` so that when `clap` falls back to the implicit `lang` subcommand and encounters a "Path not found" error, we rewrite the error message to "Error: Unrecognized subcommand '<bad>'" if the "path" does not contain path separators (`/`, `\`, `.`). This directly resolves the "unrecognized subcommands are parsed as file paths by `CliLangArgs`" memory note by fixing the UX at the display layer.

- **Why it fits this repo and shard**: The issue is directly about CLI help/default/usage sharp edges. `error_hints.rs` is already dedicated to cleaning up and contextualizing raw errors.
- **Trade-offs**:
  - **Structure**: Clean. It isolates the change to the error formatting layer.
  - **Velocity**: Fast and risk-free since it only changes the printed error representation.
  - **Governance**: Adheres to the prompt's instruction to address "unclear or low-context error messages".

### Option B
Attempt to use a `clap` parser custom validator or override the command logic to fail before the `lang` handler runs if no valid subcommand is matched.
- **When to choose it instead**: If we strictly wanted `clap` to print its own generic "error: unrecognized subcommand".
- **Trade-offs**: This would involve complex changes to how the `flatten` global arguments and default subcommands are handled, risking breakage for the `tokmd <path>` zero-config use case.

## ✅ Decision
I chose **Option A**. It's the most targeted and safe fix. I updated `error_hints.rs` to rewrite the error string when an unrecognized subcommand disguised as a missing path is detected, and updated the corresponding test.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd/src/error_hints.rs` to rewrite the main error string from "Path not found" to "Unrecognized subcommand" when the input lacks path separators.
- Updated `crates/tokmd/tests/cli_e2e_w65.rs` to assert the correct "Unrecognized subcommand" output.

## 🧪 Verification receipts
```text
$ cargo run -- frobnicate
Error: Unrecognized subcommand 'frobnicate'

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```

## 🧭 Telemetry
- **Change shape:** Patch (CLI Error Formatting)
- **Blast radius:** Error output text only
- **Risk class:** Low (No behavioral changes to analysis)
- **Rollback:** Revert changes in `error_hints.rs` and the test
- **Gates run:** `cargo test -p tokmd`, `cargo check`

## 🗂️ .jules artifacts
- `.jules/runs/run-palette-interfaces/envelope.json`
- `.jules/runs/run-palette-interfaces/receipts.jsonl`
- `.jules/runs/run-palette-interfaces/decision.md`
- `.jules/runs/run-palette-interfaces/result.json`
- `.jules/runs/run-palette-interfaces/pr_body.md`

## 🔜 Follow-ups
None.
