## 💡 Summary
This PR stops rewriting "Path not found: xxx" to "Unrecognized subcommand 'xxx'". Instead, missing paths are reported honestly, and suggestions for subcommands are properly contained in the hint block, avoiding significant confusion for users with typos in directory names.

## 🎯 Why
When a user typos a path or specifies a missing file (e.g., `tokmd srrc/` or `tokmd test`), the system was rewriting the error to say `Unrecognized subcommand 'srrc'` or `Unrecognized subcommand 'test'`. This creates severe developer friction because the user did not intend to invoke a subcommand. It masks the actual error ("the path doesn't exist").

## 🔎 Evidence
- **File:** `crates/tokmd/src/error_hints.rs`
- **Observed behavior:** `cargo run -- missing_path` returned `Error: Unrecognized subcommand 'missing_path'`.
- **Receipt:** Before changes: `cargo run -- src` returned `Error: Unrecognized subcommand 'src'`. After changes: `cargo run -- src` returns `Error: Path not found: src` with a helpful hint: `Did you mean the subcommand...` or `Run tokmd --help`.

## 🧭 Options considered
### Option A (recommended)
- Retain the base "Path not found" error, removing `missing_path_as_unrecognized_subcommand` which forcibly rewrites the string. Move subcommand typo corrections directly into the `Hints:` section.
- **Why it fits:** It's honest to what the CLI parser does (it parsed it as an implicit `lang` `[PATH]`) while still providing subcommand typo correction.
- **Trade-offs:** Changes CLI error message structure, requiring updates to 4 test files.

### Option B
- Attempt to rewrite the `clap` parser to separate the implicit `lang` path parsing from command parsing.
- **When to choose:** If we want strict clap-level rejection.
- **Trade-offs:** Significantly more complex and might break intentional default-to-lang behaviors.

## ✅ Decision
Chosen **Option A**. Modifying the DX string rewrite in `error_hints.rs` avoids lying to the user about what the parser did, keeping "Path not found" and pushing the subcommand suggestions to the hints.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd/src/error_hints.rs` to remove the forced rewriting of missing path errors to unrecognized subcommands.
- Updated tests in `crates/tokmd/tests/cli_e2e_w65.rs`, `crates/tokmd/tests/cli_error_paths_w51.rs`, `crates/tokmd/tests/cli_errors_w66.rs`, and `crates/tokmd/tests/error_handling_w70.rs` to expect the more accurate `Path not found` base error.

## 🧪 Verification receipts
```text
$ cargo run -- src/
Error: Path not found: src/

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.

$ cargo run -- anolyze
Error: Path not found: anolyze

Hints:
- Did you mean the subcommand `analyze`?

$ cargo run -- frobnicate
Error: Path not found: frobnicate

Hints:
- Run `tokmd --help` to see a list of available subcommands.

$ cargo test -p tokmd
test result: ok. 99 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🧭 Telemetry
- Change shape: Core CLI error message generation and e2e test assertions.
- Blast radius: API (error formatting only). Safe change.
- Risk class: Low risk. Modifies only string formatting for error hints on already-failed executions.
- Rollback: Revert the PR.
- Gates run: `core-rust` (test, build).

## 🗂️ .jules artifacts
- `.jules/runs/palette-1/envelope.json`
- `.jules/runs/palette-1/decision.md`
- `.jules/runs/palette-1/receipts.jsonl`
- `.jules/runs/palette-1/result.json`
- `.jules/runs/palette-1/pr_body.md`

## 🔜 Follow-ups
None.
