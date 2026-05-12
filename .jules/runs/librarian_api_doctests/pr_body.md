## 💡 Summary
This is a learning PR. I investigated adding executable doctests to the config resolver interfaces (`crates/tokmd/src/config/resolve/`) but found they already have comprehensive, passing doctest coverage.

## 🎯 Why
The goal was to improve factual docs quality and executable examples for public interfaces to prevent silent drift.

## 🔎 Evidence
- `crates/tokmd/src/config/resolve/export.rs`
- `crates/tokmd/src/config/resolve/module.rs`
- `crates/tokmd/src/config/resolve/lang.rs`
- The `resolve_*` and `resolve_*_with_config` functions all have `/// # Examples` sections containing executable ````rust` doctests.

## 🧭 Options considered
### Option A (recommended)
- Add doctests to config resolvers (`resolve_lang`, `resolve_export`, etc.).
- This fits the shard by directly targeting the core CLI configuration interfaces.
- Trade-offs: Structure is improved by ensuring public APIs have executable examples. Velocity is unaffected. Governance is improved by preventing silent drift.

### Option B
- Add doctests to `tokmd_core::workflows`.
- Choose this if the config layer is already fully covered.
- Trade-offs: Might duplicate existing integration tests, focusing less on public APIs than the configuration layer.

## ✅ Decision
Option A was chosen to investigate the config resolvers. Upon investigation, the config resolvers already have comprehensive doctest coverage. Therefore, this is submitted as a learning PR instead of forcing a fake fix.

## 🧱 Changes made (SRP)
- None. This is a learning PR.

## 🧪 Verification receipts
```text
cargo xtask docs --check
cargo test --doc
```

## 🧭 Telemetry
- Change shape: learning PR
- Blast radius: none
- Risk class: none (learning PR)
- Rollback: none
- Gates run: docs-executable (cargo xtask docs --check, cargo test --doc)

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`
- `.jules/friction/open/config_resolver_doctests.md`

## 🔜 Follow-ups
None.
