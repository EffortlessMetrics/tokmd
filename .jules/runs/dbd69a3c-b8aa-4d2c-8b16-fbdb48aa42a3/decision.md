# Decision

## Inspected Surfaces
* `crates/tokmd-types/Cargo.toml`
* `crates/tokmd-scan/Cargo.toml`
* `crates/tokmd-model/Cargo.toml`
* `crates/tokmd-format/Cargo.toml`

## Options Considered

### Option A: Remove Unused Dependencies
* **What it is:** Using `cargo machete` and manual source code inspection to find and remove unused dependencies in the `core-pipeline` shard.
* **Why it fits:** Directly aligns with the Auditor persona's primary mission of dependency hygiene and constraint tightening.
* **Trade-offs:** If the crates are already clean, attempting to force a removal would lead to broken builds or hallucinated patches.

### Option B: Generate a Learning PR
* **What it is:** Documenting that the dependency hygiene in the `core-pipeline` crates is currently well-maintained and requires no immediate actionable patches.
* **Why it fits:** Complies with the prompt's hard constraints: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
* **Trade-offs:** Doesn't directly reduce the dependency footprint but avoids creating unnecessary churn and preserves the learning for future runs.

## Final Decision
**Option B (Learning PR) selected.**
After running `cargo machete` on `tokmd-types`, `tokmd-scan`, `tokmd-model`, and `tokmd-format`, no unused direct dependencies were found. Manual inspection of `tempfile`, `serde_json`, `anyhow`, `uuid`, and `clap` confirmed their usage either in runtime logic or correctly placed under `dev-dependencies`. Feature tightening attempts resulted in compilation failures, demonstrating that the current manifest constraints are accurate and necessary.
