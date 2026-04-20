## 💡 Summary
Fix broken doctests in `crates/tokmd/src/config.rs` to accurately reflect the correct module import paths. This ensures the examples compile cleanly and serve as executable documentation.

## 🎯 Why
The doctests for the `config` module in `crates/tokmd/src/config.rs` incorrectly attempted to import and use functions from the crate root (`tokmd::resolve_config`), which were not accessible that way in the doctest environment. This caused `cargo test -p tokmd --doc` to fail, resulting in misleading documentation and breaking the executable docs contract.

## 🔎 Evidence
- **File path:** `crates/tokmd/src/config.rs`
- **Observed behavior:** `cargo test -p tokmd --doc` reported errors like `error[E0432]: unresolved import tokmd::config::config::load_config` and `could not find 'config' in 'config'` due to incorrect path referencing in the doctests.
- **Receipt:** Running `cargo test -p tokmd --doc` now passes completely.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Fix the doctest imports to correctly use `tokmd::config::<function>` instead of `tokmd::<function>`.
- **Why it fits this repo and shard:** Adheres to the memory guidelines for writing doctests in `tokmd/src/config.rs` by referencing items via their full path (`use tokmd::config::...`) since they are not re-exported at the crate root for doctest usage.
- **Trade-offs:** Fast, zero-risk fix. Directly adheres to the `docs-executable` gate profile.

### Option B
- **What it is:** Re-export the items in `crates/tokmd/src/lib.rs` to make `use tokmd::` work for doctests.
- **When to choose it instead:** If changing the public API surface of the crate is desired.
- **Trade-offs:** High risk of unintended side effects on the public API surface. Against explicit memory guidance.

## ✅ Decision
Option A. It's safe, corrects the examples without modifying the crate's external API, and directly resolves the compilation errors following the repo's established pattern.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/config.rs`: Updated all doctest import paths and function calls to explicitly use the `tokmd::config` namespace.

## 🧪 Verification receipts
```text
cargo test -p tokmd --doc
running 9 tests
test crates/tokmd/src/config.rs - config::ConfigContext (line 10) ... ok
test crates/tokmd/src/config.rs - config::resolve_config (line 242) ... ok
test crates/tokmd/src/config.rs - config::resolve_export (line 522) ... ok
...
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: Documentation update (rustdocs)
- Blast radius: Docs only
- Risk class: Low
- Rollback: Revert the commit.
- Gates run: `cargo test -p tokmd --doc`, `cargo clippy -p tokmd -- -D warnings`, `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
