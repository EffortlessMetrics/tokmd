## 💡 Summary
Reduced repeated string allocations and BTreeMap lookups inside the hot-path duplicate file analysis loop by utilizing the `Entry` API with `&str` keys instead of `String`.

## 🎯 Why
In `build_duplicate_report`, every duplicate file iteration was performing redundant `BTreeMap::get_mut` followed by `BTreeMap::insert` allocations for `module.to_string()`. This caused unnecessary string building and double lookups.

## 🔎 Evidence
- File: `crates/tokmd-analysis/src/content/mod.rs`
- Finding: Redundant `String` copies in the hot loop counting duplicates by module.
- Receipt: Cargo tests passed successfully without allocations.

## 🧭 Options considered
### Option A (recommended)
- What it is: Use `&str` bound to the `ExportData` row lifetime and the `Entry` API.
- Why it fits: Aligns perfectly with Bolt's focus on hot-path work reduction and removing unnecessary allocations inside analysis loops.
- Trade-offs: Structure is cleaner; no velocity or governance impact.

### Option B
- What it is: Sort vectors partially in `build_top_offenders`.
- When to choose it instead: When memory footprints in the top offenders map dwarf duplicated metrics building.
- Trade-offs: Harder to prove performance improvements and limits dataset size optimizations.

## ✅ Decision
Chose Option A to cleanly eliminate repetitive string building and duplicate map lookups in a hot loop.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/content/mod.rs`

## 🧪 Verification receipts
cargo test -p tokmd-analysis --verbose
cargo fmt -- --check

## 🧭 Telemetry
- Change shape: Performance optimization
- Blast radius: None
- Risk class: Low
- Rollback: `git checkout crates/tokmd-analysis/src/content/mod.rs`
- Gates run: perf-proof, core-rust

## 🗂️ .jules artifacts
- `envelope.json`
- `decision.md`
- `receipts.jsonl`
- `result.json`
- `pr_body.md`

## 🔜 Follow-ups
None
