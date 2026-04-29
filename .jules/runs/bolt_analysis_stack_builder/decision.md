# Decision

## Option A (recommended)
- what it is: Modify `build_top_offenders` in `crates/tokmd-analysis/src/derived/mod.rs` to iterate and sort references (`&FileStatRow`) instead of cloning the entire vector multiple times and sorting owned clones.
- why it fits this repo and shard: It targets "unnecessary allocations / cloning / string building" and "hot-path work reduction" which aligns exactly with the Bolt ⚡ persona guidelines for the `analysis-stack` shard.
- trade-offs: Structure is slightly more verbose due to explicitly cloning at the very end when taking the top N results, but it removes full-vector clones in a hot loop without changing behavior.

## Option B
- what it is: Build a custom sorting structure.
- when to choose it instead: If the overhead of multiple vectors (even of references) was too high.
- trade-offs: Complex and unidiomatic.

## Decision
Choosing Option A because it is straightforward, achieves exactly what is requested by eliminating unnecessary allocations in an intermediate buffer, and preserves output determinism entirely.
