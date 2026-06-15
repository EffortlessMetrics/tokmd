# Decision

## Option A (recommended)
Fix WASM compatibility for the `tokmd` crate by decoupling the `ast` feature (which pulls in `tree-sitter` and breaks standard `wasm32-unknown-unknown` due to missing `stdlib.h`) from the default features, or by removing it entirely if it shouldn't be a default.

According to memory:
> In the `tokmd` project, the `ast` feature (which pulls in `tree-sitter` and its parsers) requires a C standard library (`stdlib.h`) and breaks standard `wasm32-unknown-unknown` builds. It should not be included in `default` features for crates intended to be WASM compatible.

This perfectly fits the Compat persona's goal of fixing compatibility issues across features and targets.

## Option B
Find an issue with `--no-default-features` on another crate.

## ✅ Decision
Option A. The `ast` feature in `tokmd` currently pulls in C-dependencies that break the build on `wasm32-unknown-unknown`. It should be removed from the `default` features of `tokmd`, as `tokmd` and its WASM runner rely on default feature setups.
