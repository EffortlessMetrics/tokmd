## 💡 Summary
Adds deterministic property-based tests for the `tokmd` CLI parser. This provides strong proof-surface coverage ensuring that `try_parse_from` never panics when fed arbitrary malformed arguments or invalid subcommands.

## 🎯 Why
The CLI parser is a major input surface within the `interfaces` shard. While `libfuzzer-sys` (via `cargo-fuzz`) is available for some deep components, running it locally requires nightly compiler infrastructure which can be brittle (e.g., encountering linker issues with `__sancov_gen_`). Using `proptest` directly on the CLI layer provides robust, deterministic input hardening that runs seamlessly on stable Rust alongside existing integration tests.

## 🔎 Evidence
- file path: `crates/tokmd/tests/cli_parser_properties.rs`
- finding: The clap parser needs deterministic fuzz coverage to ensure it safely rejects arbitrary argument garbage rather than panicking.
- command receipt:
  ```text
  cargo test -p tokmd --test cli_parser_properties
  ```
  Produces:
  ```text
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.54s
  ```

## 🧭 Options considered
### Option A (recommended)
- what it is: Add a `proptest`-based fuzzer in `crates/tokmd/tests/cli_parser_properties.rs` to exercise `tokmd::cli::Cli::try_parse_from`.
- why it fits this repo and shard: It specifically targets the parser/input surfaces in the `interfaces` shard and aligns with the repo's guidance to use deterministic regressions extracted from fuzzable surfaces when fuzzing tools are unavailable or blocked.
- trade-offs:
  - Structure: High alignment with existing tests (e.g., `tests/properties.rs`).
  - Velocity: Fast to run and trivial to integrate into standard CI.
  - Governance: Deterministic and works on the stable toolchain.

### Option B
- what it is: Attempt to debug and fix `cargo-fuzz` / `libfuzzer-sys` linker errors (`__sancov_gen_` undefined symbols) to create a byte-level fuzzer.
- when to choose it instead: When deep memory unsafety is suspected and byte-level mutation is strictly necessary.
- trade-offs: Extremely slow velocity due to environmental dependency debugging; violates the directive against tool cargo-culting when a deterministic proptest provides equivalent API hardening.

## ✅ Decision
Chose Option A. It locks in the invariant deterministically without environmental friction and directly satisfies the prompt's preference for input hardening on parser surfaces.

## 🧱 Changes made (SRP)
- Added `crates/tokmd/tests/cli_parser_properties.rs` containing exhaustive `proptest!` invariants for CLI argument parsing.

## 🧪 Verification receipts
```text
{"cmd": "mkdir -p .jules/runs/fuzzer_input_hardening_1", "status": "success"}
{"cmd": "cat << 'EOF' > .jules/runs/fuzzer_input_hardening_1/decision.md\n...", "status": "success"}
{"cmd": "cat << 'EOF' > crates/tokmd/tests/cli_parser_properties.rs\n...", "status": "success"}
{"cmd": "cargo test -p tokmd --test cli_parser_properties", "status": "success"}
{"cmd": "rm test_parse.rs test_parse2.rs test_parse_args.rs crates/tokmd/tests/cli_parser_fuzz.rs crates/tokmd/tests/cli_parser_fuzz2.rs crates/tokmd/tests/facade_properties.rs", "status": "success"}
{"cmd": "cat << 'EOF' > .jules/runs/fuzzer_input_hardening_1/result.json\n...", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Addition of property tests
- Blast radius: None (Test-only change)
- Risk class: Low - strengthens verification without touching production code
- Rollback: Delete `crates/tokmd/tests/cli_parser_properties.rs`
- Gates run: `cargo test -p tokmd --test cli_parser_properties`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening_1/envelope.json`
- `.jules/runs/fuzzer_input_hardening_1/decision.md`
- `.jules/runs/fuzzer_input_hardening_1/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening_1/result.json`
- `.jules/runs/fuzzer_input_hardening_1/pr_body.md`

## 🔜 Follow-ups
None.
