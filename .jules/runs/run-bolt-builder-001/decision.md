# Decision

## Option A (recommended)
**Change:** Avoid cloning `String` fields (e.g. `path`, `module`, `lang`) when creating `FileStatRow`s in `build_file_stats` and `build_max_file_report`, by having `FileStatRow` store references `&str` instead of owned strings. Wait, `FileStatRow` is a struct exported in `tokmd_analysis_types`, used in `DerivedReport` which is serialized. So we can't easily change `FileStatRow` to use lifetimes without changing the whole DTO tree.
Wait, can we instead clone `&FileStatRow`? In `build_max_file_report` and `build_top_offenders`, we have things like:
```rust
    MaxFileReport {
        overall: overall, // ... cloned
        by_lang: by_lang
            .into_iter()
            .map(|(key, file)| MaxFileRow {
                key: key.to_string(),
                file: file.clone(),
            })
            .collect(),
```
And in `TopOffenders`:
```rust
        largest_lines: by_lines.into_iter().take(TOP_N).cloned().collect(),
```
This requires `FileStatRow` to implement `Clone`, which it does, but this means we are cloning 50 strings (5 arrays * 10 elements) + `by_lang` + `by_module` + `overall` for each analysis run. However, `FileStatRow`s are first allocated in `build_file_stats`.
Another target: In `build_ratio_rows` and `build_rate_rows` in `crates/tokmd-analysis/src/derived/ratios.rs`, `group_ratio` and `group_rate` return `BTreeMap<String, (usize, usize)>`. The `String` is cloned inside the `.into_iter().map(|(k, v)| (k.to_string(), v))` which allocates many strings. If we change it to return `BTreeMap<&'a str, (usize, usize)>` we could save allocations.

## Option B
**Change:** Let's look at `group_ratio` and `group_rate` more carefully. They take `rows: &'a [&'a FileRow]`, and return `BTreeMap<String, (usize, usize)>`. They build an intermediate `BTreeMap<&str, (usize, usize)>`, and then at the end `.into_iter().map(|(k, v)| (k.to_string(), v)).collect()`. These intermediate maps and allocations could be avoided by avoiding the string conversion until needed, but `RatioRow` requires `String` keys anyway.

Let's look at another potential target: `crates/tokmd-analysis/src/derived/ratios.rs`
```rust
fn group_ratio<'a, FKey, FVals>(
    rows: &'a [&'a FileRow],
    key_fn: FKey,
    vals_fn: FVals,
) -> BTreeMap<String, (usize, usize)>
where
    FKey: Fn(&'a FileRow) -> &'a str,
    FVals: Fn(&'a FileRow) -> (usize, usize),
{
    let mut map: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for row in rows {
        let key = key_fn(row);
        let (numer, denom_part) = vals_fn(row);
        let entry = map.entry(key).or_insert((0, 0));
        entry.0 += numer;
        entry.1 += denom_part;
    }
    map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}
```
Is this inefficient? Not really, it's just normal tree building.

Let's check `build_file_stats(&parents)`. This builds `Vec<FileStatRow>` for all files. If there are 10,000 files, we clone `path`, `module`, `lang` 10,000 times! But `FileStatRow` is used to build `max_file` and `top_offenders`. They do not need to own the strings!
If we introduced `FileStatView<'a>` in `tokmd-analysis` which holds `&'a str` instead of `String`, we could calculate `max_file` and `top_offenders` using `FileStatView` instead, and only convert to `FileStatRow` (and clone) the max and top 50 files. This would avoid cloning `path`, `module`, `lang` for *every* file in the repository!

This matches: `2) unnecessary allocations / cloning / string building` and `4) intermediate-buffer reduction`.
In a repo with 10k files, allocating `FileStatRow` 10k times does 30k string allocations. The top offenders only need 50 of them, and max file only needs ~languages+modules.

Wait, `DerivedReport` does NOT store the full array of `FileStatRow`! It only stores `max_file` and `top_offenders`.
Let's check `DerivedReport`:
```rust
pub struct DerivedReport {
    ...
    pub max_file: MaxFileReport,
    ...
    pub top: TopOffenders,
```
There is no `pub file_stats: Vec<FileStatRow>`! The `file_stats` vector in `derive_report` is purely a local intermediate buffer used to compute `max_file` and `top_offenders`, and `nesting`.
```rust
    let file_stats = build_file_stats(&parents);
    let max_file = build_max_file_report(&file_stats);
    let nesting = build_nesting_report(&file_stats);
    let top = build_top_offenders(&file_stats);
```
So we are eagerly allocating `10k * 3` strings just to throw 99% of them away!

Let's implement Option B.
