## 💡 Summary
Added an explicit fuzz target for the FFI in-memory input path parsing layer. This hardens the interface boundaries for Node.js and Python bindings against adversarial file paths.

## 🎯 Why
The parser validation logic in `crates/tokmd-core/src/ffi/inputs.rs` includes numerous path defense rules (no parent traversal, no absolute paths, no Windows drive paths, no control chars). While `fuzz_run_json.rs` guarantees the core interface won't panic on arbitrary payloads, it doesn't deterministically generate valid top-level schema that deeply exercises the inner nested logic in `inputs.rs`. Adding an explicit fuzz target for this logic helps lock in safety.

## 🔎 Evidence
- `crates/tokmd-core/src/ffi/inputs.rs`
- Missing specific coverage in existing fuzzers (checked `fuzz/fuzz_targets/fuzz_run_json.rs`)
- `cargo check --manifest-path fuzz/Cargo.toml --all-features` succeeds

## 🧭 Options considered
### Option A
- Improve `ScanOptions` redaction invariants by mutating `fuzz_scan_args.rs`.
- While useful, the existing `fuzz_scan_args.rs` is already highly robust at proving slash normalization and determinism.
- Trade-offs: Lower velocity, less direct impact on untrusted external interfaces.

### Option B (recommended)
- Add a new fuzzer specifically constructing JSON objects targeted at `ffi::inputs` inside `tokmd-core`.
- This ensures path edge cases (e.g., zero-width spaces, special Windows paths) do not panic or hang the underlying implementation when parsing the FFI arguments.
- Trade-offs: Simple, minimal structural change, directly addresses the fuzz-ability of the input parser surface.

## ✅ Decision
Chosen Option B. Added `fuzz_in_memory_inputs.rs` and registered it in `fuzz/Cargo.toml`.

## 🧱 Changes made (SRP)
- `fuzz/Cargo.toml`: Add `fuzz_in_memory_inputs` target declaration.
- `fuzz/fuzz_targets/fuzz_in_memory_inputs.rs`: Add new fuzz logic validating `run_json` path extraction constraints.

## 🧪 Verification receipts
```text
cargo check --manifest-path fuzz/Cargo.toml --all-features
cargo test -p tokmd-core
```

## 🧭 Telemetry
- Change shape: New proof surface
- Blast radius: None (fuzzing only)
- Risk class: Safe / Tools
- Rollback: Revert PR
- Gates run: fuzz tooling, `cargo check`, `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`

## 🔜 Follow-ups
None.
