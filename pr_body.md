# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Removed all panicking assertions (`unwrap`, `unwrap_err`) from tests in `tokmd-ffi-envelope` (specifically `deep_w69.rs` and `schema_compliance_w53.rs`) in alignment with the full-quality stance goal.

## 🎯 Why / Threat model
Tests using `unwrap()` or `unwrap_err()` lead to ungraceful test thread panics rather than cleanly formatted test failures when assertions aren't met. Moving to proper error propagation via `Result` allows for better diagnostics and aligns the codebase with strict security/quality hygiene.

## 🔎 Finding (evidence)
Observed multiple panics when reviewing:
- `crates/tokmd-ffi-envelope/tests/schema_compliance_w53.rs`
- `crates/tokmd-ffi-envelope/tests/deep_w69.rs`

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Refactor test signatures to return `Result<(), Box<dyn std::error::Error>>`, replacing `.unwrap()` with `?` and `.unwrap_err()` with explicit match blocks.
- **Why it fits this repo:** Moves test failure handling from thread panics to structured `Result`-based error propagation, aligning with Rust best practices and the Sentinel full quality stance.
- **Trade-offs:** Slightly longer test signatures. More robust failures.

### Option B
- **What it is:** Change `unwrap()` calls to `expect("clear error message")`.
- **When to choose it instead:** When the type does not easily convert to a standard `Error`, or changing the test signature is problematic.
- **Trade-offs:** Better error messages than unwrap, but still panics.

## ✅ Decision
**Option A**. Replacing `unwrap` with `?` is the preferred way to burn down panics and enforce rigorous test hygiene.

## 🧱 Changes made (SRP)
- `crates/tokmd-ffi-envelope/tests/schema_compliance_w53.rs`: Changed test signatures to `Result`, replaced `unwrap`/`unwrap_err`.
- `crates/tokmd-ffi-envelope/tests/deep_w69.rs`: Changed test signatures to `Result`, replaced `unwrap`/`unwrap_err`, and correctly handled `proptest!` assertions without panicking.

## 🧪 Verification receipts
```json
{
  "run_id": "7d38742b-eaf2-40e8-a354-555e60692bca",
  "timestamp_utc": "2026-03-19T12:19:19Z",
  "lane": "scout",
  "target": "crates/tokmd-ffi-envelope/tests",
  "commands": [],
  "results": [
    {
      "cmd": "cargo build --verbose",
      "status": "PASS"
    },
    {
      "cmd": "cargo test -p tokmd-ffi-envelope --verbose",
      "status": "PASS"
    },
    {
      "cmd": "cargo clippy -p tokmd-ffi-envelope -- -D warnings",
      "status": "PASS"
    }
  ]
}
```

## 🧭 Telemetry
- Change shape: Moderate refactoring of test suite signatures.
- Blast radius: Highly contained (only touches test files).
- Risk class: Low risk. Does not modify application logic.
- Rollback: Revert the PR.
- Merge-confidence gates: `cargo build`, `cargo test -p tokmd-ffi-envelope`, `cargo clippy -p tokmd-ffi-envelope`

## 🗂️ .jules updates
- Created run envelope `.jules/security/envelopes/7d38742b-eaf2-40e8-a354-555e60692bca.json`
- Created run log `.jules/security/runs/2026-03-19.md`
- Appended run outcome to `.jules/security/ledger.json`
- Created `.jules/policy/scheduled_tasks.json` and `.jules/runbooks/PR_GLASS_COCKPIT.md` as they were missing.

## 📝 Notes (freeform)
Refactoring `proptest!` blocks required using `prop_assert!` instead of `?` to avoid signature mismatch issues inside the macro expansion.

## 🔜 Follow-ups
None.
