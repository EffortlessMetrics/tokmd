## 💡 Summary
This PR adds edge-case integration tests locking in the behavior of the `tokmd` CLI when provided a non-existent file path without an explicit subcommand. It proves that the path is evaluated correctly as a "Path not found" error, instead of being inaccurately swallowed by the typo/unknown-subcommand logic.

## 🎯 Why
While the internal hint resolution logic in `error_hints.rs` is covered by unit tests, there was missing end-to-end regression coverage to prove that `tokmd missing/file.rs` implicitly defaults to the `lang` subcommand and surfaces the correct "Path not found" error instead of rewriting the error to "Unrecognized subcommand". Locking this behavior down prevents regressions where path-like arguments get swallowed by over-eager subcommand typo correctors.

## 🔎 Evidence
- File paths: `crates/tokmd/tests/cli_e2e_w65.rs`, `crates/tokmd/tests/cli_error_paths_w51.rs`
- Missing coverage for `tokmd <non-existent-path>` (with slash) as an implicit `lang` fallback.
- Test `nonexistent_file_path_keeps_path_not_found_error_output` successfully runs the CLI process and asserts the correct error boundaries.

## 🧭 Options considered
### Option A (recommended)
- Add explicit integration/BDD-style tests ensuring `tokmd missing/file.rs` outputs `Path not found: missing/file.rs` without being rewritten to `Unrecognized subcommand`.
- Fits the `interfaces` shard by increasing CLI edge-case polish and regression coverage (Specsmith core directive).
- Trade-offs: Structure is solid, high velocity, strictly local to `crates/tokmd/tests`.

### Option B
- Add unit tests inside `crates/tokmd/src/error_hints.rs` rather than e2e process tests.
- When to choose: If the failure is deep in the string manipulation and doesn't depend on `clap` routing.
- Trade-offs: Doesn't prove the full path through `clap` to the default `lang` subcommand error fallback.

## ✅ Decision
Option A was chosen because proving the full integration path from CLI args to standard error stream is the strongest guarantee against regressions for bare paths.

## 🧱 Changes made (SRP)
- Added `implicit_lang_mode_with_nonexistent_file_path_shows_path_not_found` to `crates/tokmd/tests/cli_error_paths_w51.rs`
- Added `nonexistent_file_path_keeps_path_not_found_error_output` to `crates/tokmd/tests/cli_e2e_w65.rs`

## 🧪 Verification receipts
```text
running 1 test
test implicit_lang_mode_with_nonexistent_file_path_shows_path_not_found ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.01s

running 1 test
test nonexistent_file_path_keeps_path_not_found_error_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: Test additions
- Blast radius: `tests` only, no risk to prod behavior.
- Risk class: Low, purely regression lock-in tests.
- Rollback: Revert the test additions.
- Gates run: `cargo test --test cli_e2e_w65`, `cargo test --test cli_error_paths_w51`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces_01/envelope.json`
- `.jules/runs/specsmith_interfaces_01/decision.md`
- `.jules/runs/specsmith_interfaces_01/receipts.jsonl`
- `.jules/runs/specsmith_interfaces_01/result.json`
- `.jules/runs/specsmith_interfaces_01/pr_body.md`

## 🔜 Follow-ups
None
