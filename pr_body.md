## 💡 Summary
Added integration tests to `crates/tokmd/tests/docs.rs` to verify that the `tokmd context`, `tokmd run`, and `tokmd diff` commands work as documented in `docs/tutorial.md`.

## 🎯 Why (perf bottleneck)
Not applicable (docs/test improvement). Unverified documentation leads to silent drift between the README/tutorial examples and actual API behavior, causing friction for new users.

## 📊 Proof (before/after)
Unmeasured. This change adds tests rather than optimizing performance, ensuring documentation accuracy over time.

## 🧭 Options considered
### Option A (recommended)
- Add deterministic `assert_cmd` integration tests for the tutorial recipes to `crates/tokmd/tests/docs.rs`.
- Why it fits this repo: "Docs as tests" is already established in `docs.rs`. This prevents silent drift.
- Trade-offs: Minor increase in test suite execution time.

### Option B
- Manually test the commands and update documentation if necessary.
- When to choose it instead: For one-off scripts not intended for long-term support.
- Trade-offs: Provides no automated regression protection; docs will inevitably drift again.

## ✅ Decision
Option A was chosen to ensure the tutorial documentation remains verifiable and cannot silently drift from actual CLI behavior.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/docs.rs`: Added `recipe_context_bundle` and `recipe_run_and_diff` test functions.

## 🧪 Verification receipts
```
cargo test -p tokmd --test docs
cargo fmt
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: New tests added
- Blast radius: Zero (test-only change)
- Risk class: Low
- Rollback: Revert PR
- Merge-confidence gates: `test`, `fmt`, `clippy`

## 🗂️ .jules updates
- Appended run entry to `.jules/docs/ledger.json`.
- Saved receipt envelope in `.jules/docs/envelopes/`.
- Documented run decisions in `.jules/docs/runs/`.

## 📝 Notes (freeform)
N/A

## 🔜 Follow-ups
N/A
