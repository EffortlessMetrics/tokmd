# Options Considered

## Option A (recommended)
Add property tests for `compute_diff_totals` in `crates/tokmd-format/src/diff/compute.rs`.

**Why it fits:** `compute_diff_totals` performs accumulation on an unconstrained sequence of diff rows. A property test validates that invariants like `new - old == delta` hold true on the aggregate struct, and that `fold` sums exactly match map/sum, across randomized data. This aligns directly with the "Mutant" persona's goal to strengthen behavioral proofs for a contract-facing core calculation (the core data/format pipeline).

**Trade-offs:**
*   **Structure:** Enhances the behavioral guarantees of diff reporting by formalizing structural math expectations.
*   **Velocity:** Negligible impact on compilation.
*   **Governance:** Validates exact correctness of the DiffTotals struct.

## Option B
Add property tests for JSON path serialization stability in `crates/tokmd-types/src/evidence_packet.rs`.

**When to choose it instead:** If the primary gap is in the contract boundary with review-packet consumers, checking stable serialization formats and exact data preservation under stress conditions.

**Trade-offs:**
*   **Structure:** Ensures serialized outputs maintain backwards compatibility.
*   **Velocity:** Lower payoff since these DTOs don't carry complex internal calculations.

# Decision
We will go with **Option A**. The mathematical aggregation in `compute_diff_totals` forms the backbone of the diff pipeline's summary capabilities. Ensuring deterministic correctness via property-based testing directly fulfills the gate profile `mutation` expectations around reducing uncertainty in logic.
