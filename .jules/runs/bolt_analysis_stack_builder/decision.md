# Decision

## 🎯 Why
In EffortlessMetrics/tokmd, the analysis stack has a hot path where `build_file_stats` takes an array of `&FileRow` and clones each file's `path`, `module`, and `lang` strings to create `FileStatRow`s. This happens on every run. Furthermore, the downstream functions (`build_max_file_report`, `build_top_offenders`) take owned `Vec<FileStatRow>` or slices of them, causing further cloning. By passing `&FileRow` references down and only doing the conversion (`to_stat_row`) at the very end when generating the reports, we can eliminate a significant number of string allocations (10x-50x depending on codebase size).

## 🧭 Options considered

### Option A (recommended)
- what it is: Refactor `crates/tokmd-analysis/src/derived/files.rs` and its caller in `mod.rs`. Remove `build_file_stats`. Change `build_max_file_report` and `build_top_offenders` to accept `&[&FileRow]` (which we already have as `parents`). We map to `FileStatRow` only at the edges when building the final `MaxFileReport` and `TopOffenders` structs. We also adapt `build_nesting_report` in `mod.rs` to take `&[&FileRow]` instead of `&[FileStatRow]`.
- why it fits this repo and shard: It targets a core hot path in the analysis shard (`tokmd-analysis`) reducing unnecessary allocations and string building.
- trade-offs:
  - **Structure**: Low risk, localized to `derived/files.rs` and `derived/mod.rs`.
  - **Velocity**: Speeds up analysis by bypassing a massive upfront allocation map.
  - **Governance**: Fits the `perf-proof` profile by providing a clear structural proof of allocation reduction.

### Option B
- what it is: Use `Cow<'a, str>` inside `FileStatRow`.
- when to choose it instead: If the `FileStatRow` needs to exist for an extended period without tying up the lifetime of `FileRow`.
- trade-offs: More invasive, breaks public types in `tokmd-analysis-types`.

## ✅ Decision
Option A. It's safe, reduces thousands of string clones, passes all tests, and keeps the public DTOs in `tokmd-analysis-types` intact.
