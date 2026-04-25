# Decision

## Option A: Replace `.entry(term.clone()).or_insert(0)` with `.get_mut()` / `.insert()` in `tokmd-analysis-topics`
- What it is: Replaces `*overall_tf.entry(term.clone()).or_insert(0) += *tf;` with `if let Some(count) = overall_tf.get_mut(term.as_str()) { *count += *tf; } else { overall_tf.insert(term.clone(), *tf); }` in `crates/tokmd-analysis-topics/src/lib.rs`.
- Why it fits this repo and shard: Memory explicit constraint: "To avoid unnecessary `String` allocations in hot loops when looking up or inserting into a `BTreeMap` with `String` keys, avoid the `map.entry(key.clone()).or_insert(...)` pattern." This is a known performance hot path since `overall_tf` aggregates counts for terms across all modules, and `term` string allocation inside `.entry()` happens unconditionally per term even if the term is already present in `overall_tf`.
- Trade-offs: Simple, targeted, completely aligns with project guidelines. Velocity is fast. Structure gets slightly longer but more optimal. Governance aligns with `perf-proof` using the structural logic.

## Option B: Optimize `.clone()` of `FileStatRow` in `tokmd-analysis-derived`
- What it is: Avoid the `.clone()` in `by_module.get_mut(&row.module) ... *existing = row.clone();`
- When to choose: When memory cloning struct instances is proven to be the primary bottleneck in analysis derived processing.
- Trade-offs: A bit riskier because `FileStatRow` is slightly complex and lifetimes might leak if not careful.

## Decision
**Option A**. It's an explicitly mentioned friction point in the context/memory, perfectly fits "unnecessary allocations / cloning / string building", is located in the `analysis-stack` shard (`tokmd-analysis-topics`), and guarantees reduction in allocation overhead in a hot loop that processes many tokens.
