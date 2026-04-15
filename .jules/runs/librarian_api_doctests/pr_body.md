## 💡 Summary
Added missing executable doctests to public config resolution functions in `crates/tokmd/src/config.rs`. This improves the API documentation quality by ensuring examples compile and accurately reflect the fallback behavior of profiles.

## 🎯 Why
The core Tier 5 config API lacked executable doctests for key resolution functions (`get_toml_view`, `get_json_profile`, `load_config`, `get_profile_name`, `resolve_profile`), increasing the risk of silent docs drift. By introducing executable doctests, we lock in expected behaviors and comply with the `docs-executable` gate profile.

## 🔎 Evidence
- `crates/tokmd/src/config.rs` lacked `/// # Examples` sections on standard CLI config getters.
- `cargo test --doc -p tokmd` showed missing coverage for these specific functions prior to changes.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add missing executable doctests to public CLI config resolution functions in `crates/tokmd/src/config.rs`.
- why it fits this repo and shard: Directly addresses missing example coverage for public interfaces within the `interfaces` shard.
- trade-offs: Structure is improved through tighter guarantees, Velocity is slightly reduced by added test maintenance, Governance is strengthened via deterministic documentation.

### Option B
- what it is: Add missing executable doctests to public FFI boundary functions in `crates/tokmd-core/src/ffi.rs`.
- when to choose it instead: If the FFI boundary is more heavily utilized or lacks basic assertions compared to the CLI resolution logic.
- trade-offs: Provides coverage on raw FFI but might require more complex mocking or setup for internal states compared to pure config functions.

## ✅ Decision
Chosen Option A. `crates/tokmd/src/config.rs` is a primary interface for config merging and parsing. Covering these APIs ensures accurate fallback logic documentation and adheres to the memory guidance to use full imports like `use tokmd::config::resolve_profile;`.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/config.rs`: Added executable doctests for `get_toml_view`, `get_json_profile`, `load_config`, `get_profile_name`, and `resolve_profile`.

## 🧪 Verification receipts
```text
$ cargo test --doc -p tokmd
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

$ cargo test -p tokmd
test result: ok. ...

$ cargo xtask docs --check
$ cargo fmt -- --check
$ cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Docs/tests addition.
- Blast radius: `docs` only (doctests).
- Risk class: Low, test-only addition.
- Rollback: Safe, revert commits.
- Gates run: `cargo test --doc`, `cargo test`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
