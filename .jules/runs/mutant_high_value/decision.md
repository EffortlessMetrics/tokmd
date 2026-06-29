## 🧭 Options considered

### Option A (recommended)
- **What it is**: Add missing serialization/roundtrip tests to `crates/tokmd-types/src/packet_siblings.rs` covering the `test_targets` and `do_not_touch` fields in `ManualCandidateRecord`, as well as adding property consistency checks for `DiffRow` and `DiffTotals` in `crates/tokmd-types/src/diff.rs`.
- **Why it fits this repo and shard**: These are high-value contract types within the `core-pipeline` shard. Currently, the `manual_candidates_roundtrip` test only populates `id`, `title`, and `invariant`. If a mutation removes or alters the serialization of `test_targets` or `do_not_touch`, the tests pass but the consumer contract breaks. Similarly, `DiffRow` has calculated delta fields that lack consistency assertions.
- **Trade-offs**:
  - *Structure*: Strengthens contract guarantees.
  - *Velocity*: Minimal overhead.
  - *Governance*: Prevents accidental drift in schema contracts.

### Option B
- **What it is**: Implement exhaustive fuzzing or property-based testing for all structs in `tokmd-types`.
- **When to choose it instead**: If the goal was to guarantee no panics across the entire parsing surface rather than closing specific, known assertion gaps on high-value structs.
- **Trade-offs**: Slower to run and implement, potentially out of scope for a targeted mutant fix.

## ✅ Decision
Option A. It directly closes a concrete missed-mutant-style gap where contract fields (`test_targets`, `do_not_touch`) are currently untested in the roundtrip serialization, and bolsters the `DiffRow` consistency which is core to the diff pipeline.
