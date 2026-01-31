# Avoid Double Scan and String Rebuild in Aggregation

## Context
In `tokmd-model`, `create_module_report` was iterating over `languages.reports` (raw Tokei output) to build `module_files` mapping. However, it had *already* called `collect_file_rows` which processed the same reports and normalized paths.

## Pattern
Reuse the processed `file_rows` vector instead of re-scanning raw inputs. This avoids:
1. Redundant `normalize_path` calls (CPU + Allocation).
2. Redundant `module_key_from_normalized` calls (CPU).

## Optimization: Zero-Copy Keys
`module_key_from_normalized` was refactored to return `Cow<'a, str>`.
- If the module key is a substring of the path (common case), it returns `Cow::Borrowed`.
- If reconstruction is needed (rare double slashes), it returns `Cow::Owned`.

When inserting into `BTreeMap<String, V>`:
```rust
let key_cow = get_key_cow();
if let Some(val) = map.get_mut(key_cow.as_ref()) {
    // Found without allocating String!
    val.update();
} else {
    // Must allocate String for key storage
    map.insert(key_cow.into_owned(), initial_val);
}
```
This reduces allocations from O(N) to O(M) where N is items and M is unique keys.

## Evidence
- Reduced runtime for 100k files from 345ms to 339ms.
- Structural elimination of 100k string allocations.
