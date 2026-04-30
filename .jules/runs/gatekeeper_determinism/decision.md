## Options considered
### Option A (recommended)
- Extract inline `sort_by` logic in `crates/tokmd-model/src/lib.rs` into public functions (`sort_lang_rows`, `sort_module_rows`, `sort_file_rows`).
- Replace duplicate sorting closures in integration tests (`tokmd-model/tests`, `tokmd-types/tests`) with calls to these new standalone functions.
- Why it fits this repo and shard: Directly aligns with the Gatekeeper persona's instruction to test sorting determinism using exposed public functions rather than redefining duplicate sorting logic in tests. Resolves a friction point noted in memory.
- Trade-offs: Minor API addition to `tokmd-model`.

### Option B
- Add a macro or internal test-only function to sort rows in tests.
- When to choose it instead: If exposing these sorting functions to the public API is deemed a stability risk.
- Trade-offs: Violates the explicit instruction to expose these as public standalone functions in `tokmd-model`.

## Decision
Chose Option A to strictly follow the Gatekeeper protocol and instructions from the memory.
