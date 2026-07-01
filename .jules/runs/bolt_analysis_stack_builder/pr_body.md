## 💡 Summary
Removed `build_file_stats` and deferred `FileStatRow` creation (and string cloning) until final report struct generation. This reduces allocations across the analysis stack by preventing O(N) clones of `path`, `module`, and `lang` on every `FileRow`.

## 🎯 Why
During the analysis hot path, `build_file_stats` mapped every `&FileRow` to an owned `FileStatRow`, triggering 3 string clones per file. The downstream `build_max_file_report` and `build_top_offenders` methods would then sort and take the top N. We were cloning strings for thousands of files just to throw them away.

## 🔎 Evidence
- `crates/tokmd-analysis/src/derived/files.rs`
- Structural proof: Replaced `pub(super) fn build_file_stats(rows: &[&FileRow]) -> Vec<FileStatRow>` with a deferred `to_stat_row` map at the boundaries of `build_max_file_report` and `build_top_offenders`.
- Test receipt:
```text
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Options considered
### Option A (recommended)
- Refactor `crates/tokmd-analysis/src/derived/files.rs` and its caller in `mod.rs`. Change `build_max_file_report`, `build_top_offenders`, and `build_nesting_report` to accept `&[&FileRow]`. Map to `FileStatRow` only at the edges when building the final `MaxFileReport` and `TopOffenders` structs.
- Fits the `perf-proof` profile by providing a clear structural proof of allocation reduction in the `analysis-stack` shard.
- Trade-offs: Structure is localized; Velocity improves by skipping unnecessary clones; Governance is maintained by keeping output types identical.

### Option B
- Use `Cow<'a, str>` inside `FileStatRow`.
- Choose this if the `FileStatRow` needs to exist for an extended period without tying up the lifetime of `FileRow`.
- Trade-offs: More invasive, breaks public types in `tokmd-analysis-types`.

## ✅ Decision
Option A. It's safe, reduces thousands of string clones, passes all tests, and keeps the public DTOs in `tokmd-analysis-types` intact.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/files.rs`: Removed `build_file_stats`, added `to_stat_row`, updated `build_max_file_report` and `build_top_offenders` to use `&[&FileRow]`.
- `crates/tokmd-analysis/src/derived/mod.rs`: Passed `&parents` directly to reports, adapted `build_nesting_report` to compute depth on the fly.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis --test orchestrator
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Refactoring for performance
- Blast radius: API: none, IO: none, docs: none, schema: none, concurrency: none, compatibility: none, dependencies: none
- Risk class: Low, isolated refactoring inside module boundaries.
- Rollback: Revert the PR.
- Gates run: `cargo build --verbose`, `CI=true cargo test -p tokmd-analysis --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`

## 🔜 Follow-ups
None.
