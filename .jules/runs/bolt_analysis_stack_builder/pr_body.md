## 💡 Summary
Removed unnecessary string allocations in `tokmd-analysis` hot paths, avoiding repeated `.clone()` usage for `String` keys during commit aggregations and topic map generation.

## 🎯 Why
During analysis runs on large repositories, file paths and keywords are aggregated extensively. In `git::mod`, commit files triggered repeated `.clone()` operations into `BTreeMap<String, usize>`. Similarly, topic computation involved redundant allocations when populating overall term frequencies.

## 🔎 Evidence
- `crates/tokmd-analysis/src/git/mod.rs` showed `.entry(key.clone()).or_insert(0) += 1` inside an inner loop over all files for all commits.
- Profiling structural footprint shows map keys can safely borrow from the parent `row_map` strings, dropping thousands of transient `String` clones per scan.

## 🧭 Options considered
### Option A (recommended)
- what it is: Convert `BTreeMap<String, T>` structures in the git aggregation logic to use `&str` keys, and optimize the `topics::mod` frequency build step to share the single structural loop.
- why it fits this repo and shard: Analysis loops must be as deterministic and tight as possible, as required by the `perf-proof` profile expectation of explicit structural proof.
- trade-offs: Structure / Velocity / Governance: Needs slightly stricter lifetime plumbing (tying maps to the outer iteration scope strings), but ensures deterministic performance.

### Option B
- what it is: Throwing async/parallel chunks at the problem instead.
- when to choose it instead: If the problem was CPU-bound rather than alloc-bound, and only if determinism can be strictly preserved via sorting post-chunking.
- trade-offs: Increases orchestration complexity and debugging surface.

## ✅ Decision
Option A. Explicit structural reduction is the safest and most provable path for a hot loop without jeopardizing output determinism.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/git/mod.rs` - Switched `commit_counts` and `last_change` maps to borrow `&str` keys from `row_map`.
- `crates/tokmd-analysis/src/git/freshness.rs` - Aligned freshness and age distribution signatures to accept borrowed string keys.
- `crates/tokmd-analysis/src/topics/mod.rs` - Moved `overall_tf` computation into the main file iteration pass, avoiding repeated String map traversals and cloning.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis --all-features
cargo build --release -p tokmd-analysis
cargo fmt -- --check
cargo clippy -p tokmd-analysis -- -D warnings
```

## 🧭 Telemetry
- Change shape: Optimization
- Blast radius: `tokmd-analysis` internal struct types and logic
- Risk class + why: Low, pure performance structural fix with full unit/determinism test suite backing
- Rollback: Revertible natively via git
- Gates run: `perf-proof`, `clippy`, `fmt`, `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`

## 🔜 Follow-ups
None at this time.
