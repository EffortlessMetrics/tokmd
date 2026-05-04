# Option A: Fix `String.prototype.localeCompare` determinism drift in `web/runner/ingest.js`

- **What it is**: The `localeCompare` method in JavaScript uses platform-dependent collation rules. Replacing it with strict Unicode code-unit comparisons (`leftPath < rightPath ? -1 : leftPath > rightPath ? 1 : 0`) guarantees consistent sorting of ingested entries regardless of the execution environment's locale settings.
- **Why it fits**: The prompt specifies that this shard (`bindings-targets`) contains a JS/Node environment (`web/runner`), and one of the target compat issues is "determinism drift caused by platform behavior". Memory also states: "In JS/Node environments (like `web/runner/`), `String.prototype.localeCompare()` is platform-dependent and causes determinism drift. Use strict Unicode code-unit comparisons... to ensure determinism and match Rust's native lexicographical `String::cmp` and `BTreeMap` sorting behavior."
- **Trade-offs**:
    - *Structure*: Ensures deterministic input ordering across all targets (browser vs. Node vs. Rust native).
    - *Velocity*: Quick fix that is easy to reason about.
    - *Governance*: Aligns cross-platform behavior by using standard Unicode sort.

# Option B: Investigate other --no-default-features failures

- **What it is**: Check if `crates/tokmd-wasm`, `crates/tokmd-node`, or `crates/tokmd-python` fail when compiling with `--no-default-features`.
- **When to choose it instead**: If `localeCompare` was just a minor issue and there is a more significant feature interaction bug.
- **Trade-offs**: Might take more time to find, and memory explicitly points out the `localeCompare` issue as a known drift source in `web/runner`.

## Decision
**Option A**. It's a known drift issue explicitly mentioned in memory for this exact path (`web/runner/ingest.js`). Fixing `localeCompare` to use strict comparisons directly addresses "determinism drift caused by platform behavior" which ranks as a valid target for the "Compat" persona.
