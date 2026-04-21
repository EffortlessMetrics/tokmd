# Run Context
- **Prompt ID:** `gatekeeper_determinism`
- **Persona:** Gatekeeper ✅
- **Style:** Prover
- **Primary Shard:** core-pipeline

## Investigation

The determinism tests in `crates/tokmd/tests/` rely on a regex to scrub dynamic values (e.g., timestamps and versions) so that outputs can be compared byte-for-byte.
The specific replacements used regex patterns like `r#""generated_at_ms":\d+"#`.
These patterns assume no whitespace between the colon and the number, which may fail if a tool or JSON serializer formats the document differently (e.g., `"generated_at_ms": 123`).

This brittleness was found in the core invariant determinism tests, which makes them less resilient and introduces test-suite determinism drift risks.

## Options Considered

### Option A: Update normalization regexes to handle optional whitespace (Recommended)
- **What it is:** Update the regex replacements across all determinism tests to use `\s*` after the colons.
- **Why it fits:** Fixes the immediate issue with determinism regressions and snapshot failures by making the tests more robust against minor formatting changes.
- **Trade-offs:**
  - Structure: Minimal change to test infrastructure.
  - Velocity: Fast to implement and immediate protection.
  - Governance: Ensures determinism tests won't flake out from serializing diffs.

### Option B: Replace regex matching with `serde_json` parsing and redaction
- **What it is:** Parse the JSON with `serde_json`, nullify the fields like `generated_at_ms` and `tool.version`, and re-serialize.
- **When to choose it instead:** If the structure of the JSON gets far too complex or we need guarantees that the text being processed is fully valid JSON.
- **Trade-offs:** Might mask non-JSON formatting errors in raw output that regex processing doesn't.

## Decision

**Option A**. Updating the regex logic is the most direct fix that retains the byte-for-byte exactness check on the raw string output while ignoring just the volatile fields. Serde parsing would normalize object key ordering entirely, which ruins the purpose of tests specifically asserting that keys are output in alphabetical order.
