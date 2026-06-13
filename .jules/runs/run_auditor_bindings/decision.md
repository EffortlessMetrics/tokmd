# Decision

## Option A: Remove explicit `rt-multi-thread` feature from `tokio` in `tokmd-node`
- What it is: Remove `features = ["rt-multi-thread"]` from `tokio` in `crates/tokmd-node/Cargo.toml`.
- Why it fits this repo and shard: As seen in memory notes, "In `napi-rs` bindings (e.g., `tokmd-node`), the `napi` dependency's `async` feature automatically pulls in `tokio` with the `rt-multi-thread` feature via `tokio_rt`. Explicitly requesting `features = ["rt-multi-thread"]` on a direct `tokio` dependency is redundant and can be safely tightened to `tokio = "1"`." This fits the Auditor persona's mission to remove duplicate/redundant dependency declarations.
- Trade-offs: Structure/Velocity/Governance - Reduces redundancy without breaking behavior, as the feature is still pulled in transitively. Simplifies `Cargo.toml`.

## Option B: Remove `tempfile` dev-dependency from `tokmd-node`
- What it is: Check if `tempfile` is actually used in `tokmd-node` tests. If not, remove it.
- When to choose it instead: If the `rt-multi-thread` removal is blocked or if `tempfile` is verifiably unused.
- Trade-offs: Requires scanning tests.

## Decision
Choose Option A. It's explicitly listed in the memory as a known redundancy in this codebase ("In `napi-rs` bindings (e.g., `tokmd-node`), the `napi` dependency's `async` feature automatically pulls in `tokio`..."). I will modify `crates/tokmd-node/Cargo.toml` to change `tokio = { version = "1", features = ["rt-multi-thread"] }` to `tokio = "1"`, test the change, and check `cargo deny`.
