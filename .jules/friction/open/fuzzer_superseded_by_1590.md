# Friction Item: Fuzz invariant extraction superseded

**Persona:** Fuzzer
**Shard:** interfaces
**Context:** Extracting broken `cargo fuzz` targets into deterministic `proptest` suites inside `crates/tokmd/tests/`.

**Friction:**
The task to extract high-signal `scan-args` and `context-policy` invariants from unrunnable fuzz targets was superseded by PR #1590. That PR correctly merged the invariants directly into their respective owner crates rather than centralizing them in `crates/tokmd/tests/`. This indicates that when moving fuzz invariants, they should ideally live alongside the code they test (in the owner crate) rather than a workspace-level integration test suite, assuming the owner crate supports testing them.

**Impact:**
Time spent writing the `tokmd/tests/` proptests was discarded.

**Recommendation:**
Future agents targeting fuzz surfaces should first verify if the owner crates already contain the derived invariant coverage before attempting to port them workspace-wide.
