# Decision

## 🧭 Options considered

### Option A (recommended)
- Clean up unused dependencies `pyo3-build-config` and `napi-build` across `tokmd-python` and `tokmd-node` respectively. Neither crate appears to have a `build.rs` script, yet they declare a `build-dependencies`. Wait, I just saw in memory that Node.js bindings use `napi_build::setup()` in their `build.rs` so it shouldn't be removed, and python can be safely removed if no `build.rs` exists.
Wait, let's look for structural boundaries.

Let's do Option B.
### Option B
- Wait, I need a true architectural seam improvement. Let me check the boundaries.
`cargo xtask boundaries-check`
