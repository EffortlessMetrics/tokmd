## 💡 Summary
Added an executable doctest for `tokmd::config::load_config` to demonstrate typical `ConfigContext` usage. Fixed incorrect module path references in the `resolve_*_with_config` doctests, ensuring they compile and reflect reality.

## 🎯 Why
The `Librarian` persona prioritizes executable examples that can't silently drift. The config resolution module is a key interface. `load_config` lacked an example entirely, and several existing doctests referenced `tokmd::resolve_config` instead of the correct `tokmd::config::resolve_config` path. This could confuse contributors or AI agents referencing the code.

## 🔎 Evidence
- **File:** `crates/tokmd/src/config.rs`
- **Finding:** `load_config` had no doctest. The examples for `resolve_lang_with_config`, `resolve_module_with_config`, etc. contained broken imports like `use tokmd::{resolve_config, ConfigContext};` or directly called `tokmd::resolve_config`.
- **Receipt:** Doctests failed to compile initially with the mocked patch, confirming the paths needed fixing to pass `cargo test --doc`.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Add a missing `load_config` doctest and fix the incorrect paths in existing doctests.
- **Why it fits this repo and shard:** Focuses strictly on fixing factual drift in the `interfaces` shard while satisfying the `docs-executable` gate.
- **Trade-offs:** Low risk, directly improves documentation reliability.

### Option B
- **What it is:** Refactor the config resolution logic to remove duplication across `resolve_lang_with_config` and others.
- **When to choose it instead:** If the logic was the primary issue rather than missing/drifting docs.
- **Trade-offs:** Too high risk for a documentation and examples pass.

## ✅ Decision
Option A was chosen to fulfill the 'Librarian' mission of improving factual docs and ensuring examples are executable.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/config.rs`: Added doctest to `load_config`. Fixed module paths for `resolve_config`, `resolve_lang_with_config`, `resolve_module_with_config`, and `resolve_export_with_config`.

## 🧪 Verification receipts
```text
$ cargo test --doc -p tokmd
running 12 tests
test crates/tokmd/src/config.rs - config::ResolvedConfig (line 210) ... ok
test crates/tokmd/src/config.rs - config::get_profile_name (line 145) ... ok
test crates/tokmd/src/config.rs - config::load_config (line 46) ... ok
test crates/tokmd/src/config.rs - config::ConfigContext (line 11) ... ok
test crates/tokmd/src/config.rs - config::resolve_config (line 310) ... ok
test crates/tokmd/src/config.rs - config::resolve_export (line 616) ... ok
test crates/tokmd/src/config.rs - config::resolve_lang (line 337) ... ok
test crates/tokmd/src/config.rs - config::resolve_export_with_config (line 699) ... ok
test crates/tokmd/src/config.rs - config::resolve_lang_with_config (line 394) ... ok
test crates/tokmd/src/config.rs - config::resolve_module (line 464) ... ok
test crates/tokmd/src/config.rs - config::resolve_module_with_config (line 532) ... ok
test crates/tokmd/src/config.rs - config::resolve_profile (line 172) ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: Documentation update (doctests)
- Blast radius: Docs only
- Risk class: Safe
- Rollback: `git checkout crates/tokmd/src/config.rs`
- Gates run: `cargo test --doc -p tokmd`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None
