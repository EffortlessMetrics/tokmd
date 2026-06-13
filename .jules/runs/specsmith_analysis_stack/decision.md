# Decision

## Option A (recommended)
Move the BDD scenarios that validate the core behavior of `tokmd-analysis` from `tests/analysis_deep_w64.rs` into a dedicated `tests/bdd.rs` file.

- **Why it fits:** The workspace convention establishes that behavior-driven scenario tests (Given/When/Then) should reside in dedicated `bdd.rs` files rather than being mixed into generic structural unit test files (like `analysis_deep_w64.rs`). This matches what is already done in `tokmd-gate`, `tokmd-envelope`, `tokmd-cockpit`, etc.
- **Trade-offs:**
  - Structure: +1, aligns `tokmd-analysis` with the standard workspace architecture.
  - Velocity: 0, minor file movement.
  - Governance: 0, standard test reorganization.

## Option B
Do nothing or add new tests to `analysis_deep_w64.rs`.

- **When to choose it instead:** If there was no strong workspace convention around `bdd.rs`.
- **Trade-offs:** Retains drift in testing structure across crates.

## Decision
**Option A.** Creating `tests/bdd.rs` inside `tokmd-analysis` resolves an architectural drift (mixed concerns in generic test files) and brings the crate into compliance with workspace conventions.
