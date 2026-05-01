Option A: Optimize `normalize_path` in `tokmd-model` by reducing redundant string operations.
- Fits the repo/shard because `normalize_path` is a hot-path function executed on every file collected by `tokmd`.
- Trade-offs: Difficult to achieve substantial wins without restructuring allocations, which broke tests/benchmarks on initial attempts. Small slice tweaks resulted in minimal (<2%) gains while risking double-allocations on edge cases.

Option B: Record a learning PR explaining that a significant, coherent performance improvement inside the `core-pipeline` shard could not be safely and honestly justified within this bounded prompt.
- When to choose: When attempts to optimize hot paths result in negligible measured gains or risk behavioral regressions.
- Trade-offs: No code patch delivered, but adheres strictly to the rule: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."

Chosen Option B because it avoids hallucinating metrics and introducing unstable logic for microscopic gains.
