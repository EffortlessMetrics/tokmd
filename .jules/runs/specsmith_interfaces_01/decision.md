# Specsmith Decision

## Option A (Add regression test for config resolution defaults and override edge cases)
- Expand `crates/tokmd/tests/config_resolution.rs` to include tests that check `resolve_module` and `resolve_analyze` just like `resolve_lang`, to lock in the override logic where CLI parameters overwrite profile settings, which map to `cli::Profile` config values.
- Adds test coverage that spans CLI options -> Default Profiles -> Explicit Profiles resolution logic to guarantee we do not silently break configuration precedence.

## Option B (Improve config test for JSON vs Tsv output to include bad profiles)
- Instead of just checking basic overriding in `config_resolution.rs`, we could add an integration test that passes an invalid TOML file.

## Decision
I choose **Option A**. The issue requests missing BDD/integration coverage for an important path or an edge-case regression locked in by tests in `interfaces` (Config, core facade, and CLI interfaces). The file `config_resolution.rs` only tests `resolve_lang`, but ignores `resolve_module` and `resolve_analyze`, leaving their configuration override logic uncovered. Adding those directly locks in configuration fallback paths in a unified manner.
