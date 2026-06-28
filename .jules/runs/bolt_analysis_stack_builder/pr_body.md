## 💡 Summary
Refactored `is_text_like` logic into `as_text` to eliminate double-UTF-8 passes and unnecessary allocations. By doing this, we avoid doing a UTF-8 validation check and then immediately calling `String::from_utf8_lossy` (which also does UTF-8 validation and allocates a `Cow`).

## 🎯 Why
The codebase frequently checked if a file was text by running `std::str::from_utf8(bytes).is_ok()` inside `is_text_like`, and then immediately called `String::from_utf8_lossy(&bytes)` to get the text. This meant every byte of every text file was scanned for UTF-8 validity twice. Additionally, `String::from_utf8_lossy` returns a `Cow<'_, str>` which, while often borrowed, still requires wrapping and matching. Returning the parsed `&str` directly from the initial check eliminates the second pass and provides a clean `&str` directly.

## 🔎 Evidence
- `crates/tokmd-analysis/src/content/io/bytes.rs`: `is_text_like` performs `std::str::from_utf8(bytes).is_ok()`.
- `crates/tokmd-analysis/src/content/mod.rs` (and others like `api_surface/report.rs`, `complexity/mod.rs`, `halstead/mod.rs`): Immediately after checking `is_text_like`, the code does `let text = String::from_utf8_lossy(&bytes);`.
- `crates/tokmd-analysis/src/content/io.rs`: `as_text` introduced to solve this.

## 🧭 Options considered
### Option A
- Read text directly from bytes using `std::str::from_utf8(&bytes).unwrap()` instead of `String::from_utf8_lossy`.
- Fits the repo as it eliminates the `String::from_utf8_lossy` wrapper but still requires two UTF-8 passes (one in `is_text_like`, one in `from_utf8`).
- Trade-offs: Structure/Velocity/Governance - Minimal structure change, but still leaves repeated work.

### Option B (recommended)
- Introduce `as_text(bytes: &[u8]) -> Option<&str>` to perform the check and return the valid string slice in one pass.
- When to choose it instead: When we can eliminate both the repeated work (second UTF-8 validation) and the unnecessary `Cow` allocation wrapper.
- Trade-offs: Requires updating callers to use the new `as_text` instead of `is_text_like` + `String::from_utf8_lossy`.

## ✅ Decision
Option B. It aligns with target ranking #3 (repeated parsing/formatting that can be reused) and #2 (unnecessary allocations/cloning). We traverse the bytes once for UTF-8 validation and get a `&str` directly.

## 🧱 Changes made (SRP)
- Added `as_text` to `crates/tokmd-analysis/src/content/io/bytes.rs` and `crates/tokmd-analysis/src/content/io.rs`.
- Updated `crates/tokmd-analysis/src/content/mod.rs` to use `as_text` instead of `is_text_like` and `String::from_utf8_lossy`.
- Updated `crates/tokmd-analysis/src/complexity/mod.rs` to use `as_text`.
- Updated `crates/tokmd-analysis/src/halstead/mod.rs` to use `as_text`.
- Updated `crates/tokmd-analysis/src/api_surface/report.rs` to use `as_text`.

## 🧪 Verification receipts
```text
cargo build --verbose
CI=true cargo test -p tokmd-analysis --verbose
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Hot-path work reduction
- Blast radius: Internal API / IO
- Risk class + why: Low risk, purely performance and mechanical transformation of an existing validation pass into a returning validation pass.
- Rollback: Revert the PR
- Gates run: perf-proof, core-rust

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`

## 🔜 Follow-ups
None.
