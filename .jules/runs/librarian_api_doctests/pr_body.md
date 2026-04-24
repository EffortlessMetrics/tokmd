## 💡 Summary
Added missing executable doctests to public interface functions and structs in `tokmd-core` and `tokmd::config`. This ensures the public API is fully documented with code that is verified continuously by the test suite.

## 🎯 Why
The "docs-executable" gate demands executable examples (doctests) to ensure the documentation does not drift from actual behavior. Many public APIs in `tokmd::config` and the `from_inputs` workflow functions in `tokmd-core` were missing these executable tests entirely.

## 🔎 Evidence
- `crates/tokmd-core/src/lib.rs` and `crates/tokmd/src/config.rs` were missing doctests for `from_inputs` workflows and configuration parsing respectively.
- `cargo test -p tokmd-core --doc` and `cargo test -p tokmd --doc`

## 🧭 Options considered
### Option A (recommended)
- what it is: Add/fix doctests for public functions in `tokmd-core` and `tokmd`
- why it fits this repo and shard: The "docs-executable" gate demands executable examples (doctests) to ensure the documentation does not drift from actual behavior.
- trade-offs: Increases documentation quality and code reliability.

### Option B
- what it is: Fix reference drift in CLI docs
- when to choose it instead: When we find the markdown is severely desynced from `clap`.
- trade-offs: Lower impact than executable docs because `clap` generates help.

## ✅ Decision
Option A was chosen. Adding doctests for common usage of `from_inputs` workflows and config resolvers gives downstream bindings code exact usage guarantees and protects against silent drift.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/lib.rs` - Added doctests for `lang_workflow_from_inputs`, `module_workflow_from_inputs`, and `export_workflow_from_inputs`.
- `crates/tokmd/src/config.rs` - Added doctests for `load_config`, `get_profile_name`, `resolve_profile`, `get_toml_view` and `get_json_profile`.

## 🧪 Verification receipts
```text
running 10 tests
test crates/tokmd-core/src/lib.rs - (line 24) ... ok
test crates/tokmd-core/src/lib.rs - (line 42) ... ok
test crates/tokmd-core/src/ffi.rs - ffi::run_json (line 57) ... ok
test crates/tokmd-core/src/lib.rs - export_workflow_from_inputs (line 292) ... ok
test crates/tokmd-core/src/lib.rs - lang_workflow_from_inputs (line 128) ... ok
test crates/tokmd-core/src/lib.rs - module_workflow_from_inputs (line 205) ... ok
test crates/tokmd-core/src/lib.rs - diff_workflow (line 352) ... ok
test crates/tokmd-core/src/lib.rs - export_workflow (line 257) ... ok
test crates/tokmd-core/src/lib.rs - module_workflow (line 170) ... ok
test crates/tokmd-core/src/lib.rs - scan_workflow (line 727) ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

running 14 tests
test crates/tokmd/src/config.rs - config::ConfigContext::get_json_profile (line 45) ... ok
test crates/tokmd/src/config.rs - config::ConfigContext (line 10) ... ok
test crates/tokmd/src/config.rs - config::ConfigContext::get_toml_view (line 31) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig (line 187) ... ok
test crates/tokmd/src/config.rs - config::get_profile_name (line 141) ... ok
test crates/tokmd/src/config.rs - config::resolve_export (line 567) ... ok
test crates/tokmd/src/config.rs - config::resolve_config (line 287) ... ok
test crates/tokmd/src/config.rs - config::load_config (line 60) ... ok
test crates/tokmd/src/config.rs - config::resolve_lang (line 314) ... ok
test crates/tokmd/src/config.rs - config::resolve_export_with_config (line 653) ... ok
test crates/tokmd/src/config.rs - config::resolve_module (line 429) ... ok
test crates/tokmd/src/config.rs - config::resolve_lang_with_config (line 373) ... ok
test crates/tokmd/src/config.rs - config::resolve_profile (line 164) ... ok
test crates/tokmd/src/config.rs - config::resolve_module_with_config (line 499) ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Docs/test improvement
- Blast radius: none (doc-only change)
- Risk class: minimal (only tests touched)
- Rollback: Revert the commit.
- Gates run: `cargo test --doc`, `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
