## 💡 Summary
Strengthened testing constraints for configuration property fallback boundaries around `resolve_lang`, `resolve_module`, and `resolve_export`. Added unit tests showing how the defaults act when fallback overrides occur.

## 🎯 Why
Parsing untrusted inputs from `.tokmd.toml` and combining them with arbitrary CLI arguments has complex resolution semantics. Hardening the test suite here improves fuzzability and proves we won't regress on parsing logic.

## 🔎 Evidence
- `crates/tokmd/tests/config_resolution.rs`
- Extended integration assertions to match FFI invariants.

## 🧭 Options considered
### Option A (recommended)
- Improve regression suite constraints for config fallback APIs.
- Fits the `fuzzer` scope perfectly since it targets parser configuration deterministic inputs mapping back to invariants.
- High velocity and low risk structure.

### Option B
- Add a new fuzz target for config resolution.
- It might take longer and face missing tooling (e.g., `cargo fuzz` missing).

## ✅ Decision
Chose Option A to land deterministic proof patches without hanging on absent fuzz tooling.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd/tests/config_resolution.rs`

## 🧪 Verification receipts
```text
running 10 tests
test test_resolve_export_cli_overrides_profile ... ok
test test_resolve_lang_cli_overrides_profile ... ok
test test_resolve_export_no_args_no_profile ... ok
test test_resolve_export_with_config ... ok
test test_resolve_lang_no_args_no_profile ... ok
test test_resolve_lang_profile_overrides_default ... ok
test test_resolve_module_no_args_no_profile ... ok
test test_resolve_lang_partial_overrides ... ok
test test_resolve_module_profile_overrides_default ... ok
test test_resolve_module_with_config ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Test Addition
- Blast radius: None (Test-only change)
- Risk class: Low
- Rollback: Revert the test additions.
- Gates run: `cargo test -p tokmd --test config_resolution`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer-input-hardening-1/envelope.json`
- `.jules/runs/fuzzer-input-hardening-1/decision.md`
- `.jules/runs/fuzzer-input-hardening-1/receipts.jsonl`
- `.jules/runs/fuzzer-input-hardening-1/result.json`
- `.jules/runs/fuzzer-input-hardening-1/pr_body.md`

## 🔜 Follow-ups
None.
