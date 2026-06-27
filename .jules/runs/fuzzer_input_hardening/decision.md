# Option A: Improve `ScanOptions` redaction invariants and fuzz target robustness

While reviewing `fuzz_scan_args.rs`, `fuzz_toml_config.rs`, and the `fuzz_json_types.rs` fuzzer, we notice they do an excellent job asserting determinism, but we can make them stronger. In particular:

1. `fuzz_json_types.rs` misses enforcing proper deserialization bounds on `RunReceipt`, `CockpitReceipt` and `LangReceipt`. We can strengthen `fuzz_json_types.rs` by fully validating parsed schema versions and array invariants.
2. `fuzz_toml_config.rs` verifies some fields, but `redact_mode` and complex sub-structs can be better verified during roundtrips.

However, since there is a `fuzz_scan_args.rs` validating `scan_args` logic, let's explore if `fuzz_scan_args.rs` actually proves its own coverage is robust by introducing additional paths and exclusions properties.

# Option B: Replay Corpus & Add explicit harness checks around FFI inputs validation

The assignment is around `interfaces` and "Improve fuzzability or input hardening around parser/input surfaces" for `Fuzzer` persona. The parser in `crates/tokmd-core/src/ffi/inputs.rs` does extensive `validate_in_memory_input_path`. Let's add a robust fuzz target for `parse_in_memory_inputs` or enhance the existing `fuzz_run_json.rs` to intentionally generate these nested edge cases so we can guarantee we never panic on deep FFI data.

Specifically, `fuzz_run_json.rs` already provides `run_json` arbitrary coverage, but we can explicitly fuzz the in-memory inputs extractor!

Let's look at `fuzz_run_json.rs`. It just blindly calls `run_json(mode, args_json)`. If we add a dedicated fuzzer `fuzz_in_memory_inputs.rs` that takes arbitrary bytes, constructs a JSON object resembling `{"inputs": [{"path": "<arbitrary>", "text": "..."}]}`, and passes it to `parse_in_memory_inputs`, we explicitly harden the path validation loop.

# Decision

**Option B** is more direct. We will create a new fuzz target `fuzz_in_memory_inputs.rs` specifically for `crates/tokmd-core/src/ffi/inputs.rs` logic to harden the `parse_in_memory_inputs` and `validate_in_memory_input_path` surfaces. This will guarantee `validate_in_memory_input_path` does not panic or hang on arbitrary adversarial paths containing control characters, Windows drive prefixes, empty strings, dots, or invalid UTF-8 (when valid as JSON). We will register it in `fuzz/Cargo.toml`.
