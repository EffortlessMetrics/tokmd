## 💡 Summary
Optimized top offenders calculation by leveraging `select_nth_unstable_by`. Instead of completely sorting the data vectors (`O(N log N)`) for up to five dimensions, we now partition the required top items in `O(N)` time and only sort those top 10 elements.

## 🎯 Why
Finding the top 10 files by lines, tokens, bytes, density, and least documented files triggers full array clones and full mergesorts across all export elements (which can be over 100,000 files in large repositories). This hot-path creates unnecessary allocations and CPU utilization for elements that will never make it to the top 10. We can achieve the exact same deterministic top 10 results by using the `select_nth_unstable_by` standard library function.

## 🔎 Evidence
File path: `crates/tokmd-analysis/src/derived/files.rs`

Performance baseline using a synthetic repo of 100,000 files:
```text
Orig sort: 803.159281ms
select_nth_unstable: 138.042331ms
```
By switching to partial unstable sorting, the algorithm computes the top offenders up to ~5.8x faster.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Use `select_nth_unstable_by` inside `build_top_offenders` to isolate the `TOP_N` elements before sorting.
- **Why it fits this repo and shard**: Performance work inside analysis derived fields is a direct priority.
- **Trade-offs**:
  - *Structure*: The interface remains stable. Same deterministic sorting applied to the sliced array.
  - *Velocity*: Immense scaling improvement for massive repositories without architectural changes.
  - *Governance*: Standard library algorithm usage, virtually zero risk.

### Option B
- **What it is**: Optimize topic tokenization using splitting over array iterations.
- **When to choose it instead**: If the `tokenize_path` string operations overshadow derived iteration.
- **Trade-offs**: Tokenization optimization only yields marginal throughput variance versus standardizing the Big-O optimization of analysis sorting.

## ✅ Decision
Implemented Option A. It's a clean, standard-library-backed algorithmic optimization that massively accelerates derived report generation for large repositories by changing `O(N log N)` to `O(N)`.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/files.rs`: Replaced full `sort_by` calls in `build_top_offenders` with a helper closure leveraging `select_nth_unstable_by`.

## 🧪 Verification receipts
```text
cargo build --verbose
bash -c 'CI=true cargo test -p tokmd-analysis --verbose'
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- **Change shape**: Feature improvement
- **Blast radius**: Minimal (only `tokmd-analysis` top offenders derived reporting). Schema and formatting unaffected.
- **Risk class + why**: Low. Standard sorting logic substitution.
- **Rollback**: Standard git revert.
- **Gates run**: `perf-proof` (benchmarks + structural optimization), core-rust gates.

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`

## 🔜 Follow-ups
None.
