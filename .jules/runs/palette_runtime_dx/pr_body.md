## 💡 Summary
Aligned the error output for `tokmd check-ignore <nonexistent-path>` to use the standard "Error: Path not found:" prefix instead of "Error: Path '<path>' does not exist".

## 🎯 Why
The `check-ignore` subcommand was emitting a unique error message string ("Path '<path>' does not exist") when asked to check a non-existent file. By aligning this to the standard `Path not found: <path>` message used by other subcommands, we trigger the CLI's global error hint machinery, which appends helpful troubleshooting suggestions (like verifying the path and using absolute paths) to the user.

## 🔎 Evidence
- `crates/tokmd/src/commands/check_ignore.rs`
- Observed behavior: `cargo run -- check-ignore non-exist.rs` printed `Error: Path 'non-exist.rs' does not exist` without any hints.
- After change: `cargo run -- check-ignore non-exist.rs`
```text
Error: Path not found: non-exist.rs

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `crates/tokmd/src/commands/check_ignore.rs` and the related test string to output "Path not found: <path>".
- why it fits this repo and shard: It directly improves the runtime developer experience (Palette's goal) by providing clear, consistent context and activating existing hint logic, adhering strictly to the `interfaces` shard.
- trade-offs:
  - Structure: Requires modifying the hardcoded string and test assertion.
  - Velocity: Extremely fast to implement and verify.
  - Governance: Low risk, purely a UX consistency improvement.

### Option B
- what it is: Do not fix `check-ignore` error messages and find another target.
- when to choose it instead: If the unique error string served a specific contractual purpose (it does not).
- trade-offs: Leaves a known UX gap unresolved.

## ✅ Decision
Option A was chosen as it aligns perfectly with the goal of improving unclear error messages by using the existing generic hint machinery.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/commands/check_ignore.rs`

## 🧪 Verification receipts
```text
$ cargo run -- check-ignore non-exist.rs
Error: Path not found: non-exist.rs

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.

$ cargo test -p tokmd --test cli_error_paths_w51
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
```

## 🧭 Telemetry
- Change shape: Minor string alignment
- Blast radius: Output wording for `tokmd check-ignore` on missing paths.
- Risk class: Low - affects error string matching only (updated test).
- Rollback: Revert the string in `check_ignore.rs`.
- Gates run: `core-rust` (test, fmt, clippy, build)

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None
