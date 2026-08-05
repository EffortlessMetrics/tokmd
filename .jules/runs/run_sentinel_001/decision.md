## Options considered

### Option A (recommended)
- **What it is:** Update `clean_path` in `crates/tokmd-format/src/redact/mod.rs` to properly resolve `..` (parent directory) segments so that paths like `../src/lib.rs` and `src/../src/lib.rs` are normalized correctly before redaction (hashing). This prevents identical logical paths from producing different hashes and potentially leaking directory structures.
- **Why it fits:** `clean_path` is used by `redact_path` and `short_hash`, which are the foundational functions for trust-boundary redaction of sensitive paths across the entire pipeline. Fixing `clean_path` closes a trust boundary path normalization gap.
- **Trade-offs:**
  - **Structure:** Keeps changes localized to `tokmd-format::redact::clean_path` while improving security boundary determinism globally.
  - **Velocity:** Low-risk, pure functional change with tests proving correctness.
  - **Governance:** Fits perfectly within Sentinel's mission of redaction correctness and trust boundary hardening.

### Option B
- **What it is:** Modify the CLI or analysis pipeline to perform path canonicalization via `std::fs::canonicalize` prior to passing paths to `redact_path`.
- **When to choose it instead:** When the system relies on physical file existence on disk at redaction time instead of logical path strings.
- **Trade-offs:** Introduces IO operations on what should be a pure formatting boundary, creating potential panics or errors if paths no longer exist or are virtual/synthetic (which happens often in tests or receipts). Increases risk and blast radius.

## Decision
I am choosing Option A. It's a low-risk, high-confidence pure functional change that hardens the logical path normalization and redaction layer, closing a potential structure leak gap in how `..` segments are handled, without relying on physical disk canonicalization.
