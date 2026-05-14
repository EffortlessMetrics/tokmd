## Options Considered

### Option A: Remove String allocations in duplicate analysis hot loop
- **What it is:** Change `BTreeMap<String, ...>` to `BTreeMap<&str, ...>` in `build_duplicate_report` (inside `tokmd-analysis/src/content/mod.rs`). Replace redundant `get_mut` / `insert` blocks with `entry(module).or_default()`.
- **Trade-offs:** Clean win. Reduces repeated hashing/lookups and removes string copying completely during the hot loop of counting duplicate and wasted files by module. Very aligned with Bolt's "hot-path work reduction" and "unnecessary string building".

### Option B: Partial sorting in `build_top_offenders`
- **What it is:** Use `select_nth_unstable` in `tokmd-analysis/src/derived/files.rs` to avoid full `O(N log N)` sorting on large file trees.
- **Trade-offs:** `select_nth_unstable` requires mutable, owned vectors, so we'd still have to allocate vectors. And while it saves sorting time, the duplicate report string building happens per duplicate file group, which can be significant.

## ✅ Decision
We will proceed with Option A because `tokmd-analysis/src/content/mod.rs` does repetitive `to_string()` allocations and double map lookups in a hot loop (iterating every duplicate file). By binding the module strings to the lifetime of the input `ExportData` and using the `Entry` API natively, we remove the string allocations and halve the map lookups.
