## 💡 Summary
Added a new `fuzz_cli_parser` target to continuously test the outermost CLI boundary of `tokmd`. This directly hardens the CLI input parsing boundary against panics from pathological input.

## 🎯 Why
The `Cli::try_parse_from` boundary is the first touchpoint for untrusted or fuzzed data when run as a standalone binary. While we have property tests for the CLI (`cli_parser_properties.rs`), they do not have the persistence and coverage-guided depth of a proper `libfuzzer` target. A dedicated fuzz target reduces uncertainty around parser panics.

## 🔎 Evidence
- `fuzz/fuzz_targets/fuzz_cli_parser.rs` was created.
- Command: `cd fuzz && cargo check --bin fuzz_cli_parser --features cli` compiles successfully.

## 🧭 Options considered
### Option A
- Improve `fuzz_toml_config.rs` Corpus Generativity by expanding `fuzz/dict/toml.dict`.
- High structure and velocity, but focuses on an already-fuzzed surface.
- Trade-offs: Structure / Velocity / Governance

### Option B (recommended)
- Add a `fuzz_cli_parser.rs` Target to test the outermost parsing boundary.
- Fits the `fuzzer` persona's mission to improve fuzzability around parser/input surfaces perfectly.
- Trade-offs: Medium velocity, but high structure and direct hardening value.

## ✅ Decision
Option B was chosen because fuzzing the CLI parser boundary is a critical input hardening step that brings a new, untested surface under the fuzzing umbrella.

## 🧱 Changes made (SRP)
- `fuzz/Cargo.toml`: Added `tokmd` dependency and `cli` feature, and registered the new `fuzz_cli_parser` target.
- `fuzz/fuzz_targets/fuzz_cli_parser.rs`: Implemented the new fuzzer target.

## 🧪 Verification receipts
```text
cd fuzz && cargo check --bin fuzz_cli_parser --features cli
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test -p tokmd --verbose
```

## 🧭 Telemetry
- Change shape: New fuzz target
- Blast radius: None (fuzz tooling only)
- Risk class: Low
- Rollback: Revert PR
- Gates run: fuzz (fallback: cargo build, clippy, test)

## 🗂️ .jules artifacts
- `.jules/runs/run_fuzzer_1234/envelope.json`
- `.jules/runs/run_fuzzer_1234/decision.md`
- `.jules/runs/run_fuzzer_1234/receipts.jsonl`
- `.jules/runs/run_fuzzer_1234/result.json`
- `.jules/runs/run_fuzzer_1234/pr_body.md`

## 🔜 Follow-ups
None.
