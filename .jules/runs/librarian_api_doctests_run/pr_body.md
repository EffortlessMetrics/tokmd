## 💡 Summary
Added executable doctests to `tokmd::config::get_profile_name` and `tokmd::config::resolve_profile`. This improves the API documentation by providing compile-time verified usage examples for resolving configuration profiles, ensuring they do not silently drift from the actual code behavior.

## 🎯 Why
The `Librarian` persona prioritizes executable documentation and examples over static prose to prevent documentation drift. While `config.rs` has several good doctests, `get_profile_name` and `resolve_profile` are public API surfaces that lacked any executable examples. Adding them provides concrete, verified examples of how legacy JSON configs and CLI profile name resolutions work.

## 🔎 Evidence
- **Target:** `crates/tokmd/src/config.rs`
- **Finding:** Missing `/// # Example` executable doctests for `get_profile_name` and `resolve_profile`
- **Verification:** Ran `cargo test --doc -p tokmd -- config` and `cargo test -p tokmd` to prove the new examples compile and execute successfully.

## 🧭 Options considered

### Option A (recommended)
- Add executable doctests to `get_profile_name` and `resolve_profile`.
- **Why it fits:** Aligns directly with the `Librarian` mandate to improve executable coverage for core/config/CLI public APIs.
- **Trade-offs:** High value for documentation drift prevention; low structural/velocity cost.

### Option B
- Document internal private configuration functions.
- **When to choose:** When internal contributor documentation is the main bottleneck.
- **Trade-offs:** Low leverage for end-user API consumers; not valuable for public crate docs.

## ✅ Decision
Option A. Public APIs should have executable examples that are verified by the compiler. This adheres to the `Librarian` directive to prefer executable documentation over prose rewrites and targets the core/config interfaces shard directly.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/config.rs`: Added `/// # Example` rust block to `get_profile_name`.
- `crates/tokmd/src/config.rs`: Added `/// # Example` rust block to `resolve_profile`.

## 🧪 Verification receipts
```text
$ cargo test --doc -p tokmd -- config
running 11 tests
...
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

$ cargo clippy -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
```

## 🧭 Telemetry
- **Change shape:** Doc comment additions.
- **Blast radius:** Docs-only; no production runtime behavior changes.
- **Risk class:** Very low; safe rollback by reverting the doc comments.
- **Rollback:** git revert.
- **Gates run:** `cargo test --doc -p tokmd`, `cargo test -p tokmd`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests_run/envelope.json`
- `.jules/runs/librarian_api_doctests_run/decision.md`
- `.jules/runs/librarian_api_doctests_run/receipts.jsonl`
- `.jules/runs/librarian_api_doctests_run/result.json`
- `.jules/runs/librarian_api_doctests_run/pr_body.md`

## 🔜 Follow-ups
None immediately.
