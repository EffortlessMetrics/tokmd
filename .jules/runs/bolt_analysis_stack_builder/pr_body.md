## 💡 Summary
Optimizes term aggregation in `tokmd-analysis-topics` by replacing the `.entry(term.clone()).or_insert(0)` pattern with `.get_mut(term.as_str())` and a fallback `.insert()`. This avoids an unconditional string allocation for every token processed across all files.

## 🎯 Why
The project's memory specifically calls out the `.entry(key.clone()).or_insert(...)` pattern as an anti-pattern for performance hot loops with `BTreeMap<String, ...>`, because it forces a `String` clone even if the key is already present. In `build_topic_clouds`, aggregating tokens across all modules into the `overall_tf` map is a dense hot path, so removing the unconditional string clone per word parsed provides a nice performance win.

## 🔎 Evidence
- File path: `crates/tokmd-analysis-topics/src/lib.rs`
- Finding: The aggregation loop iterated over `tf_map` and invoked `*overall_tf.entry(term.clone()).or_insert(0) += *tf;`
- Receipt:
  ```text
  Pattern 1 (entry clone): 8.229688ms
  Pattern 2 (get_mut): 5.182316ms
  ```

## 🧭 Options considered
### Option A (recommended)
- what it is: Replace `.entry(term.clone()).or_insert(0)` with a `.get_mut()` check + `.insert(term.clone())`.
- why it fits this repo and shard: Directly targets memory-documented performance anti-patterns in the `analysis-stack` shard (`tokmd-analysis-topics`) and is proven by standard benchmarking.
- trade-offs: Structure is slightly longer, but velocity and execution efficiency are better.

### Option B
- what it is: Address `.clone()` calls on larger `FileStatRow` values in `tokmd-analysis-derived`.
- when to choose it instead: If profiling indicates struct clones dwarf hot-loop string tokenizations.
- trade-offs: Struct cloning is larger memory overhead but happens far less frequently than per-token parsing allocations.

## ✅ Decision
Option A. It explicitly resolves a well-known friction item documented in the memory context, targets string allocation in a hot loop perfectly fitting the `bolt` persona, and successfully drops execution time of the mapping logic by ~30% in isolation tests.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-topics/src/lib.rs`: Removed `.entry(term.clone()).or_insert(0)` and implemented equivalent allocation-free-if-present `get_mut()` logic.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis-topics
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
...
```

## 🧭 Telemetry
- Change shape: Implementation optimization
- Blast radius: Internal to `tokmd-analysis-topics`. Output determinism guaranteed to match exactly.
- Risk class: Low
- Rollback: Revert to `.entry().or_insert()`
- Gates run: `cargo test -p tokmd-analysis-topics`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`

## 🔜 Follow-ups
None.
