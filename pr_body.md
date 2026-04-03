## 💡 Summary
Replaced generic `.unwrap()` calls with context-specific `.expect()` messages in `crates/tokmd/tests/json_output.rs` and `crates/tokmd/tests/regression_suite_w52.rs`.

## 🎯 Why (user/dev pain)
During test failures, generic `.unwrap()` calls cause panic messages with no context, forcing developers to look up the exact line of code. Clear `.expect()` messages provide immediate diagnostic context.

## 🔎 Evidence (before/after)
- `crates/tokmd/tests/json_output.rs`
- `crates/tokmd/tests/regression_suite_w52.rs`
- Before: `let rows = json["rows"].as_array().unwrap();`
- After: `let rows = json["rows"].as_array().expect("output JSON should contain a 'rows' array");`

## 🧭 Options considered
### Option A (recommended)
- What it is: Context-specific `.expect()` replacements in test files.
- Why it fits this repo: Aligns with `.jules/palette/` DX focus on diagnostic improvements and memory guidelines explicitly stating to use specific context for `.unwrap()` replacements.
- Trade-offs: Minor increase in verbosity for significant test DX improvement.

### Option B
- What it is: Ignore `.unwrap()` in tests and look for CLI error messages.
- When to choose it instead: If the test suite already has excellent DX.
- Trade-offs: Leaves low-hanging fruit in the test suite unfixed.

## ✅ Decision
Option A. It's an easy-to-review SRP win that directly improves test suite diagnostics.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/json_output.rs`: Replaced 3 `.unwrap()` calls with `.expect()`.
- `crates/tokmd/tests/regression_suite_w52.rs`: Replaced 17 `.unwrap()` calls with `.expect()`.

## 🧪 Verification receipts
- `cargo fmt -- --check`: PASS
- `cargo clippy -p tokmd -- -D warnings`: PASS
- `cargo test -p tokmd`: PASS

## 🧭 Telemetry
- Change shape: Refactor
- Blast radius: Only test files. No production code changes.
- Risk class: Low.
- Rollback: `git revert`
- Merge-confidence gates: fmt, clippy, test.

## 🗂️ .jules updates
- Appended run envelope to `.jules/palette/ledger.json`.
- Logged run in `.jules/palette/runs/`.
- Created note in `.jules/palette/notes/` about `.expect()` vs `.unwrap()` pattern.

## 📝 Notes (freeform)
N/A

## 🔜 Follow-ups
None.
