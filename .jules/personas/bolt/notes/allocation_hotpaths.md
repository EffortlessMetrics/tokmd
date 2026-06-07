## Allocation Hotpaths

When dealing with high-iteration loops aggregating string properties (like file paths in git commits or tokens in text analysis), avoiding `String::clone()` inside the loop yields massive gains.

**Key technique applied:**
When you already have a "master map" or lookup table that owns the `String` allocations (like `row_map` in `tokmd-analysis`), you can use `BTreeMap::get_key_value` to retrieve a reference to the existing `String` key as an `&str` reference.
You can then use these `&str` references to key transient secondary maps built during the same scoped iteration, dramatically reducing allocations.

Example:
```rust
// Instead of:
// *commit_counts.entry(key.clone()).or_insert(0) += 1;

// Do this, if row_map owns the string logic and you know the file must exist in it:
if let Some((map_key, &(_row, module))) = row_map.get_key_value(&key) {
    *commit_counts.entry(map_key.as_str()).or_insert(0) += 1;
}
```
