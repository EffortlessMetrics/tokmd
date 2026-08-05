## Options Considered

### Option A: Add integration test for `resolve_lang_with_config` (Recommended)
- **What it is**: Implement `test_resolve_lang_with_config` in `crates/tokmd/tests/config_resolution.rs`.
- **Why it fits this repo and shard**: The `interfaces` shard explicitly covers `crates/tokmd-config` and `crates/tokmd/src/config.rs`. Adding missing BDD/integration coverage for this critical path addresses the highest ranked target ("1) missing BDD/integration coverage for an important path") for the Specsmith persona.
- **Trade-offs**:
  - Structure: High. Ensures that CLI overrides of `lang` command settings correctly take precedence over TOML and JSON configurations.
  - Velocity: High. A straightforward addition that plugs a direct gap in `tokmd`'s testing matrix.
  - Governance: High. Locks in determinism for how configuration sources are prioritized.

### Option B: Add unit test for `resolve_profile`
- **What it is**: Implement tests around profile lookup (`resolve_profile` in `crates/tokmd/src/config.rs`).
- **When to choose it instead**: If profile resolution had complex fallback logic.
- **Trade-offs**:
  - `resolve_profile` is a simpler map lookup logic, less critical than the tiered `resolve_*_with_config` mechanisms that reconcile CLI, JSON profile, and TOML.

## ✅ Decision
Option A. `resolve_lang_with_config` is the key function defining precedence for the `lang` command. While `export` and `module` have their `_with_config` variants tested, `lang` does not. We'll add this test to ensure complete and robust coverage across all command configuration resolvers.
