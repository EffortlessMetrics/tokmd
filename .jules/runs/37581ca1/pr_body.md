## 💡 Summary
Fixed a redaction correctness bug where specific file extensions (like `md` and `svelte`) were incorrectly stripped from redacted paths because the allowlist array was unsorted.

## 🎯 Why
The `SAFE_PATH_EXTENSIONS` array in `crates/tokmd-format/src/redact/mod.rs` was used with a `binary_search` to safely preserve known file extensions during path redaction. However, the array was not strictly sorted alphabetically (e.g., `md` came after `mjs`, `svelte` came after `svg`). This caused the binary search to return `Err` for those extensions, stripping them during redaction and resulting in overly aggressive (and inconsistent) redaction behavior.

## 🔎 Evidence
- `crates/tokmd-format/src/redact/mod.rs`
- Observed behavior: `md` and `svelte` extensions were stripped during redaction because `binary_search` missed them.
- `cargo test -p tokmd-format --lib` receipt demonstrating the new tests pass, enforcing strictly sorted elements.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Sort the `SAFE_PATH_EXTENSIONS` array alphabetically in `crates/tokmd-format/src/redact/mod.rs` so that `binary_search` succeeds for all items.
- **Why it fits this repo and shard:** The array was already hardcoded as a `const &[&str]`, making it highly efficient for binary searches if sorted. Sorting the array fixes the logical bug with zero runtime overhead or dependency additions, fitting perfectly within the `core-pipeline` shard's boundaries for zero-cost abstraction and redaction correctness.
- **Trade-offs:**
  - **Structure:** Keeps existing architecture completely intact.
  - **Velocity:** Extremely fast and minimal patch.
  - **Governance:** Easy to review, minimal risk.

### Option B
- **What it is:** Replace the sorted slice with a `HashSet` (using `lazy_static`/`OnceLock`) or a perfect hash function set (`phf` crate).
- **When to choose it instead:** When the allowlist is incredibly large, or when the cost of maintaining a sorted array manually is deemed too high by the team.
- **Trade-offs:** Requires introducing additional macro dependencies or initialization overhead. Slower than a simple `binary_search` over ~70 items. Overcomplicates a simple allowlist.

## ✅ Decision
I chose **Option A**. The issue was simply that `md` and `svelte` (and possibly others) were missed because the array was out of alphabetical order. Sorting it properly and adding strict unit tests to ensure it stays sorted resolves the bug while keeping the implementation highly performant and allocation-free.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`: Alphabetically sorted `SAFE_PATH_EXTENSIONS`.
- `crates/tokmd-format/src/redact/mod.rs`: Added `test_safe_path_extensions_are_strictly_sorted` to prevent regression.
- `crates/tokmd-format/src/redact/mod.rs`: Added `test_all_safe_path_extensions_are_preserved` to verify all elements are found by `binary_search`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-format --lib
test result: ok. 138 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

cargo test -p tokmd-types
test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Patch
- Blast radius: Internal redaction utilities across the pipeline.
- Risk class: Low risk. Fixes a correctness issue in redaction boundaries.
- Rollback: Revert the PR.
- Gates run: `cargo test -p tokmd-format`, `cargo test -p tokmd-types`

## 🗂️ .jules artifacts
- `.jules/runs/37581ca1/envelope.json`
- `.jules/runs/37581ca1/decision.md`
- `.jules/runs/37581ca1/receipts.jsonl`
- `.jules/runs/37581ca1/result.json`
- `.jules/runs/37581ca1/pr_body.md`

## 🔜 Follow-ups
None.
