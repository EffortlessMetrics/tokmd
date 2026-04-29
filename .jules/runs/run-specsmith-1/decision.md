# Decision

## Option A (recommended)
Fix the byte-indexing panic in `halstead::tokenize_for_halstead` when processing CJK (or multi-byte) characters. Currently, the multi-char operator logic slices `&remaining[..len]`, where `len` is a char count but is passed as a byte index. We will change the logic to slice at the correct char boundaries by taking advantage of `char_indices()`.
We'll also add a behavior-level test to `crates/tokmd-analysis/src/halstead/mod.rs` to prove we handle multi-byte characters safely during analysis without crashing.

- Fits the repo and shard because this is an edge-case regression in analysis behavior involving halstead tokenization, matching the Specsmith persona's goal (edge-case polish around analysis behavior).
- Trade-offs:
  - Structure: Improves robustness of the string tokenization logic using correct char indices.
  - Velocity: Small fix with immediate impact on avoiding panics when analyzing codebases containing non-ASCII symbols next to operators.

## Option B
Instead of changing the slice logic, we simply truncate non-ASCII characters earlier during tokenization or restrict the halstead analyzer to only ASCII streams.
- When to choose it instead: If the halstead analyzer didn't care at all about correct string preservation and we just wanted the fastest way to drop all CJK text.
- Trade-offs: Degrades overall analysis accuracy for files containing comments or strings mixed with multi-byte chars; goes against the design of supporting modern repos.

## Decision
Choosing **Option A**. Slicing correctly along character boundaries via `char_indices()` is the correct, Rust-first way to fix the panic and maintains correct token counting without losing any data.
