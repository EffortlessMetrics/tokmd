## Options Considered

### Option A (recommended)
Move the `js` feature of the `uuid` dependency in `crates/tokmd-format/Cargo.toml` to a target-specific dependency block for `wasm32`.
This avoids unconditionally compiling JavaScript transitive dependencies like `wasm-bindgen` and `js-sys` for native targets (like the CLI), while keeping it available for WebAssembly builds.

### Option B
Keep the `js` feature on `uuid` unconditionally.
This compiles unnecessary Wasm integration layers for native targets, increasing compile times and dependencies needlessly.

## Decision
Option A. It correctly isolates Wasm-specific dependencies to Wasm targets, maintaining a cleaner dependency hygiene for the primary native applications in `tokmd`.
