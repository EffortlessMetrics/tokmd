## Options considered

### Option A (recommended)
- Add missing deterministic regression tests for `max-commits`, `max-commit-files`, and `max-file-tokens` into `crates/tokmd/tests/cli_parser_fuzz_regression.rs`.
- These tests construct `OsString` bytes that are deliberately invalid UTF-8 and pass them to the numeric flags that use a custom `value_parser` (`budget_fraction` and `positive_usize`).
- This ensures that if standard library or `clap` updates change string handling for paths or flags, invalid byte sequences gracefully return an `InvalidUtf8` parse error instead of triggering a panic.
- This fits the `fuzzer` persona well by locking in regression tests against un-fuzzable or hard-to-fuzz invalid inputs for input handling logic without relying on cargo fuzz itself.
- Trade-offs:
  - Structure: Minimal, just adding tests to an existing test file.
  - Velocity: Quick to implement and verify.
  - Governance: Low risk, no production code changes required.

### Option B
- Add explicit UTF-8 validation in `tokmd/src/cli/parser/validate.rs` functions.
- While the inputs are technically strings when they reach `budget_fraction` and `positive_usize` (clap has already done validation depending on flag definitions), we could add additional sanity checks if we don't trust clap's error surfacing.
- When to choose: If clap fails to correctly reject invalid utf8 for String-typed flags, or if the flag definitions had `value_parser = clap::builder::OsStringValueParser::new()` and needed custom logic afterwards.
- Trade-offs: Unnecessary because clap already intercepts invalid UTF-8 on String-typed flags before custom value parsers run, as verified by testing.

## ✅ Decision
Option A was chosen. Adding deterministic tests for the numeric flags in `crates/tokmd/tests/cli_parser_fuzz_regression.rs` guarantees the invariant that invalid UTF-8 is rejected safely (with `InvalidUtf8`), preventing panics, and meeting the specific goal of the `fuzzer` persona for this assignment (deterministic regressions extracted from fuzzable surfaces).
