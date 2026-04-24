## 💡 Summary
Improved the `tokmd` "Path not found" error message to automatically detect and suggest subcommand typos. When a user runs `tokmd expor` (instead of `export`), the CLI now provides a helpful "did you mean `export`?" hint instead of only telling them that the path doesn't exist.

## 🎯 Why
When an unrecognized subcommand is passed to `tokmd` (e.g. `tokmd ran`), Clap naturally parses it as the `[PATH]...` positional argument. This means the user gets a confusing "Path not found: ran" error. While there was a generic hint about subcommands, adding a Levenshtein-based spell-checker makes the error significantly more actionable and reduces friction for common typos.

## 🔎 Evidence
* **Before:**
  ```
  $ cargo run -- expor
  Error: Path not found: expor

  Hints:
  - Verify the input path exists and is readable.
  - Use an absolute path to avoid working-directory confusion.
  - If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.
  ```
* **After:**
  ```
  $ cargo run -- expor
  Error: Path not found: expor

  Hints:
  - Verify the input path exists and is readable.
  - Use an absolute path to avoid working-directory confusion.
  - If this was meant to be a subcommand, did you mean `export`?
  ```

## 🧭 Options considered
### Option A (recommended)
- Add a lightweight, built-in Levenshtein distance check to `error_hints.rs`.
- Why it fits: Solves a sharp edge natively in `tokmd` without adding a new heavy dependency or trying to hack clap's argument parsing behavior.
- Structure / Velocity / Governance: No new dependencies needed (implemented a simple algorithm inline).

### Option B
- Modify Clap's parsing sequence to force subcommands to be checked first.
- Why to choose it: Would avoid the "path not found" misclassification entirely.
- Trade-offs: Clap doesn't easily support this without complex overrides that might break the standard `tokmd [OPTIONS] [PATH] [COMMAND]` fallback behavior where `[COMMAND]` can be omitted to default to `lang`.

## ✅ Decision
Option A was chosen. Adding a custom fuzzy match to the `suggestions()` function handles the UX papercut exactly at the moment the error is rendered, while maintaining standard `clap` parsing logic.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/error_hints.rs`: Added `levenshtein()` string distance function and logic to extract the missing path and suggest known `tokmd` subcommands.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test cli_e2e_w65
test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s

cargo test -p tokmd
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
```

## 🧭 Telemetry
- Change shape: CLI error hint enrichment
- Blast radius: Only affects the text of the CLI output when a "Path not found" error is raised.
- Risk class: Low
- Rollback: Revert the `error_hints.rs` changes.
- Gates run: `cargo test -p tokmd`

## 🗂️ .jules artifacts
- `.jules/runs/run-palette-dx/envelope.json`
- `.jules/runs/run-palette-dx/decision.md`
- `.jules/runs/run-palette-dx/receipts.jsonl`
- `.jules/runs/run-palette-dx/result.json`
- `.jules/runs/run-palette-dx/pr_body.md`

## 🔜 Follow-ups
None.
