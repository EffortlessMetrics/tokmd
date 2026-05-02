# Decision

## Option A (recommended)
- **What it is**: Update the length check in `redact_path` in `crates/tokmd-format/src/redact/mod.rs` from `<= 8` to `<= 5`. Standard file extensions (like `.rs`, `.cpp`, `.json`, `.toml`) are typically 2 to 4 characters long, with some rare ones being 5. Limiting the extension length to `<= 5` prevents leaking longer strings that happen to be after a dot (e.g., `file.secret12`), effectively hardening the security boundary while preserving genuine extensions.
- **Why it fits this repo and shard**: The assignment focuses on redaction correctness and leakage prevention in the `core-pipeline` shard, specifically targeting `tokmd-format`. This limits the risk of exposing sensitive data disguised as an extension.
- **Trade-offs**:
  - *Structure*: Minimal modification; updates a hardcoded length threshold.
  - *Velocity*: Very quick and targeted fix.
  - *Governance*: Enhances security of redacted outputs. Might redact uncommon but legitimate 6-8 char extensions (like `.groovy`, `.action`), masking them as a bare hash, which is acceptable under redaction mode.

## Option B
- **What it is**: Implement a strict allowlist of known safe extensions (e.g., `rs`, `js`, `json`, `md`) instead of a length check.
- **Why it fits**: Provides an even stricter security guarantee by completely discarding unknown extensions.
- **Trade-offs**: Requires maintaining an exhaustive list of extensions, which could be cumbersome given the number of languages and file types. Legitimate but unlisted extensions would be fully redacted, potentially confusing users trying to understand the composition of their codebase.

## Decision
I'm proceeding with **Option A** because it is a low-risk, high-confidence improvement that aligns perfectly with the "Stabilizer" style and "Sentinel" persona. It hardens the trust boundary by preventing the leak without the maintenance burden of an allowlist.
