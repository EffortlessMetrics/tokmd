## Option A

Expose the `classify_blast` function in `crates/tokmd-analysis-effort/src/delta.rs` as `pub` and add a new proptest to `crates/tokmd-analysis-effort/tests/proptest_models.rs` that verifies its classification invariants over a wide range of input values (including negative and large positive values).

**Why it fits:**
- Tightens property-based tests around real invariants (the `classify_blast` logic).
- Aligns perfectly with the Invariant persona's goal of adding missing invariant coverage in model/analysis surfaces.
- Small, focused, and explicitly satisfies the property gate profile.

**Trade-offs:**
- Makes a previously private function public, slightly expanding the crate's internal API surface (though only within the `delta` module).

## Option B

Extract `classify_blast` into a more generic, isolated module or trait, and test it there without modifying visibility in `delta.rs`.

**Why it fits:**
- Keeps `delta.rs` interface completely unchanged while gaining the test coverage.

**Trade-offs:**
- Involves unnecessary refactoring for a simple mapping function, reducing velocity and increasing the risk of structural churn.

## Decision

**Option A**. It directly satisfies the goal of tightening invariant coverage on an analysis model surface with minimal structural disruption. Exposing the function to `pub` within the module scope is an acceptable trade-off to enable property testing.
