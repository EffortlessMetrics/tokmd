## 💡 Summary
Added missing executable doctests to `tokmd` configuration resolution APIs in `crates/tokmd/src/config.rs`.

## 🎯 Why
The `docs-executable` gate profile requires documentation to include executable examples to prevent factual drift. The core `ConfigContext` and `ResolvedConfig` methods lacked any executable examples.

## 🔎 Evidence
- `crates/tokmd/src/config.rs`
- Methods like `ConfigContext::get_toml_view`, `ConfigContext::get_json_profile`, `load_config`, and `ResolvedConfig::*` getters had no doctests.
- `cargo test -p tokmd --doc` showed only 12 tests running initially.

## 🧭 Options considered
### Option A (recommended)
- Add complete doctest examples to `ConfigContext` and `ResolvedConfig` methods.
- Ensures all configuration resolution logic is executably covered in docs.
- Trade-offs: Increases LOC in `config.rs`, but guarantees docs don't drift.

### Option B
- Ignore the missing doctests and fix the legacy JSON profile resolution examples.
- Does not fix the main missing coverage block.

## ✅ Decision
Chose Option A to align with the Librarian Prover persona and enforce executable examples across the `interfaces` shard.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/config.rs`: Added 13 new doctests for `ConfigContext` and `ResolvedConfig` methods.

## 🧪 Verification receipts
```text
running 25 tests
test crates/tokmd/src/config.rs - config::ConfigContext (line 16) ... ok
test crates/tokmd/src/config.rs - config::ConfigContext::get_toml_view (line 37) ... ok
test crates/tokmd/src/cli/parser.rs - cli::parser::Cli (line 81) ... ok
test crates/tokmd/src/config.rs - config::ConfigContext::get_json_profile (line 51) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig (line 223) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::children (line 329) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::files (line 279) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::format (line 247) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::meta (line 396) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::max_rows (line 363) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::min_code (line 346) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::redact (line 380) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::module_depth (line 312) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::module_roots (line 295) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig<'_>::top (line 263) ... ok
test crates/tokmd/src/config.rs - config::get_profile_name (line 158) ... ok
test crates/tokmd/src/config.rs - config::load_config (line 66) ... ok
test crates/tokmd/src/config.rs - config::resolve_config (line 413) ... ok
test crates/tokmd/src/config.rs - config::resolve_profile (line 185) ... ok
test crates/tokmd/src/config/resolve/export.rs - config::resolve::export::resolve_export (line 14) ... ok
test crates/tokmd/src/config/resolve/export.rs - config::resolve::export::resolve_export_with_config (line 92) ... ok
test crates/tokmd/src/config/resolve/lang.rs - config::resolve::lang::resolve_lang_with_config (line 65) ... ok
test crates/tokmd/src/config/resolve/lang.rs - config::resolve::lang::resolve_lang (line 14) ... ok
test crates/tokmd/src/config/resolve/module.rs - config::resolve::module::resolve_module (line 14) ... ok
test crates/tokmd/src/config/resolve/module.rs - config::resolve::module::resolve_module_with_config (line 76) ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## 🧭 Telemetry
- Change shape: Missing executable coverage addition
- Blast radius: Docs only
- Risk class: Low
- Rollback: Revert
- Gates run: `cargo test -p tokmd --doc`

## 🗂️ .jules artifacts
- `.jules/runs/run-librarian-interfaces/envelope.json`
- `.jules/runs/run-librarian-interfaces/decision.md`
- `.jules/runs/run-librarian-interfaces/receipts.jsonl`
- `.jules/runs/run-librarian-interfaces/result.json`
- `.jules/runs/run-librarian-interfaces/pr_body.md`

## 🔜 Follow-ups
None
