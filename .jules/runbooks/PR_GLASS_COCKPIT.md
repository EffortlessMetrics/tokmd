---
# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Replaces generic `.expect("should exist")` assertions with specific, context-aware error messages in `context_pack.rs` and `export_bundle.rs` to improve developer experience.

## 🎯 Why (user/dev pain)
Generic `expect("should exist")` strings provide no actionable context when tests fail or assertions trigger, making it harder for developers to debug why an invariant was violated. Specific messages clarify *what* went wrong.

## 🔎 Evidence (before/after)
File paths affected:
- `crates/tokmd/src/context_pack.rs`
- `crates/tokmd/src/export_bundle.rs`
Before: `.expect("should exist")`
After: e.g., `.expect("Valid 128k budget should parse correctly")`

## 🧭 Options considered
### Option A (recommended)
- Update `.expect` calls with descriptive string literals.
- Why it fits: Aligns with project guidelines to avoid generic expect strings.
- Trade-offs: Simple and minimally invasive, preserves test structure.

### Option B
- Refactor test functions to return `anyhow::Result<()>` and use the `?` operator.
- When to choose: Useful for cascading errors.
- Trade-offs: Changes test signatures, can swallow exact line numbers in some runners.

## ✅ Decision
Option A was chosen as it strictly adheres to the provided repo rules about descriptive panic messages without altering test function signatures.

## 🧱 Changes made (SRP)
- Refactored `expect` messages in `crates/tokmd/src/context_pack.rs`.
- Refactored `expect` messages in `crates/tokmd/src/export_bundle.rs`.

## 🧪 Verification receipts
- `cargo test -p tokmd --no-default-features`: PASS
- `cargo clippy -p tokmd -- -D warnings`: PASS
- `cargo xtask gate`: PASS

## 🧭 Telemetry
- Blast radius: localized to tests and unwrapping logic in two files.
- Risk class: Low (Error message strings only).
- Rollback: Safe to revert.
- Merge-confidence gates: `cargo xtask gate --check`

## 🗂️ .jules updates
Added run envelope and log for `2026-03-29`. Updated `.jules/palette/ledger.json`.

## 📝 Notes (freeform)
N/A

## 🔜 Follow-ups
N/A
---
