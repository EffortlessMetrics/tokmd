## 💡 Summary
Reduced memory allocations during derived analysis report generation. I refactored the file stat processing flow to use string slices (`&str`) through a new `FileStatView` struct rather than eagerly cloning all strings (`String`) for every processed `FileRow`.

## 🎯 Why
In large repositories (e.g. 10,000 files), `build_file_stats` allocates `FileStatRow` for every single file. Each `FileStatRow` clones `path`, `module`, and `lang`, causing a massive amount of unnecessary heap allocations (e.g. 30,000 strings). The `DerivedReport` only stores these rows for the top outliers (`max_file` and `top_offenders`). The remaining ~99% of allocations are discarded immediately after computing stats, introducing avoidable memory pressure and overhead.

## 🔎 Evidence
In `crates/tokmd-analysis/src/derived/files.rs`:
```rust
pub(super) fn build_file_stats(rows: &[&FileRow]) -> Vec<FileStatRow> {
    rows.iter()
        .map(|r| FileStatRow {
            path: r.path.clone(),
            module: r.module.clone(),
            lang: r.lang.clone(),
            ...
```

## 🧭 Options considered
### Option A (recommended)
- Change `build_file_stats` to generate an array of `FileStatView<'a>` which borrows the strings rather than owning them. Convert this view to `FileStatRow` only for the few elements that are selected as top offenders/max files.
- Why it fits: Solves the performance problem locally within the builder, without modifying the serialized contract (`tokmd_analysis_types`).
- Trade-offs: Requires a new internal struct `FileStatView` but significantly reduces memory pressure.

### Option B
- Modify the entire `tokmd_analysis_types` crate schema to use `Cow<'a, str>` or reference lifetimes in its DTOs.
- When to choose: If zero-allocation serialization across the entire stack was a priority.
- Trade-offs: Highly intrusive change across crates. Affects JSON serialization logic.

## ✅ Decision
Option A was chosen. It allows us to drop the thousands of wasted allocations in `tokmd-analysis` without breaking downstream consumers of the `tokmd-analysis-types` crate.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/files.rs`:
  - Added `FileStatView<'a>` struct to track intermediate statistics.
  - Updated `build_file_stats`, `build_max_file_report`, and `build_top_offenders` to process views and invoke `.into_row()` only on the selected outliers.
- `crates/tokmd-analysis/src/derived/mod.rs`:
  - Updated `build_nesting_report` to process `FileStatView<'a>` and avoid `String` key clones during its internal grouping loop.

## 🧪 Verification receipts
```text
$ cargo clippy -p tokmd-analysis -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.96s

$ CI=true cargo test -p tokmd-analysis --verbose
...
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
...
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
...
```

## 🧭 Telemetry
- Change shape: Internal structural optimization
- Blast radius: Low risk; isolated to `tokmd-analysis` internal build steps.
- Risk class: Low - no serialization schema updates.
- Rollback: Revert the PR.
- Gates run: `perf-proof`

## 🗂️ .jules artifacts
- `.jules/runs/run-bolt-builder-001/envelope.json`
- `.jules/runs/run-bolt-builder-001/decision.md`
- `.jules/runs/run-bolt-builder-001/receipts.jsonl`
- `.jules/runs/run-bolt-builder-001/result.json`
- `.jules/runs/run-bolt-builder-001/pr_body.md`

## 🔜 Follow-ups
N/A
