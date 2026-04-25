## Option A
Remove `pyo3-build-config` from `crates/tokmd-python/Cargo.toml`. Since there is no `build.rs` file in `crates/tokmd-python`, this build dependency is completely unused and can be safely removed. This tightens the build surface and removes a redundant dependency.

## Option B
Remove `js-sys` from `crates/tokmd-wasm/Cargo.toml`. `js-sys` is only used once in the source code but is also brought in transitively. This requires more investigation and could potentially break wasm-bindgen features.

## Decision
Choose Option A. It's a straightforward, evidence-backed removal of an unused build dependency (`pyo3-build-config`) in the Python bindings manifest since there's no `build.rs` file to use it.
