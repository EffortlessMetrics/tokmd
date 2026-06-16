## 💡 Summary
Updated `docs/debugging.md` to fix outdated CLI arguments in the example bash commands. The examples now use the correct positional path argument and updated output directory flags (`--output-dir` and `--artifacts-dir`).

## 🎯 Why
The examples in `docs/debugging.md` demonstrated factual drift from actual behavior. Running `cargo run -p tokmd -- run --path . --out target/tokmd-debug` fails with `error: unexpected argument '--path' found`. This breaks executable documentation for new contributors trying to debug the application locally.

## 🔎 Evidence
- File path: `docs/debugging.md`
- Observed behavior: `cargo run -p tokmd -- run --path .` fails because `--path` is no longer a valid named argument.
- Receipt:
```text
$ cargo run -p tokmd -- run --path . --out target/tokmd-debug
error: unexpected argument '--path' found
  tip: to pass '--path' as a value, use '-- --path'
Usage: tokmd run [OPTIONS] [PATH]...
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Fix the CLI flags directly in `docs/debugging.md`.
- why it fits this repo and shard: Fixes explicit factual drift in the `docs` target space, aligning executable documentation with current CLI contract.
- trade-offs: Structure (low risk), Velocity (high), Governance (maintains accuracy).

### Option B
- what it is: Change the CLI parser to accept `--path` and `--out` as aliases.
- when to choose it instead: If the goal was to avoid breaking old scripts or to preserve back-compat for scripts not covered by existing snapshot tests.
- trade-offs: Increases parser surface area unnecessarily when the goal is to guide contributors correctly via documentation.

## ✅ Decision
Chosen Option A. Updating the documentation directly is the fastest, lowest-risk approach to fixing the factual drift without altering the current CLI interface.

## 🧱 Changes made (SRP)
- `docs/debugging.md`: Replaced `--path .` with positional argument `.`.
- `docs/debugging.md`: Replaced `--out` with `--output-dir` for `tokmd run`.
- `docs/debugging.md`: Replaced `--out` with `--artifacts-dir` for `tokmd cockpit`.

## 🧪 Verification receipts
```text
$ cargo run -p tokmd -- run . --output-dir target/tokmd-debug
Writing run artifacts to: target/tokmd-debug
$ cargo run -p tokmd -- analyze . --format json
[JSON output omitted]
$ cargo run -p tokmd -- cockpit --base origin/main --head HEAD --artifacts-dir target/cockpit-debug
[JSON output omitted]
$ cargo xtask docs --check
Documentation is up to date.
$ cargo fmt -- --check
$ cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Docs patch
- Blast radius: docs
- Risk class: low (no behavior change)
- Rollback: git revert
- Gates run: cargo xtask docs --check, cargo fmt, cargo clippy, cargo test, targeted command execution

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
