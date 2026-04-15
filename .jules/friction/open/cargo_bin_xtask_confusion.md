# Cargo `run --bin xtask` Confusion

**Component**: Workspace / Cargo Tools
**Severity**: Low
**Description**:
Running `cargo run --bin xtask` at the workspace root fails with `error: no bin target named xtask in default-run packages`. This occurs because the `tokmd` crate is designated as the `default-run` target for the workspace, and Cargo searches its `[[bin]]` definitions instead of resolving the `xtask` package binary.

**Workaround**:
Use the `.cargo/config.toml` alias: `cargo xtask` (which maps to `cargo run -p xtask --`), or explicitly specify the package: `cargo run -p xtask --bin xtask`.
