## 💡 Summary
Replaced upfront string allocations in derived metrics logic with non-allocating reference processing.

## 🎯 Why
`derive_report` was blindly performing `path.clone()`, `module.clone()`, and `lang.clone()` for every file in the codebase simply to build a statistical digest where the majority of files are discarded (e.g. they don't make the Top 10 lists). In very large codebases, this generates hundreds of thousands of needless allocations on the hot path of report generation, burning CPU and heap.

## 🔎 Evidence
- `crates/tokmd-analysis/src/derived/files.rs` aggressively cloned memory upfront using `rows.iter().map(|r| FileStatRow { path: r.path.clone(), ... })`.
- A newly introduced structural benchmark `derived_alloc_reduction_proof` completes in ~2 seconds over 100,000 files by operating entirely on references and only calling `to_owned()` if the file is ultimately selected for output.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `crates/tokmd-analysis/src/derived/files.rs` to process an intermediate `FileStatRef<'a>` containing references and defer full `FileStatRow` instantiation until filtering is complete.
- why it fits this repo and shard: This fits cleanly within the analysis shard by avoiding unnecessary allocations while keeping outputs perfectly deterministic.
- trade-offs:
  - Structure: Improves internal analytics architecture slightly.
  - Velocity: Meaningfully faster over larger repositories.
  - Governance: Maintains all existing APIs and determinism guarantees (no change to JSON payloads).

### Option B
- what it is: Do nothing, ignore intermediate buffer bloat.
- when to choose it instead: If structural changes broke tests or complexity was unwarranted.
- trade-offs: We continue bleeding memory via string allocations for every single file.

## ✅ Decision
Option A. It's a clean, non-invasive structural change that limits String cloning exclusively to the files that make the final report.

## 🧱 Changes made (Srp)
- `crates/tokmd-analysis/src/derived/files.rs`: Introduced `FileStatRef` to defer `.clone()`.
- `crates/tokmd-analysis/src/derived/mod.rs`: Passed `FileStatRef` lists through internal functions.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis derived_alloc_reduction_proof -- --nocapture
cargo build --verbose
CI=true cargo test --verbose -p tokmd-analysis
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Optimization
- Blast radius: `crates/tokmd-analysis` / derived metrics (strictly internal optimization)
- Risk class + why: Low. The deterministic nature of `.unwrap_or` and `FileRow` fields guarantees the same records are produced.
- Rollback: Revert the PR.
- Gates run: `perf-proof` (structural benchmark written), build, test, clippy, fmt.

## 🗂️ .jules artifacts
- `.jules/runs/run-bolt-analysis-stack/envelope.json`
- `.jules/runs/run-bolt-analysis-stack/decision.md`
- `.jules/runs/run-bolt-analysis-stack/receipts.jsonl`
- `.jules/runs/run-bolt-analysis-stack/result.json`
- `.jules/runs/run-bolt-analysis-stack/pr_body.md`

## 🔜 Follow-ups
None.
