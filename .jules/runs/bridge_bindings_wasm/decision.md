## Option A (recommended)
Implement `run_json_bytes` and `runJsonBytes` in the Node.js and Python bindings to mirror the core crate and WebAssembly bindings.

- **Structure**: High alignment. Ensures the same FFI surfaces available in Rust and Wasm are exported correctly in other bindings natively.
- **Velocity**: Fast. Trivial binding glue.
- **Governance**: Bridges a drift between `tokmd-wasm` / `tokmd-core` and the Node.js / Python targets.

## Option B
Do not implement `run_json_bytes` in Node.js and Python.

- **When to choose**: If those runtimes would prefer not to support archive-zip scanning over in-memory buffers due to performance concerns.
- **Trade-offs**: Creates behavioral drift; the FFI API won't match across targets.
