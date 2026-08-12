# Decision

## Option A (recommended)
Fix strict JSON object validation drift in FFI bindings.
- what it is: Modify `tokmd-wasm`, `tokmd-python`, and `tokmd-node` to strictly validate that the top-level parsed JSON value is an object before passing it to the core rust code.
- why it fits this repo and shard: This aligns the bindings with `tokmd-core` which strictly expects top-level json objects and fails fast otherwise. It eliminates a drift in edge-case validation behavior.
- trade-offs: Structure / Velocity / Governance: Improves structure by normalizing validation across all language boundaries at the minor cost of duplicate parsing in some bindings.

## Option B
Do not validate in the bindings and rely solely on `tokmd-core` error propagation.
- what it is: Let the bindings pass raw strings and rely entirely on `tokmd_core::ffi::run_json` to return validation errors.
- when to choose it instead: If performance is the absolute highest priority and parsing JSON twice is unacceptable.
- trade-offs: Prevents early failure, especially in Python where failing fast while holding the GIL is preferred to raise native exceptions properly.
