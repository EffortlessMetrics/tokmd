## 💡 Summary
Improves the error message hint when an unrecognized subcommand is provided. Instead of giving path-related hints (like "Verify the input path exists"), it now correctly suggests running `tokmd --help` to list available subcommands.

## 🎯 Why
When users mis-type a subcommand (and it's not close enough for the "Did you mean?" fuzzy matching), the CLI falls back to treating it as a bad path and displays path-related hints:
```
Error: Unrecognized subcommand 'abc'

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```
This is confusing because the error clearly states it was parsed as a subcommand. This change ensures that when the parser definitively interprets the input as a bad subcommand, it provides a relevant hint to check the help menu.

## 🔎 Evidence
Before:
```bash
$ cargo run --bin tokmd -- abc
Error: Unrecognized subcommand 'abc'

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```

After:
```bash
$ cargo run --bin tokmd -- abc
Error: Unrecognized subcommand 'abc'

Hints:
- Run `tokmd --help` to see a list of available subcommands.
```

## 🧭 Options considered
### Option A (recommended)
- Update `error_hints.rs` to detect when a bare string without path separators is definitively treated as an unrecognized subcommand and return a hint pointing to `--help`, skipping the path-related hints.
- Why it fits: Directly fixes the confusing output with minimal code changes. Preserves "Did you mean?" suggestions for typos and path-related hints for actual paths.
- Trade-offs: Low risk, high value DX improvement.

### Option B
- Modify clap's parsing to completely separate path arguments from subcommands so they never fall back to each other.
- When to choose it instead: If the CLI syntax allowed strict positional separation of paths vs subcommands.
- Trade-offs: High risk. Would require restructuring the CLI syntax and breaking backwards compatibility.

## ✅ Decision
Option A was chosen because it directly addresses the confusing output at the point of error formatting, preserving the flexible CLI syntax while significantly improving the user experience for simple mistakes.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/error_hints.rs`: Updated `suggestions` function to return an early hint `Run \`tokmd --help\` to see a list of available subcommands.` when an unrecognized subcommand is detected and no fuzzy match is found. Updated tests to reflect the new hint.

## 🧪 Verification receipts
```text
$ cargo run --bin tokmd -- abc
Error: Unrecognized subcommand 'abc'

Hints:
- Run `tokmd --help` to see a list of available subcommands.

$ cargo run --bin tokmd -- anolyze
Error: Unrecognized subcommand 'anolyze'

Hints:
- Did you mean the subcommand `analyze`?

$ cargo run --bin tokmd -- missing/path/to/file
Error: Path not found: missing/path/to/file

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```

## 🧭 Telemetry
- Change shape: Logic modification
- Blast radius: Output wording
- Risk class: Low - Only affects error message formatting
- Rollback: Revert `error_hints.rs` changes
- Gates run: `core-rust` (cargo build, CI=true cargo test, cargo fmt, cargo clippy)

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None.
