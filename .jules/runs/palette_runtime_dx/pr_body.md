## 💡 Summary
Improved DX for misspelled subcommands by adding an explicit hint to "Path not found" errors.

## 🎯 Why
Because `tokmd` defaults to the `lang` command and treats unrecognized inputs as positional file paths, typing `tokmd expotr` results in a confusing "Path not found: expotr" error rather than an "Unrecognized command" error. This is a common and frustrating trap for CLI users.

## 🔎 Evidence
- File: `crates/tokmd/src/error_hints.rs`
- Running `tokmd foo` yields the new hint: `- If this was meant to be a subcommand, it is not recognized. Use tokmd --help.`

## 🧭 Options considered
### Option A (recommended)
- Add a static hint to the "path not found" error explaining the implicit command fallback.
- Low complexity, immediate DX win, zero risk of false positives.

### Option B
- Implement fuzzy matching against valid subcommands using `strsim`.
- High complexity, risk of false positives on legitimate path typos.

## ✅ Decision
Option A. It provides a massive usability win with minimal structural change.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/error_hints.rs`: Added the subcommand hint to the path-not-found suggestion block and updated tests to assert its presence.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd
test error_hints::tests::suggests_for_missing_path ... ok

$ cargo run --bin tokmd -- foo
Error: Path not found: foo

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
- If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.
```

## 🧭 Telemetry
- Change shape: Patch
- Blast radius: Output / Help (Error formatting)
- Risk class: Low
- Rollback: Revert the PR
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None.
