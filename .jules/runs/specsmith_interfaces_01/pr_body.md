## 💡 Summary
Added integration and BDD-style unit tests for the missing configuration resolution paths (`resolve_module`, `resolve_export`, and `resolve_analyze`) in `crates/tokmd/tests/config_resolution.rs`. This locks in configuration fallback paths where CLI arguments overwrite config profile values correctly.

## 🎯 Why
The issue required finding one missing BDD/integration scenario coverage in the interfaces shard (Config, core facade, and CLI interfaces). Only `resolve_lang` was being tested for correctness in configuration resolution logic. Without proper regression test locks on `resolve_module`, `resolve_export`, and `resolve_analyze`, we risk silent failures during refactoring where the CLI falls back to profile configurations instead of overriding them.

## 🔎 Evidence
- File path: `crates/tokmd/tests/config_resolution.rs`
- Finding: Previous tests only covered `test_resolve_lang_*` cases. Adding tests for `test_resolve_module_*`, `test_resolve_export_*`, and `test_resolve_analyze_*` verifies that configuration defaults correctly fallback.
- Commands run: `cargo test -p tokmd --test config_resolution`

## 🧭 Options considered
### Option A (recommended)
- Add regression tests for config resolution defaults and override edge cases explicitly in `crates/tokmd/tests/config_resolution.rs`.
- Why it fits: Matches the focus on fixing BDD/integration logic for configuration overrides within the bounded `interfaces` shard.
- Trade-offs: Minor structural duplication among tests, but provides explicit boundary validation velocity.

### Option B
- Add failing integration tests using full broken TOML profile configs rather than building Profiles inline.
- When to choose: Helpful when verifying TOML parser behaviour, rather than testing precedence handling in structs.
- Trade-offs: Slower test execution, introduces IO assumptions.

## ✅ Decision
Chose **Option A**, directly expanding `crates/tokmd/tests/config_resolution.rs`. It provides robust and targeted unit tests directly to the configuration struct fallback parameters for `resolve_module`, `resolve_export`, and `resolve_analyze`.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/config_resolution.rs`: Added 7 test cases (`test_resolve_module_no_args_no_profile`, `test_resolve_module_cli_overrides_profile`, `test_resolve_module_profile_overrides_default`, `test_resolve_export_no_args_no_profile`, `test_resolve_export_cli_overrides_profile`, `test_resolve_export_profile_overrides_default`, and `test_resolve_analyze_no_args_no_profile`).

## 🧪 Verification receipts
```text
running 11 tests
test test_resolve_analyze_no_args_no_profile ... ok
test test_resolve_export_cli_overrides_profile ... ok
test test_resolve_export_no_args_no_profile ... ok
test test_resolve_export_profile_overrides_default ... ok
test test_resolve_lang_cli_overrides_profile ... ok
test test_resolve_lang_no_args_no_profile ... ok
test test_resolve_lang_partial_overrides ... ok
test test_resolve_lang_profile_overrides_default ... ok
test test_resolve_module_cli_overrides_profile ... ok
test test_resolve_module_no_args_no_profile ... ok
test test_resolve_module_profile_overrides_default ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: New unit test targets for configuration bounds.
- Blast radius: `tests` execution only (no API, schema, concurrency, or dependencies touched).
- Risk class: Low risk. Verification of logic defaults lock.
- Rollback: Safely reverted by reverting the commit.
- Gates run: `cargo test -p tokmd --test config_resolution`, `cargo test -p tokmd-core && cargo test -p tokmd-wasm && npm --prefix web/runner test && npm --prefix crates/tokmd-node test && cargo test -p tokmd-python`.

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces_01/envelope.json`
- `.jules/runs/specsmith_interfaces_01/decision.md`
- `.jules/runs/specsmith_interfaces_01/receipts.jsonl`
- `.jules/runs/specsmith_interfaces_01/result.json`
- `.jules/runs/specsmith_interfaces_01/pr_body.md`

## 🔜 Follow-ups
None.
