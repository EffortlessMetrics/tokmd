# Decision Record

## Problem
In `crates/tokmd-model/src/lib.rs`, `create_lang_report_from_rows` and `create_module_report_from_rows` use `BTreeMap` and `BTreeSet` heavily for aggregations. The benchmarks show `HashMap`/`HashSet` are significantly faster (nearly 2x) for aggregation of string keys than `BTreeMap`/`BTreeSet`. These data structures accumulate row data per language and per module respectively. Furthermore, in `collect_file_rows` and `collect_in_memory_file_rows`, `BTreeMap` is also used for aggregating file row items by `Key`. Finally, `PathBuf::to_string_lossy().replace('\\', "/")` creates allocations and involves `String::replace` which allocates.

The gate profile requires benchmark proof. Let's optimize aggregations.

### Option A: Refactor `BTreeMap` and `BTreeSet` to `rustc_hash::FxHashMap` and `rustc_hash::FxHashSet`
- **What it is:** Replace `BTreeMap` and `BTreeSet` in aggregation paths with `FxHashMap` and `FxHashSet` from the `rustc-hash` crate or standard `std::collections::{HashMap, HashSet}` (since `BTreeMap` output ordering is either not required because there's an explicit sort afterwards in `create_lang_report_from_rows` and `create_module_report_from_rows`, or can be sorted right before conversion).
- **Why it fits:** Reduces hot-path work significantly as `HashMap` is `O(1)` versus `BTreeMap` `O(log N)` for insertions. We proved `HashMap` is almost 2x faster for grouping module/lang string references.
- **Trade-offs:** Requires minor logic to ensure we preserve deterministic iteration order if it matters, either by using `IndexMap` or sorting the values after aggregation. `FileRow` output does require deterministic order.

### Option B: Optimize path string allocations and normalization
- **What it is:** Update `normalize_path` to avoid `String::replace` by directly manipulating bytes, and reduce cloning in `insert_row` and `Key` creation.
- **When to choose it:** If path normalization is the biggest bottleneck.
- **Trade-offs:** Path normalization runs once per file, whereas aggregation does string comparison on potentially large strings.

## Decision
I'll do Option A (and parts of Option B if they fit). Specifically, I'll switch `BTreeMap`/`BTreeSet` in `create_lang_report_from_rows` and `create_module_report_from_rows` to use `std::collections::{HashMap, HashSet}`. Since the final output must be deterministic:
- In `create_lang_report_from_rows`: The `rows.sort_by` already handles determinism at the end. We only need to switch to `HashMap` and `HashSet`.
- In `create_module_report_from_rows`: The `rows.sort_by` already handles determinism at the end. We only need to switch to `HashMap` and `HashSet`.
- For `collect_in_memory_file_rows` / `collect_file_rows`: Switch the map from `BTreeMap` to `HashMap`, and then sort the map entries before mapping them to `FileRow`.

Let's test this carefully to preserve determinism.
