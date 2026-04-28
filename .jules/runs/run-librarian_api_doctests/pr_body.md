## 💡 Summary
Replaced sparse doctests in `crates/tokmd/src/config.rs` with comprehensive executable examples that demonstrate realistic config resolution and override behavior.

## 🎯 Why
The `resolve_lang_with_config`, `resolve_module_with_config`, and `resolve_export_with_config` functions are key parts of the configuration precedence pipeline (CLI > View Profile > TOML > JSON > Default). The previous doctests initialized `ResolvedConfig` and args using defaults, which didn't actually prove or document how the merge precedence works. Enhancing these doctests aligns with the Librarian/Gatekeeper profile of ensuring correct, executable documentation.

## 🔎 Evidence
- `crates/tokmd/src/config.rs`
- The previous doctests used `ResolvedConfig::default()` and didn't exercise overriding `ResolvedConfig` fields with CLI args.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add comprehensive doctests for `resolve_lang_with_config`, `resolve_module_with_config`, and `resolve_export_with_config` to cover the `ResolvedConfig` resolution precedence (CLI > View Profile > TOML > JSON > Default).
- why it fits this repo and shard: It adds executable coverage to the public configuration resolution API, matching the Gatekeeper/Librarian `docs-executable` profile.
- trade-offs: Structure / Velocity / Governance. Excellent value.

### Option B
- what it is: Rewrite docs and fix prose drift.
- when to choose it instead: If the docs were fundamentally misleading in prose rather than lacking executable examples.
- trade-offs: Anti-drift rules state "Do not land tone-only prose rewrites." Option A is better.

## ✅ Decision
Option A. Added robust doctests that assert how config inputs cascade and interact.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/config.rs`: Replaced doctests for `resolve_lang_with_config`, `resolve_module_with_config`, and `resolve_export_with_config` to mock TOML configurations and simulate overriding them with CLI arguments.

## 🧪 Verification receipts
```text
running 9 tests
test crates/tokmd/src/config.rs - config::ConfigContext (line 11) ... ok
test crates/tokmd/src/config.rs - config::ResolvedConfig (line 143) ... ok
test crates/tokmd/src/config.rs - config::resolve_export (line 551) ... ok
test crates/tokmd/src/config.rs - config::resolve_config (line 243) ... ok
test crates/tokmd/src/config.rs - config::resolve_lang_with_config (line 327) ... ok
test crates/tokmd/src/config.rs - config::resolve_export_with_config (line 634) ... ok
test crates/tokmd/src/config.rs - config::resolve_lang (line 270) ... ok
test crates/tokmd/src/config.rs - config::resolve_module (line 398) ... ok
test crates/tokmd/src/config.rs - config::resolve_module_with_config (line 466) ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## 🧭 Telemetry
- Change shape: doctest enhancements
- Blast radius: API / docs
- Risk class: low (tests only)
- Rollback: revert `config.rs`
- Gates run: `cargo test --doc -p tokmd`, `cargo test -p tokmd --lib`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/run-librarian_api_doctests/envelope.json`
- `.jules/runs/run-librarian_api_doctests/decision.md`
- `.jules/runs/run-librarian_api_doctests/receipts.jsonl`
- `.jules/runs/run-librarian_api_doctests/result.json`
- `.jules/runs/run-librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
