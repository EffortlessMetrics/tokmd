## 💡 Summary
Learning PR: Explored hardening the FFI configuration boundary to assert JSON setting blocks are valid objects, but found this work was superseded by #2713.

## 🎯 Why
The FFI parser implicitly permitted passing non-object values to settings keys (e.g. `{"lang": "not an object"}`) by falling back to the entire root args object. This lax behavior breaks the assumption of strict configuration payloads at the FFI trust boundary and complicates downstream fuzzing by creating ambiguous error conditions.

## 🔎 Evidence
Minimal proof:
- file paths: `crates/tokmd-core/src/ffi/settings_parse.rs` and `crates/tokmd-core/src/ffi/parse.rs`
- observed behavior: `let obj = args.get("lang").unwrap_or(args);` would blindly accept any JSON value type (like strings or arrays) instead of strictly returning an error.
- command receipt: A deterministic regression test added in `crates/tokmd-core/tests/regression_settings_fuzz.rs` captures this edge case.

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
Work stopped because this issue is already solved in #2713.

## 🧱 Changes made (SRP)
- Reverted code changes since the fix was redundant.
- Captured friction item for superseded work.

## 🧪 Verification receipts
N/A

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class + why: None
- Rollback: N/A
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening/envelope.json`
- `.jules/runs/fuzzer_input_hardening/decision.md`
- `.jules/runs/fuzzer_input_hardening/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening/pr_body.md`
- `.jules/runs/fuzzer_input_hardening/result.json`
- `.jules/friction/open/superseded-ffi-hardening.md`

## 🔜 Follow-ups
- Check other config endpoints (like TOML/CLI mapping) for similar unvalidated type assumptions at the boundary.
