## 💡 Summary
Replaced expensive full-vector cloning in `build_top_offenders` with iteration and sorting over references. This prevents cloning the entire array of `FileStatRow` items multiple times, cloning only the final `TOP_N` elements required for the result.

## 🎯 Why
The analysis module previously cloned the full `FileStatRow` vector multiple times per execution to find top offenders (by lines, tokens, bytes, least documented, and density). These structs contain string fields, meaning deep clones were occurring across thousands of rows.

## 🔎 Evidence
- **File**: `crates/tokmd-analysis/src/derived/mod.rs`
- **Benchmark Receipt**:
```text
top_offenders_original  time:   [5.4343 ms 5.4486 ms 5.4631 ms]
top_offenders_optimized time:   [945.30 µs 952.54 µs 960.73 µs]
```

## 🧭 Options considered
### Option A (recommended)
- Replace `.to_vec()` and `.cloned()` iteration on the full result set with `.iter().collect::<Vec<&FileStatRow>>()`
- Why it fits: Specifically removes unnecessary string and object cloning on an intermediate buffer while preserving full test determinism.
- Trade-offs: Structure / Velocity / Governance. Minimal structural change. Velocity is improved. No governance impact.

### Option B
- Write a custom Top N collection routine that avoids sorting altogether.
- When to choose: If maintaining vectors of references was too slow.
- Trade-offs: Too complex, requires a custom binary heap approach which is less readable than simple sort functions.

## ✅ Decision
Option A was chosen as it achieved a ~5x performance speedup on the microbenchmark and perfectly matched the persona goals with minimal structural divergence.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/mod.rs`

## 🧪 Verification receipts
```text
test derived::tests::properties::totals_equal_sum_of_rows ... ok
test derived::tests::properties::top_offenders_bounded_by_ten ... ok

test result: ok. 427 passed; 0 failed; 0 ignored; 0 measured; 1101 filtered out; finished in 0.88s
```

## 🧭 Telemetry
- Change shape: Optimization
- Blast radius: API/IO/docs compatibility untouched. Determinism maintained.
- Risk class: Low
- Rollback: Revert single commit.
- Gates run: `cargo check --workspace`, `cargo test -p tokmd-analysis --release`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`

## 🔜 Follow-ups
None.
