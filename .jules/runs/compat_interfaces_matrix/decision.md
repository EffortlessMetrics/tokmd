### Option A
Ensure the `ast` feature is disabled when building for `wasm32-unknown-unknown` because `tree-sitter` parsers rely on `stdlib.h` which isn't present by default in the `wasm32-unknown-unknown` target. The `default` features of `tokmd` currently include `ast`, breaking wasm builds unless `default-features = false` is specified or `ast` is removed from `default` features. Removing it from `default` features in `tokmd` is a clean way to ensure compatibility without needing custom configurations for wasm targets, as per the memory instructions.

### Option B
Provide a custom `cfg` attribute in `Cargo.toml` to disable the `ast` feature for wasm targets automatically.

**Decision: Option A** is aligned with the memory which explicitly states: "In the `tokmd` project, the `ast` feature (which pulls in `tree-sitter` and its parsers) requires a C standard library (`stdlib.h`) and breaks standard `wasm32-unknown-unknown` builds. It should not be included in `default` features for crates intended to be WASM compatible." Option A adheres directly to this constraint.
