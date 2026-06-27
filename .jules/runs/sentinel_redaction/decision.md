## Options Considered

### Option A: Fix Path Normalization to Resolve Parent Segments
- **What it is**: Enhance the `clean_path` function in `crates/tokmd-format/src/redact/mod.rs` to correctly resolve `..` segments along with handling absolute paths and separating the existing logic into a more robust stack-based normalization.
- **Why it fits**: The prompt specifically states: "In the `tokmd-format` crate, path redaction and normalization logic (such as `clean_path`) must correctly resolve parent directory segments (`..`) and unify path separators to guarantee that logically identical paths produce deterministic hashes, preventing directory structure leakage." This fits the "redaction correctness and leakage prevention" target perfectly.
- **Trade-offs**:
  - Structure: Improves security invariants of redaction and eliminates directory structure leakage paths.
  - Velocity: Quick to implement, has low blast radius because it's localized to redaction format outputs.
  - Governance: High alignment with `security-boundary` gate profile expectations for deterministic safety.

### Option B: Document the Leak and Leave as-is
- **What it is**: Create a learning PR noting that `..` normalization is missing, without fixing it.
- **When to choose it**: If fixing it proves too complex or if it breaks other systems depending on literal traversal formats.
- **Trade-offs**: Violates explicit mission of landing "one security-significant hardening improvement" when the fix is straightforward.

## Decision
Option A. The `clean_path` function is the canonical source for hash string generation to avoid leakage. Fixing it perfectly aligns with the Sentinel persona and target ranking #1 (redaction correctness and leakage prevention). The logic is purely string-based and well-understood.
