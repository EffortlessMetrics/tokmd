## 💡 Summary
Fixed a boundary edge case in `is_test_path` where test directories at the root of a parsed path were incorrectly categorized as regular source code, and tightened property tests to guarantee its invariant.

## 🎯 Why
The `is_test_path` utility uses substring matching with slashes (e.g., `/test/`, `/tests/`, `/__tests__/`) to enforce segment boundaries. However, if a path was normalized such that the root itself was a test directory (e.g., `test/foo.rs`), the leading slash was absent and the directory fell through the logic. This caused drift where tests were inadvertently analyzed as standard production logic simply based on repository folder location. Furthermore, there was no property-based testing covering this root directory invariant.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-analysis-types/src/util.rs`
- Creating a scratch program that evaluated `is_test_path("test/foo.rs")` returned `false`, while `is_test_path("src/test/foo.rs")` returned `true`.
- Added new `proptest!` blocks in `crates/tokmd-analysis-types/src/util/tests/properties.rs` (e.g., `is_test_path_identifies_root_directories`) to explicitly guarantee the invariant that a root directory with a valid test name is correctly categorized as a test.

## 🧭 Options considered
### Option A (recommended)
- Prepend a `/` to the normalized lowercase string before doing the directory boundary substring check, and add missing property tests for root directory invariants.
- why it fits this repo and shard: It restores the intended boundary matching behavior for test classification without fundamentally altering the existing test criteria, while strengthening invariant guarantees.
- trade-offs: Structure: Preserves existing structure. Velocity: Quick, focused patch. Governance: Improves analysis correctness globally.

### Option B
- Split the path into segments and manually check each segment for test-directory markers.
- when to choose it instead: If the overhead of formatting a string `/foo...` on each file was identified as a critical hot-path bottleneck.
- trade-offs: More verbose, requires re-implementing existing robust suffix checks.

## ✅ Decision
Option A. Safely aligns `is_test_path` with root directory patterns without risking behavioral regressions, and tightly locks in the new capability via new property tests.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis-types/src/util.rs`: Prepended a slash to `lower` so `lower_slash.contains("/test/")` successfully targets root test directories.
- `crates/tokmd-analysis-types/src/util/tests/properties.rs`: Added 4 new `proptest!` property-based tests strictly verifying boundary conditions around `is_test_path` logic for `test`, `tests`, `__tests__`, `spec`, and `specs` directories located at the root of a path, as well as rejecting non-test root directories.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis-types --test properties
cargo test -p tokmd-analysis-types
```

## 🧭 Telemetry
- Change shape: Logic fix and property test additions
- Blast radius: API/analysis stats
- Risk class: Low, only affects classification.
- Rollback: Revert the files.
- Gates run: `cargo check`, `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
