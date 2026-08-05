## 💡 Summary
Updated `tokmd-format::redact::clean_path` to resolve `..` (parent directory) segments logically. This ensures that functionally identical paths like `src/../src/lib.rs` and `src/lib.rs` produce the same redacted hashes, preventing path structure leakage across the trust boundary.

## 🎯 Why
The previous `clean_path` implementation normalized separators and resolved `.` segments, but it completely ignored `..` segments. Because `redact_path` hashes the output of `clean_path`, un-normalized `..` segments meant `src/../src/lib.rs` would hash differently than `src/lib.rs`. This represents a determinism gap and a potential leakage of internal directory structures in redacted outputs, violating the trust boundary requirement that logically identical paths must produce deterministic hashes.

## 🔎 Evidence
- **File:** `crates/tokmd-format/src/redact/mod.rs`
- **Observed behavior:** `redact_path("src/../src/lib.rs")` returned `068192f9dd7b6cd0.rs` while `redact_path("src/lib.rs")` returned `fd6a3d2bf9e43131.rs`.
- **Receipt:** Wrote a test helper `clean_path("src/../src/lib.rs")` which output `"src/../src/lib.rs"` under the old implementation, demonstrating the lack of parent directory resolution.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Re-write `clean_path` to split paths by `/` and logically resolve `..` segments by pushing and popping to a parts vector, while continuing to ignore `.` and `""` (double slashes).
- **Why it fits this repo and shard:** `clean_path` is the canonical path normalization boundary for redaction in `tokmd-format`. Fixing it here ensures all consumers of `redact_path` and `short_hash` automatically gain the hardening.
- **Trade-offs: Structure / Velocity / Governance:** Pure string manipulation keeps the change safe, fast, and free of disk IO, fitting the pure-function boundary of `tokmd-format`.

### Option B
- **What it is:** Use `std::fs::canonicalize` before redaction.
- **When to choose it instead:** When virtual or synthetic paths are never used, and disk access is guaranteed to succeed.
- **Trade-offs:** Fails when paths are synthetic (common in receipts/tests). Introduces disk IO and potential system errors to a pure formatting boundary.

## ✅ Decision
Chose Option A. It correctly hardens the logical trust boundary without side effects, keeping `tokmd-format` pure and ensuring reliable redaction across all platforms and path representations.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`
  - Replaced `clean_path` with an implementation that resolves `..` segments via a vector-based split-and-fold.
  - Added test `test_clean_path_resolves_parent_segments` to prove `clean_path` behavior.
  - Added test `test_redact_path_normalizes_parent_segments` to prove redaction equivalence.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-format
test result: ok. 240 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
$ cargo build --verbose
Success
$ cargo fmt -- --check
Success
$ cargo clippy -- -D warnings
Success
```

## 🧭 Telemetry
- **Change shape:** Patch to path string normalization logic + unit tests.
- **Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies):** Compatibility/Determinism. Changes the output hash for paths containing `..`, but only to make them correctly align with the hashes of their normalized equivalents. No API or IO changes.
- **Risk class + why:** Low risk. It's a pure string manipulation hardening boundary that strictly expands correctness.
- **Rollback:** `git revert`.
- **Gates run:** `security-boundary` fallback expectations: `cargo test -p tokmd-format`, `cargo clippy`, `cargo fmt`.

## 🗂️ .jules artifacts
- `.jules/runs/run_sentinel_001/envelope.json`
- `.jules/runs/run_sentinel_001/decision.md`
- `.jules/runs/run_sentinel_001/receipts.jsonl`
- `.jules/runs/run_sentinel_001/result.json`
- `.jules/runs/run_sentinel_001/pr_body.md`

## 🔜 Follow-ups
None.
