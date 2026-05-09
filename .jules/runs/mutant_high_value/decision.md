## Options Considered

### Option A: Expand test assertions for `env_interpreter_token` (Recommended)
- **What it is:** The `cargo mutants` tool indicated that several match arms within the `tokmd-model` crate's `env_interpreter_token` function are effectively dead code in the eyes of the test suite. If an attacker/regression were to delete `-S`, `--split-string`, or `--ignore-environment` handling, no test would fail. We add tests targeting these cases.
- **Why it fits:** This exactly aligns with the "Mutant" persona and the "Prover" style which tasks me with improving proof surfaces where high value behavior is under-tested.
- **Trade-offs:**
  - Structure: Low risk, isolated to tests.
  - Velocity: Quick to implement, prevents future regressions.
  - Governance: High alignment. Increases test confidence.

### Option B: Delete the 'dead' code
- **What it is:** If the code isn't tested, maybe it's not needed. We could delete the match arms for `-S`, `--split-string`, etc.
- **When to choose it instead:** If this was dead code that the application never actually intended to support.
- **Trade-offs:** The env command does support these flags. Stripping them out just to satisfy a mutant runner degrades the product's ability to accurately detect environments in shell scripts.

## Decision
Proceeding with Option A. We will add test assertions for `--split-string` and `--ignore-environment` in `crates/tokmd-model/src/lib.rs` to close the mutant gap without degrading application behavior.
