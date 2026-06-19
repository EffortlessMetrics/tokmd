## Options Considered

### Option A: Improve path redaction determinism test
- **What it is:** The `clean_path` logic in `crates/tokmd-format/src/redact/mod.rs` strips redundant paths (`./`, `/./`). However, it doesn't correctly handle resolving parent segments (`..`). The memory note explicitly says: "path redaction and normalization logic (such as `clean_path`) must correctly resolve parent directory segments (`..`) and unify path separators to guarantee that logically identical paths produce deterministic hashes, preventing directory structure leakage."
I'll add the test that fails right now (e.g. `crates/tokmd/../foo/lib.rs` and `crates/foo/lib.rs` yielding different hashes) and then fix it. I will write an explicit test for it and then improve `clean_path`.
- **Trade-offs:** High value for security/determinism, explicitly requested in memory.

### Option B: Fix the token audit logic
- **What it is:** Not exactly what the memory mentions.

## Decision
Option A. It's explicitly listed in the memory file as a requirement, and the test fails. It improves the test suite to catch meaningful code changes and prevents directory structure leakage. I'm Mutant, so I want to improve tests around high-value production surfaces with weak assertions and fix the gap. Wait, I'm mutant: "Target ranking: 1) improve tests around a high-value production surface with weak assertions 2) close a concrete missed-mutant-style gap... You may land a proof-improvement patch even if it does not change production code. If the strongest target you find is outside the shard, record it as friction instead of chasing it. Primary shard: core-pipeline. Primary paths: crates/tokmd-format/**". I'm allowed to touch `tokmd-format`.

Actually, wait. If my persona is Mutant, the primary job is to "reduce uncertainty by improving tests... close a concrete assertion gap". Modifying production code to fix a logical gap without adding robust new behavioral tests violates persona instructions. But wait, if I *both* fix the gap *and* add the tests, is that okay? Yes! "Modifying production code to fix a logical gap without simultaneously fulfilling the core directive of adding robust new behavioral tests to prove the fix violates persona instructions." - meaning, if I modify production code I MUST add tests.
