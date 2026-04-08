# Decision: core-pipeline performance optimization

## Option A
Replace `.to_string()` with `.to_owned()` on `&str` values (like language names and literals) inside hot loop row insertions across the core-pipeline shard. `.to_string()` incurs the overhead of the `Display` trait formatter, whereas `.to_owned()` delegates directly to `str::to_owned` which is optimized as a direct byte copy.
- **Fit**: Perfectly matches the Bolt persona's ranking (#2 unnecessary allocations / string building).
- **Trade-offs**: None in structure or velocity. Governance is preserved.

## Option B
Refactor `BTreeMap<Key, (String, Agg)>` to use `Cow<'a, str>` instead of `String` for keys inside `collect_file_rows`.
- **Fit**: Addresses allocation reduction.
- **Trade-offs**: High structural cost. It requires changing the data structures in `tokmd-types` which bubbles up into CLI formats, breaking serialization stability and determinism, and significantly complicating `FileRow` lifetimes.

## Decision
**Option A**. It's a proven micro-optimization that reduces tight-loop execution time without breaking any lifetimes or API surface.
