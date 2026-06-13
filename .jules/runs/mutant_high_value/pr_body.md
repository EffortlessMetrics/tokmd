## 💡 Summary
This change adds property-based testing to `compute_diff_totals` to mathematically guarantee the deterministic summation of diff rows.

## 🎯 Why
The diff calculations sit at the edge of the core formatting pipeline. While some static smoke tests existed, the mathematical aggregation over arbitrary rows lacked structured invariant checks (`new - old == delta`, map/sum matching `fold`). Adding Proptest coverage here strengthens confidence that `tokmd diff` emits correct metrics under randomized inputs.

## 🔎 Evidence
- `crates/tokmd-format/src/diff/compute.rs`
- Observed gap: No randomized property tests for the `DiffTotals` reduction function.

## 🧭 Options considered
### Option A (recommended)
- Add property tests for `compute_diff_totals` in `crates/tokmd-format/src/diff/compute.rs`.
- Why it fits: Aligns directly with the "Mutant" objective to reduce uncertainty around contract surfaces by testing structural math.
- Trade-offs: Minor code footprint; negligible test-time cost; high confidence in core accumulation.

### Option B
- Add serialization stability tests for json boundary DTOs.
- When to choose: Better if the core structural issue was backwards-incompatible breaks in the manifest definitions rather than math accumulation logic.
- Trade-offs: Testing DTO fields is less mathematically rigorous than fuzzing an accumulator.

## ✅ Decision
Option A was chosen. Enhancing mathematical validation of the diff totals directly improves the proof guarantees around `tokmd diff`'s accuracy.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/diff/compute.rs`:
  - Added property tests for invariant maintenance (`new - old = delta`).
  - Added property tests confirming `fold` accumulator behavior exactly matches simple mapping sums.
  - Added deterministic zero check for empty sequences.

## 🧪 Verification receipts
```text
running 3 tests
test diff::compute::tests::diff_totals_empty_is_zero ... ok
test diff::compute::tests::diff_totals_maintains_delta_invariants ... ok
test diff::compute::tests::diff_totals_preserves_row_sums ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 144 filtered out; finished in 0.02s
```

## 🧭 Telemetry
- Change shape: Test-only additions
- Blast radius: None to production logic. Tests constrained to `tokmd-format`.
- Risk class: Low
- Rollback: Revert the test block.
- Gates run: `cargo test`, `cargo fmt -- --check`, `cargo clippy`, `cargo build --verbose`.

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None
