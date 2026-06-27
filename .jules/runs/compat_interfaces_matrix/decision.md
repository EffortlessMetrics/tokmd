# Decision

## 🧭 Options considered

### Option A (recommended)
- **What it is:** Remove `ast` from the `default` features of the `tokmd` crate.
- **Why it fits this repo and shard:** The memory explicitly states "In the `tokmd` project, the `ast` feature (which pulls in `tree-sitter` and its parsers) requires a C standard library (`stdlib.h`) and breaks standard `wasm32-unknown-unknown` builds. It should not be included in `default` features for crates intended to be WASM compatible." By removing it from `default` in `crates/tokmd/Cargo.toml`, we fix `wasm32-unknown-unknown` compatibility for the main crate under `--all-features` without breaking `x86_64` standard builds. We'll still keep the feature available to explicitly opt-in if someone has the C stdlib on WASM or is compiling for a native target, but it shouldn't be default.
- **Trade-offs:**
  - *Structure:* Better alignment with standard WASM compilation expectations.
  - *Velocity:* High velocity, as it's a simple feature adjustment.
  - *Governance:* Reduces friction for downstream consumers using standard wasm targets.

### Option B
- **What it is:** Provide a proxy WASM shim for `stdlib.h` so that `ast` can compile.
- **When to choose it instead:** If `ast` was deeply essential for all wasm runs and we couldn't just gate it behind a non-default feature.
- **Trade-offs:** Significant complexity, requires maintaining C stubs which is beyond the scope of a simple config fix.

## ✅ Decision
Option A. The `ast` feature explicitly prevents out-of-the-box WASM builds due to `tree-sitter`'s reliance on `stdlib.h`. Removing it from the `default` features list in `tokmd` restores default WASM compatibility.
