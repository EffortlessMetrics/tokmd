## 🧭 Options considered
### Option A (recommended)
Fix the CLI error reporting in `crates/tokmd/src/error_hints.rs` so that when `clap` falls back to the implicit `lang` subcommand and encounters a "Path not found" error, we rewrite the error message to "Error: Unrecognized subcommand '<bad>'" if the "path" does not contain path separators (`/`, `\`, `.`). This directly resolves the "unrecognized subcommands are parsed as file paths by `CliLangArgs`" memory note by fixing the UX at the display layer, without requiring massive restructuring of how `clap` handles fallbacks. It also provides the hint "If you intended to pass a file path, verify it exists and is readable."

- **Why it fits this repo and shard**: The issue is directly about CLI help/default/usage sharp edges. `error_hints.rs` is already dedicated to cleaning up and contextualizing raw errors. This simply moves the "Did you mean subcommand" logic to rewrite the main error string when appropriate.
- **Trade-offs**:
  - **Structure**: Clean. It isolates the change to the error formatting layer (`error_hints.rs` and `cli_e2e_w65.rs`).
  - **Velocity**: Fast and risk-free since it only changes the printed error representation.
  - **Governance**: Adheres to the prompt's instruction to address "unclear or low-context error messages" and "confusing diagnostics".

### Option B
Attempt to use a `clap` parser custom validator or override the command logic to fail before the `lang` handler runs if no valid subcommand is matched.
- **When to choose it instead**: If we strictly wanted `clap` to print its own generic "error: unrecognized subcommand" rather than our `tokmd` styled error system.
- **Trade-offs**: This would involve complex changes to how the `flatten` global arguments and default subcommands are handled in `parser.rs`. It risks breaking the `tokmd <path>` zero-config use case by incorrectly flagging valid relative directories that don't have dots or slashes.

## ✅ Decision
I chose **Option A**. It's the most targeted and safe fix. I updated `error_hints.rs` to rewrite the error string when an unrecognized subcommand disguised as a missing path is detected, and updated the corresponding test in `cli_e2e_w65.rs` to expect "Error: Unrecognized subcommand 'frobnicate'".
