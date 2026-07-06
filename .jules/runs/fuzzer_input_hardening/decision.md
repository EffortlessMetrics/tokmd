# Option A: Extend InvalidUtf8 regression tests for numeric flags
We can improve deterministic proof for the CLI parsing by adding an invalid UTF-8 regression test for `--max-commits` and `--max-commit-files`. This validates that `value_parser` works correctly in conjunction with non-UTF8 input and does not panic, matching the logic established in `cli_parser_fuzz_regression.rs` for `--exclude`.

# Option B: Add a fuzzer target for CLI argument parsing
We can write a new fuzz target under `fuzz/fuzz_targets/fuzz_cli.rs` that feeds arbitrary `&[u8]` strings to `Cli::try_parse_from`, validating that it rejects invalid input deterministically.

# Decision: Option A
Given that fuzz targets may fail in CI/local execution natively depending on tooling constraints (as proven by our inability to run `cargo fuzz` locally), the instruction suggests prioritizing deterministic proofs or extending corpus cases. Furthermore, extending `crates/tokmd/tests/cli_parser_fuzz_regression.rs` clearly fulfills the fallback expectation: "otherwise deterministic regression or harness commands" and directly applies input hardening logic. Option A is chosen.
