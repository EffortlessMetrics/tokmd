## 💡 Summary
Strengthened the test suite by fixing missing metrics accumulation in `aggregate.rs`. This correctly propagates `bytes` and `tokens` up to language reports for child files across both `Collapse` and `Separate` child modes.

## 🎯 Why
`cargo mutants` identified that modifying aggregation logic for `bytes` and `tokens` within `create_lang_report_from_rows` produced identical outcomes, meaning tests were failing to assert complete shape constraints for `FileKind::Child` elements. Without this check, logic around secondary metadata on child elements could quietly regress during future updates.

## 🔎 Evidence
- `crates/tokmd-model/src/aggregate.rs` missed two mutants.
- Missing `entry.0.bytes += row.bytes` and `entry.0.tokens += row.tokens` in `Child` file matches for both child modes.

```text
crates/tokmd-model/src/aggregate.rs:70:34: replace += with -= in create_lang_report_from_rows
crates/tokmd-model/src/aggregate.rs:70:34: replace += with *= in create_lang_report_from_rows
```

## 🧭 Options considered

### Option A (recommended)
- Add missing metric aggregation to production logic and write targeted deterministic unit tests to ensure shape regressions are caught instantly going forward.
- Fits the model logic surface cleanly by aligning `Child` accumulation parity with `Parent` accumulation.
- Structure/Velocity/Governance tradeoff: Low friction. Adds simple unit tests without slowing down execution time.

### Option B
- Modify only the property tests to check all stats.
- Does not explicitly map individual structural expectations to child logic branching. Can lead to brittle property tests.
- High tradeoff of obscuring structural expectations behind property-based blackboxes.

## ✅ Decision
Option A was chosen. Adding targeted unit tests alongside the fix prevents mutation-style silent regressions around child path aggregation tracking inside model generation constraints.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/aggregate.rs`: Added `+=` increment statements for bytes and tokens handling in `ChildrenMode::Collapse` and `ChildrenMode::Separate`.
- `crates/tokmd-model/tests/aggregate_test.rs`: Added granular structural assertions verifying bytes and tokens are propagated during `create_lang_report_from_rows` calls.

## 🧪 Verification receipts
```text
cargo test -p tokmd-model
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
```

## 🧭 Telemetry
- Change shape: Implementation fix + structural testing patch
- Blast radius: `tokmd-model` (Testing / Metrics aggregation)
- Risk class: Low - fixes missing structural aggregations.
- Rollback: Safe to revert without breaking storage constraints.
- Gates run: `cargo mutants`, `cargo test -p tokmd-model`

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None.
