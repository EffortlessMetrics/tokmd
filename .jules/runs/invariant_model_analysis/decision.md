# Decision

## Focus
We need to add missing property-based invariants around a model or analysis surface.

## Options Considered

### Option A: Add property-based invariants to `source_complexity.rs`
The `source_complexity.rs` module in `tokmd-analysis` contains a lightweight heuristic Rust code parser (`analyze_rust_function_complexity`) that tracks function complexity across a single source file. This code does not have existing proptest invariants in a `properties.rs` module. We can add invariant checks:
- Reordering the sequence of independent functions does not change the aggregate `total_complexity` or `max_complexity`.
- The aggregate `total_complexity` is always greater than or equal to `max_complexity`.
- Inserting formatting tokens (like spaces/tabs/newlines) does not impact the complexity score of the source.

**Trade-offs:**
- Fits the `analysis-stack` shard and the `property` gate profile.
- Explicitly tests structural and aggregation invariants.

### Option B: Look for property-based tests inside `crates/tokmd-gate`
Another possibility is generating random configurations for gates (e.g. `ratchet.rs` inside `tokmd-gate`). While useful, testing heuristic parsers (Option A) has historically uncovered more edge cases and better fits the 'brittle edge behavior that benefits from generated inputs' prompt.

## Decision
**Option A**. Adding property-based invariants around `source_complexity.rs` directly tests aggregation/heuristic parsing invariants that should remain stable across source code structure variations. It perfectly matches the `property` profile and provides an honest proof improvement in the `analysis-stack` shard.
