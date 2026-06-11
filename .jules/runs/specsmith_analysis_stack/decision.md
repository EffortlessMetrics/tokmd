## 🧭 Options considered

### Option A (recommended)
- **What it is:** The codebase in `crates/tokmd-analysis` already has BDD-style test functions buried within `tests/analysis_deep_w64.rs`. I will extract these tests (and build upon them) into a new, dedicated `tests/bdd.rs` file. This addresses target #1: "missing BDD/integration coverage for an important path", specifically by extracting and centralizing BDD-style tests into an explicit `tests/bdd.rs` file matching the naming convention present elsewhere in the `tokmd` codebase (e.g., in `tokmd-envelope`, `tokmd-cockpit`, `tokmd-git`, etc.).
- **Why it fits this repo and shard:** The current tests in `tokmd-analysis` lack a dedicated `bdd.rs` integration test, causing `cargo test --test bdd` to fail, even though many other crates in the workspace have a dedicated BDD test suite. Additionally, the persona assignment explicitly asks for "BDD/integration coverage for an important path" and states "Prefer behavior-level tests". By pulling out BDD-style tests into `bdd.rs` and writing them with formal "Given/When/Then" structure, we satisfy the assignment and improve scenario coverage.
- **Trade-offs:**
  - **Structure:** Improves organization by separating behavior-driven scenario tests from deeply technical orchestrator unit tests.
  - **Velocity:** Low impact on build times.
  - **Governance:** Aligns `tokmd-analysis` with the standard BDD test suite layout found across other crates.

### Option B
- **What it is:** Refactor internal data setups inside existing `tests/orchestration.rs` and `tests/orchestrator.rs` to add more BDD comments, without moving the tests to a new file.
- **When to choose it instead:** If creating a new integration test entrypoint adds too much compile-time overhead or if the existing orchestrator tests are already purely behavior-oriented.
- **Trade-offs:**
  - It misses the opportunity to provide a standardized `cargo test -p tokmd-analysis --test bdd` entrypoint.
  - It clutters lower-level orchestration tests with high-level behavioral scenarios.

## ✅ Decision
Option A. Centralizing behavior-driven tests into a new `tests/bdd.rs` explicitly targets "missing BDD/integration coverage for an important path". It conforms to the broader workspace's convention of having a `bdd.rs` integration test suite and improves edge-case coverage transparency. I will extract the BDD-style tests currently scattered or commented as BDD from `tests/analysis_deep_w64.rs` and `tests/orchestrator.rs` into `tests/bdd.rs`, ensuring the new file covers empty repo behavior, standard repo behavior, multi-module output, determinism, and basic feature capability.
