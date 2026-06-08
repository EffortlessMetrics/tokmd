## 💡 Summary
Added a dedicated fuzz target for `tokmd::cli::Cli` to harden the argument parsing surface.

## 🎯 Why
While there are property-based tests in `cli_parser_properties.rs`, `libfuzzer` can execute millions of iterations rapidly and efficiently explore edge cases, uncovering potential panics or regressions that might be missed by simple proptests.

## 🔎 Evidence
Added `fuzz_cli_parser.rs` which splits string inputs into an array of arbitrary tokens to simulate random CLI parameters, and passes them to `Cli::try_parse_from`. Confirmed it successfully builds as a fuzz target via `cargo check --bin fuzz_cli_parser --features cli_parser` in the `fuzz` directory.

## 🧭 Options considered
### Option A (recommended)
- Add a new continuous fuzzing target `fuzz_cli_parser` that directly feeds arbitrary strings into `tokmd::cli::Cli` via `clap`.
- **Why it fits:** The prompt explicitly asks to "Improve fuzzability or input hardening around parser/input surfaces." We already have a property-based test (`cli_parser_properties.rs`), but adding a true fuzz target unlocks libfuzzer coverage over the CLI parser, hardening it against panics on malformed input.
- **Trade-offs:** Minimal addition, sits naturally alongside other fuzz targets. High velocity, straight-forward to implement using existing `tokmd` dependencies. Aligns well with the fuzz-gate profile.

### Option B
- Expand the existing `cli_parser_properties.rs` proptests to cover more edge cases, like empty arguments or massive vectors of string parts.
- **When to choose it instead:** When libfuzzer tooling is not available or if random properties suffice.
- **Trade-offs:** Lower signal compared to actual continuous fuzzing with `cargo-fuzz`. Takes more thought to design good proptest generators. Does not fully leverage the libfuzzer engine we have set up in `fuzz_targets/`.

## ✅ Decision
Option A. It adds a direct fuzz target that works within the libfuzzer ecosystem established in `fuzz/`, providing better long-term security/robustness than a purely random proptest could.

## 🧱 Changes made (SRP)
- `fuzz/Cargo.toml`: Added `cli_parser` feature to support `fuzz_cli_parser`.
- `fuzz/fuzz_targets/fuzz_cli_parser.rs`: Added the actual fuzzing logic using `libfuzzer-sys` to parse the CLI input via `tokmd::cli::Cli`.

## 🧪 Verification receipts
```text
$ cd fuzz && cargo check --bin fuzz_cli_parser --features cli_parser
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

$ CI=true cargo test -p tokmd --verbose
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
```

## 🧭 Telemetry
- Change shape: New Fuzz Target
- Blast radius: None (Test-only change)
- Risk class: Low
- Rollback: Revert `fuzz/fuzz_targets/fuzz_cli_parser.rs` and `fuzz/Cargo.toml`
- Gates run: cargo build, cargo check, cargo test

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`
- `.jules/friction/open/cargo_fuzz_missing.md`
- `.jules/personas/fuzzer/notes/cli_parser_fuzzing.md`

## 🔜 Follow-ups
None.
