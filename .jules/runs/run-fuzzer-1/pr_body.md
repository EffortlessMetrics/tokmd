## 💡 Summary
Added deterministic fuzz regression tests to ensure invalid UTF-8 inputs are cleanly rejected by CLI numeric flags. This is a proof-improvement patch locking down parser invariants. Also added a friction item for `cargo-llvm-cov` being absent in CI when the proof executor tries to run it.

## 🎯 Why
CLI flags using custom `value_parser` logic for numbers (like `--max-commits` and `--max-commit-files`) must cleanly return `InvalidUtf8` errors when fed malformed byte sequences instead of panicking. Adding explicit deterministic tests for these cases guarantees this invariant remains intact against future `clap` or standard library changes.

## 🔎 Evidence
- `crates/tokmd/tests/cli_parser_fuzz_regression.rs`
- Ran deterministic test suite and verified standard `clap::error::ErrorKind::InvalidUtf8` errors are raised when `--max-commits`, `--max-commit-files`, and `--max-file-tokens` receive non-UTF8 `OsString` bytes.

## 🧭 Options considered
### Option A (recommended)
- Add deterministic regression tests for `max-commits`, `max-commit-files`, and `max-file-tokens` into `crates/tokmd/tests/cli_parser_fuzz_regression.rs` using raw invalid `OsString` bytes.
- Fits the `interfaces` shard and `fuzzer` persona well by locking in hard-to-reach edge cases without depending on a live fuzz run.
- Trade-offs: Low risk, high value for preventing regressions.

### Option B
- Add explicit UTF-8 validation in `tokmd/src/cli/parser/validate.rs` functions.
- When to choose: If clap failed to correctly reject invalid utf8 for `String`-typed flags before they reach custom value parsers.
- Trade-offs: Unnecessary because clap already intercepts invalid UTF-8 for these flags, so adding redundant code would just add noise.

## ✅ Decision
Chose Option A to lock in deterministic proof of safe rejection for numeric flags via the existing fuzz regression suite.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_parser_fuzz_regression.rs`: Added `cli_parser_rejects_invalid_utf8_numeric_flags` test case covering `--max-commits`, `--max-commit-files`, and `--max-file-tokens`.

## 🧪 Verification receipts
```text
cargo test --test cli_parser_fuzz_regression
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Test additions
- Blast radius: None (test only)
- Risk class: Low
- Rollback: Revert the test file changes.
- Gates run: Fuzz gate (fallback: deterministic regression tests, cargo build/test, clippy).

## 🗂️ .jules artifacts
- `.jules/runs/run-fuzzer-1/envelope.json`
- `.jules/runs/run-fuzzer-1/decision.md`
- `.jules/runs/run-fuzzer-1/receipts.jsonl`
- `.jules/runs/run-fuzzer-1/result.json`
- `.jules/runs/run-fuzzer-1/pr_body.md`
- `.jules/friction/open/cargo_llvm_cov_missing.md`

## 🔜 Follow-ups
Fix `cargo-llvm-cov` installation in CI to prevent `xtask proof` executor panics.
