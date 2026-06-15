## 💡 Summary
Added structural invariant properties tests to the COCOMO estimation model. This guarantees models monotonically grow with source size changes, preventing negative scaling.

## 🎯 Why
The previous core models `cocomo81_effort_pm` and `cocomo2_effort_pm` lacked direct verification that increasing size reliably increases estimated effort and duration. If parameter changes occur, we must ensure these logical constraints (monotonicity) hold, as they define real bounding invariants.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-analysis/src/effort/tests/proptest_models.rs`
- Observed behavior: `cocomo2_effort_pm` and `cocomo81_effort_pm` scale correctly, but weren't tested for monotonicity over ranges.
- Tested by running `cargo test -p tokmd-analysis`.

## 🧭 Options considered
### Option A (recommended)
- Add strict proptest guarantees that evaluate two sizes and guarantee `e2 > e1` and `s2 > s1`.
- Why it fits: Matches 'Invariant' persona to focus on bounding guarantees using property testing in the analysis stack.
- Trade-offs: Structure / Velocity / Governance - Very minimal complexity, immediate safety net.

### Option B
- Improve `uncertainty.rs` Bound Checking instead.
- When to choose: When uncertainty limits are known to drift under scaling conditions.
- Trade-offs: Testing derived behavior rather than base model assumptions.

## ✅ Decision
Decided on Option A to lock down the mathematical foundation of the estimator itself.

## 🧱 Changes made (SRP)
- Modified `crates/tokmd-analysis/src/effort/tests/proptest_models.rs` to add monotonic growth properties.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Added tests.
- Blast radius: `tests` only, no runtime risk.
- Risk class: Safe / Local.
- Rollback: Revert the test block.
- Gates run: `property` fallbacks.

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
