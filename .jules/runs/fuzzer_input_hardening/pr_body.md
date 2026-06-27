## 💡 Summary
Hardens the FFI configuration boundary by strictly asserting that JSON setting blocks are valid objects. Previously, non-object JSON values (like strings or arrays) passed as configuration blocks would silently fall back to `unwrap_or(args)`, bypassing validation and potentially causing unexpected downstream behavior or deterministic failures.

## 🎯 Why
During exploration and fuzzy input testing of the FFI interface (`run_json`), it was discovered that the parser implicitly permitted passing non-object values to settings keys (e.g. `{"lang": "not an object"}`) by falling back to the entire root args object. This lax behavior breaks the assumption of strict configuration payloads at the FFI trust boundary and complicates downstream fuzzing by creating ambiguous error conditions.

## 🔎 Evidence
Minimal proof:
- file paths: `crates/tokmd-core/src/ffi/settings_parse.rs` and `crates/tokmd-core/src/ffi/parse.rs`
- observed behavior: `let obj = args.get("lang").unwrap_or(args);` would blindly accept any JSON value type (like strings or arrays) instead of strictly returning an error.
- command receipt: A deterministic regression test added in `crates/tokmd-core/tests/regression_settings_fuzz.rs` captures this edge case, proving the fix.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add strict `is_object()` validation to all settings parser surfaces (`scan`, `lang`, `module`, `export`, `analyze`, `cockpit`, `diff`) in `tokmd-core/src/ffi/settings_parse.rs` and `parse.rs` when accessing nested configuration fields. Include a regression test to lock this behavior in.
- why it fits this repo and shard: Strongly types the trust boundary against unexpected JSON inputs, directly aligning with the `fuzzer` persona's mandate for input hardening on the `interfaces` shard.
- trade-offs: Structure is improved by tightening validation, Governance is improved by explicitly managing edge cases, and Velocity is unimpacted due to the localized nature of the change.

### Option B
- what it is: Only fix the `scan` settings parser, as it's the most foundational block, and leave the mode-specific ones alone.
- when to choose it instead: If the mode-specific settings blocks are somehow validated externally before reaching the core, or if the risk is deemed too low for individual modes.
- trade-offs: Leaves a gap in the FFI surface where other modes (`lang`, `export`, etc.) could still misbehave or silently ignore malformed non-object inputs.

## ✅ Decision
Option A was chosen. The anti-pattern of using `args.get(field).unwrap_or(args)` allows non-object inputs to silently bypass validation or cause unexpected behavior when nested fields are subsequently queried. Enforcing `is_object()` explicitly hardens the trust boundary, preventing deterministic bugs or fuzzing failures.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/ffi/parse.rs`: Added `is_object()` validation and `Result` wrapping to `scan_arg_object`. Added test for new error case.
- `crates/tokmd-core/src/ffi/settings_parse.rs`: Added explicit `is_object()` validation to `parse_scan_settings`, `parse_lang_settings`, `parse_module_settings`, `parse_export_settings`, `parse_analyze_settings`, `parse_cockpit_settings`, and `parse_diff_settings`.
- `crates/tokmd-core/tests/regression_settings_fuzz.rs`: Added a deterministic regression test confirming invalid configuration inputs yield a clear `invalid_settings` error code across all modes.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-core --test regression_settings_fuzz
running 1 test
test test_regression_settings_fuzz ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo clippy -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 51.54s

$ CI=true cargo test -p tokmd-core --verbose
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

## 🧭 Telemetry
- Change shape: Hardening / input validation patch
- Blast radius: FFI boundary (API) parsing logic.
- Risk class + why: Low. Simply enforces an existing structural assumption earlier and more strictly, returning descriptive errors instead of silent fallback.
- Rollback: `git revert`
- Gates run: `cargo build`, `CI=true cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`
- `.jules/runs/fuzzer_input_hardening/result.json`

## 🔜 Follow-ups
- Check other config endpoints (like TOML/CLI mapping) for similar unvalidated type assumptions at the boundary.
