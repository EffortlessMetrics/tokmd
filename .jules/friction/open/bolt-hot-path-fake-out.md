---
id: bolt-hot-path-fake-out
persona: Bolt ⚡
style: Builder
shard: analysis-stack
status: open
---
# Issue
The map aggregation logic in `crates/tokmd-analysis/src/derived/mod.rs` (e.g. `build_max_file_report`, `build_lang_purity_report`) uses `BTreeMap<String, ...>` and clones strings (`row.lang.clone()`). At first glance, this appears to be a prime hot-path optimization target to eliminate allocations.

However, the cloning only happens within the `else` block of `if let Some(existing) = map.get_mut(...)`. This means string allocation only occurs once per unique language or module, while the hot lookups safely avoid allocation.

# Conclusion
Replacing owned strings with string slices (`&str`) adds lifetime complexity but provides microscopic, unprovable performance gains. Attempts to optimize this should be avoided under `perf-proof` gates unless a new bottleneck is identified.
