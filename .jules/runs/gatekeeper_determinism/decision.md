# Decision

## Option A (recommended)
Fix deterministic BTreeMap ordering across tests and core crates. The `Prover` persona focuses on improving determinism, tests, and proof surfaces. BTreeMap iterators return entries sorted by key ascending, but testing determinism across platforms and environments often requires specific, predictable multi-key sorting (like code descending, then language ascending) for deterministic serialization.

- **Why it fits this repo and this shard:** The problem of output stability and deterministic testing maps directly to the `Gatekeeper` persona in the `core-pipeline` shard.
- **Trade-offs:** Better test robustness and regression prevention vs slightly more verbose code.

## Option B
Create a learning PR to track unwrap panics in CLI execution tests and improve parsing boundaries.

- **When to choose it instead:** If there was a specific input/output test panic causing immediate upstream issues.
- **Trade-offs:** We haven't identified critical new parsing panics in the target crates, and the explicit BTreeMap key determinism rule was discovered missing in several places in memory.

## Decision
Choose Option A. Implement explicit `BTreeMap` entry sorting across test and model code as specified by the deterministic sorting rule to lock in ordering.
