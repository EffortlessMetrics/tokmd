## 💡 Summary
Added integration test coverage for `resolve_lang_with_config` in `tokmd` config resolution.

## 🎯 Why
There was a gap in BDD/integration testing for the `lang` command's configuration resolution logic. While `module` and `export` commands had `_with_config` integration tests, `resolve_lang_with_config` was missing. As configuration precedence (CLI vs TOML vs JSON Profile) is a critical interface behavior, locking it down with an explicit test improves confidence and prevents edge-case drift.

## 🔎 Evidence
Missing integration test for `resolve_lang_with_config` in `crates/tokmd/tests/config_resolution.rs`.
Running `cargo test -p tokmd --test config_resolution` confirms `test_resolve_lang_with_config` now successfully executes and validates CLI vs TOML `ViewProfile` precedence.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add `test_resolve_lang_with_config` integration test.
- why it fits this repo and shard: Directly addresses the priority to add missing BDD/integration coverage for critical paths in the `interfaces` shard (covering config and CLI).
- trade-offs: Structure is solid, velocity is fast, and governance impact is positive by locking down determinism.

### Option B
- what it is: Add a unit test for profile resolution map lookups.
- when to choose it instead: If profile mapping had complex branching.
- trade-offs: `resolve_profile` map lookups are trivial compared to the deep merging performed by `resolve_lang_with_config`.

## ✅ Decision
Chosen Option A. Reconciling `CliLangArgs`, `ViewProfile`, and defaults via `resolve_lang_with_config` is an essential logic path that deserved explicit integration coverage.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/config_resolution.rs`: Added `test_resolve_lang_with_config` test.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test config_resolution
test test_resolve_lang_with_config ... ok
13 passed; 0 failed
```

## 🧭 Telemetry
- Change shape: Proof-improvement patch
- Blast radius: `tests/` boundary only
- Risk class + why: Lowest risk. Test suite addition only. No runtime logic change.
- Rollback: Revert the test commit.
- Gates run: `cargo test -p tokmd --test config_resolution`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces/envelope.json`
- `.jules/runs/specsmith_interfaces/decision.md`
- `.jules/runs/specsmith_interfaces/receipts.jsonl`
- `.jules/runs/specsmith_interfaces/result.json`
- `.jules/runs/specsmith_interfaces/pr_body.md`

## 🔜 Follow-ups
None.
