# Auditor Persona Note

When reviewing dependency flags or unneeded direct dependencies in core crates, ensure that features like `js` or `wasm-bindgen` are not blindly removed from the root `[dependencies]` array without substituting them in a target-specific `[target.'cfg(...)'.dependencies]` array if the workspace supports cross-compilation to WebAssembly. A blanket removal breaks the Wasm build surface.
