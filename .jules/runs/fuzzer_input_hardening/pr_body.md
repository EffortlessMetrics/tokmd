## 💡 Summary
Extended deterministic proof coverage for CLI argument parsing. Added test cases to ensure that numeric flags using the `value_parser` (specifically `--max-commits` and `--max-commit-files`) gracefully reject invalid UTF-8 bytes with a typed `InvalidUtf8` error instead of panicking, hardening the input surface.

## 🎯 Why
Fuzz coverage found that clap parsing with custom `value_parser` configurations might panic when given raw bytes that cannot be interpreted as valid UTF-8 strings. Since live fuzzing tools (`cargo fuzz`) might not be available across all environments, deterministic fuzz regressions lock in edge-case stability.

## 🔎 Evidence
- `crates/tokmd/tests/cli_parser_fuzz_regression.rs`
- Observed behavior: Adding invalid UTF-8 arguments to `--max-commits` and `--max-commit-files` now correctly triggers a clap `ErrorKind::InvalidUtf8` response instead of crashing the parser.
- Receipt: `CI=true cargo test -p tokmd --verbose` passed successfully.

## 🧭 Options considered
### Option A (recommended)
- what it is: Extend the existing `cli_parser_fuzz_regression.rs` suite with explicit test cases for `--max-commits` and `--max-commit-files`.
- why it fits this repo and shard: It aligns with the existing practice of extracting deterministic fuzz properties into standard tests when live `cargo fuzz` isn't viable in CI or local environments.
- trade-offs: Structure / Velocity / Governance: High velocity, low risk, directly fixes proof drift around input hardening.

### Option B
- what it is: Introduce a new fuzz target specifically feeding raw `&[u8]` inputs to the CLI parser (`Cli::try_parse_from`).
- when to choose it instead: When fuzzing toolchains are strictly guaranteed to be present and reliable.
- trade-offs: Increases maintenance burden for fuzz environments and may fail to run consistently based on host tooling constraints.

## ✅ Decision
Option A was chosen. Adding deterministic tests guarantees immediate proof execution during standard `cargo test` without relying on external fuzzing toolchains, satisfying the fallback expectations for the `fuzz` gate.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_parser_fuzz_regression.rs`: Added `cli_parser_rejects_invalid_utf8_max_commits_value` and `cli_parser_rejects_invalid_utf8_max_commit_files_value`.

## 🧪 Verification receipts
```text
cargo build --verbose
CI=true cargo test -p tokmd --verbose
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Proof improvement patch
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Minimal (only tests added)
- Risk class + why: Very low (does not alter production behavior, solely bolsters deterministic proof)
- Rollback: Safely revertable by removing added test cases
- Gates run: `build`, `test`, `fmt`, `clippy`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`

## 🔜 Follow-ups
None required.
