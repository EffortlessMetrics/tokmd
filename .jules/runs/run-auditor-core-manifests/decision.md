# Decision

## Problem
In `crates/tokmd-format/Cargo.toml`, the `uuid` dependency is configured with the `js` feature:
`uuid = { version = "1.22", features = ["v4", "js"] }`

Memory notes state:
> In Rust native applications or CLIs (like those in `tokmd`), avoid enabling the `js` feature on the `uuid` dependency, as it introduces unnecessary WebAssembly/JavaScript transitive dependencies (`wasm-bindgen`, `js-sys`) to the native compile surface.

This is a manifest hygiene issue directly within the target shard (`core-pipeline`) since `tokmd-format` is part of it. The `js` feature brings in WASM-specific bindings (often indirectly through `getrandom` or similar) that have no place in a native compilation artifact unless compiling to wasm.

## Options considered
### Option A (recommended)
- **What it is:** Remove the `js` feature from the `uuid` dependency in `crates/tokmd-format/Cargo.toml`.
- **Why it fits:** It is a high-signal dependency hygiene improvement. It removes redundant/unnecessary feature surface area without breaking existing native behavior. The `js` feature in `uuid` is only needed when generating v4 UUIDs inside a WebAssembly environment (like the browser).
- **Trade-offs:**
  - Structure: Better isolation, prevents JS-specific dependencies from polluting the native build graph.
  - Velocity: Negligible change to compilation time.
  - Governance: Lowers total attack/dependency surface area.

### Option B
- **What it is:** Leave it as is and look for another dependency to remove.
- **When to choose it instead:** If `tokmd-format` is specifically compiled to WASM and relies on JS functionality. But the instructions specify native applications/CLIs should avoid it, and `tokmd-format` doesn't exclusively target WASM. Furthermore, `tokmd-wasm` handles Wasm.

## Decision
I will choose **Option A**. The memory constraint specifically highlights this exact pattern to avoid, and it fits perfectly under "tightening feature flags to reduce compile surface" which is target ranking 3 for the Auditor persona.
