# Decision

## Option A
Create a hallucinated "determinism fix" by sorting BTreeMap output explicitly in `tokmd-analysis-content`.
**Why it fits:** Does not fit. Violates the core performance prompt because it adds O(N log N) work, altering existing sorted-key behavior incorrectly, and hallucinates a fix without an honest performance story.

## Option B (recommended)
Acknowledge the lack of a clear, verifiable hot-path performance win inside the `analysis-stack` shard in the current short exploration window, and generate a **learning PR** instead. Document the friction that identifying allocations/repeated work requires better benchmarks or deeper structural analysis.
**Why it fits:** Follows the hard constraint: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."

## ✅ Decision
Option B. Without a stable benchmark harness readily identifiable in `crates/tokmd-analysis-content` for allocations, attempting a blind structural "performance" patch is a hallucinated fake fix. We will document this friction.
