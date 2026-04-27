# Decision

## Option A: Remove unused dependency in `crates/tokmd-node`
The `serde` dependency is declared in `crates/tokmd-node/Cargo.toml` but is only used as `serde::Serializer` in one place: `Err(serde::ser::Error::custom("boom"))`. This might not actually require the `serde` crate if it can use an alternate error creation, but it is used. `tempfile` is used in tests. No blatantly unused dependencies in bindings according to machete.

## Option B: Tighten/remove redundant feature flags or dependencies in bindings.
In `crates/tokmd-node/Cargo.toml`:
`napi = { version = "3", features = ["async", "serde-json", "tokio_rt"] }`
`serde_json` and `tokio_rt` seem necessary because we use async fns and passing JSON back and forth.
Wait, `crates/tokmd-python` build dependency: `pyo3-build-config = "0.28.3"`. According to pyo3 docs, `pyo3-build-config` is usually an internal crate or used in `build.rs` if there is one. The `crates/tokmd-python` does *not* have a `build.rs`!
Let's verify if `pyo3-build-config` is used.

## Decision: Option B (Remove unused `pyo3-build-config` build-dependency)
The `cargo-machete` output does not check build-dependencies reliably if there is no `build.rs`, or it might miss it. Wait, the memory states: "The `cargo-machete` tool may not reliably flag unused `[build-dependencies]`. To verify if a build dependency is truly unused, manually check for the presence of a `build.rs` file in the crate; if `build.rs` is absent, the build dependencies are unused and safe to remove."
I ran `ls crates/tokmd-python/build.rs` and it failed. `crates/tokmd-python` does NOT have a `build.rs`. Therefore, `pyo3-build-config` is a useless `[build-dependencies]` and can be safely removed.
