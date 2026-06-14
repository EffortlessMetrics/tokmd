## 💡 Summary
Removed redundant UTF-8 validation and string allocation in the analysis and content enrichers. Files that passed `is_text_like` (which internally does a UTF-8 check) were being re-checked and allocated via `String::from_utf8_lossy`.

## 🎯 Why
To reduce hot-path work and unnecessary string building. `String::from_utf8_lossy` unconditionally scans the string for invalid UTF-8 and allocates a `Cow`, even when the caller just proved the bytes were valid UTF-8 via `is_text_like()`.

## 🔎 Evidence
- `crates/tokmd-analysis/src/api_surface/report.rs`
- `crates/tokmd-analysis/src/halstead/mod.rs`
- `crates/tokmd-analysis/src/content/mod.rs`
- `crates/tokmd-analysis/src/complexity/mod.rs`
- `crates/tokmd-analysis/src/content/io/read.rs`
- Observed behavior: `is_text_like` returns `true` only for valid utf-8 strings without null bytes. Following this check with `String::from_utf8_lossy` forces an unnecessary secondary pass over the same file buffers.

## 🧭 Options considered
### Option A (recommended)
- what it is: Replace `is_text_like` + `from_utf8_lossy` with a single `std::str::from_utf8` that guards against nulls and returns a `&str` directly without allocating.
- why it fits this repo and shard: It achieves the Bolt persona's goal of removing hot-path validation and redundant allocations while maintaining deterministic structural proof in analysis.
- trade-offs: Structure / Velocity / Governance - slightly changes code shape (using a `match`), but clearly aligns with performance and zero-cost abstraction goals.

### Option B
- what it is: Try to avoid reading files to bytes at all by reading into a `String` directly.
- when to choose it instead: If all files were known to be text.
- trade-offs: Fails gracefully handling binary blobs.

## ✅ Decision
Option A. It optimizes the hot paths directly with minimal structural impact.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/api_surface/report.rs`: Replaced `is_text_like` + `from_utf8_lossy` with `from_utf8`.
- `crates/tokmd-analysis/src/halstead/mod.rs`: Replaced `is_text_like` + `from_utf8_lossy` with `from_utf8`.
- `crates/tokmd-analysis/src/content/mod.rs`: Replaced `is_text_like` + `from_utf8_lossy` with `from_utf8`.
- `crates/tokmd-analysis/src/complexity/mod.rs`: Replaced `is_text_like` + `from_utf8_lossy` with `from_utf8`.
- `crates/tokmd-analysis/src/content/io/read.rs`: Optimized `read_text_capped` to use `from_utf8` instead of unconditional `from_utf8_lossy`.

## 🧪 Verification receipts
```text
cargo check -p tokmd-analysis
cargo test -p tokmd-analysis
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Optimization
- Blast radius: `crates/tokmd-analysis`
- Risk class: Low
- Rollback: Revert the PR
- Gates run: `cargo build --verbose`, `CI=true cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`

## 🔜 Follow-ups
None.
