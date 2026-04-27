## 💡 Summary
Improves the runtime developer experience by enriching the CLI error message for unknown paths. When a user provides a path that does not exist and `tokmd` guesses it might be a misspelled subcommand, the hint now explicitly quotes the unrecognized string.

## 🎯 Why
Previously, if a user provided an invalid path, `tokmd` would output a generic error hint: `"If this was meant to be a subcommand, it is not recognized."` This was confusing because users often provide valid subcommands followed by invalid positional paths (e.g., `tokmd analyze nonexistent`), leading to a misleading hint implying they hadn't provided a subcommand at all. By quoting the token and adding a structural heuristic, the CLI now gives targeted, high-context feedback.

## 🔎 Evidence
Before:
`cargo run -p tokmd -- handoff --no-git nonexistent --force`
`Error: Path not found: nonexistent`
`Hints:`
`- If this was meant to be a subcommand, it is not recognized. Use tokmd --help.`

After:
`cargo run -p tokmd -- handoff --no-git nonexistent --force`
`Error: Path not found: nonexistent`
`Hints:`
`- If \`nonexistent\` was intended as a subcommand, it is not recognized. Use tokmd --help.`

## 🧭 Options considered
### Option A (recommended)
- Enhance the `suggestions` heurustic in `error_hints.rs` to extract `bad_path` and format it into the "not recognized" fallback string, skipping it if the path contains separators (`/`, `.`, `\`).
- Fits the repo and shard because it directly addresses the low-context error within the existing diagnostic pipeline without altering the core CLI struct definitions or parser boundaries.
- Trade-offs: Requires tracking state within the heuristic loop.

### Option B
- Plumb whether a valid subcommand was parsed into the error formatting layer to disable the hint entirely when appropriate.
- When to choose: If the error structures natively carried `clap` argument match data.
- Trade-offs: Overly invasive to the `anyhow::Error` chain used across the application.

## ✅ Decision
Option A was chosen as it delivers a high-impact DX improvement within a bounded scope, conforming to the existing heuristic patterns in `error_hints.rs`.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/error_hints.rs`: Updated `suggestions` to capture the unrecognized path string and conditionally inject it into the hint if it doesn't resemble a file path.

## 🧪 Verification receipts
```text
cargo test -p tokmd --lib error_hints
test result: ok. 6 passed

cargo test -p tokmd --test cli_error_paths_w51
test result: ok. 22 passed

cargo test -p tokmd --test cli_error_help_w73
test result: ok. 30 passed
```

## 🧭 Telemetry
- Change shape: Implementation
- Blast radius: CLI Diagnostics (Runtime output only)
- Risk class: Low
- Rollback: Revert the format string and test logic in `error_hints.rs`
- Gates run: `cargo check`, `cargo fmt`, `cargo clippy`, `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None.
