# Decision

## 🧭 Options considered

### Option A (recommended)
- **What it is:** Polish and tighten assertion specificity in error/help CLI tests (e.g. `tests/cli_error_paths_w51.rs`, `tests/cli_errors_w66.rs`, and `tests/error_handling_w70.rs`) by changing vague `.stderr(predicate::str::is_empty().not())` to explicit checks for known hints like `"invalid value"`, `"not found"`, etc.
- **Why it fits this repo and shard:** The assignment explicitly calls for "scenario-driven sharp-edge polish" and warns about vague `.stderr(predicate::str::is_empty().not())` in tests as an anti-pattern. This is explicitly covered in the `.jules` memories!
- **Trade-offs:**
  - Structure: Better locked-in behavior for error outputs.
  - Velocity: High; clear path to updating multiple test files.
  - Governance: Fits the Specsmith persona perfectly (fixing test assertions).

### Option B
- **What it is:** Investigate edge cases in `handoff` context strategies, e.g., finding edge-cases that are not covered around budget tokens limit and strategy combinations.
- **When to choose it instead:** If there is a clear missing edge-case in budget parsing or boundary limits that is not asserted.
- **Trade-offs:** Requires deep digging to find a legitimate uncovered path, which might not exist or might require cross-shard changes.

## ✅ Decision
I will go with **Option A**. The memory specifically notes: `When writing or updating CLI integration tests in the tokmd workspace, avoid using vague assertions like .stderr(predicate::str::is_empty().not()). Instead, use explicit .stderr(predicates::str::contains("...")) assertions to strictly validate specific error hints and subcommand suggestions.` We will find instances of this in W51, W66, and W70 tests and replace them with specific assertions.
