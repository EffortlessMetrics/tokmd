# Option A: Conditionally Compile Git Interactions
Use `#[cfg(feature = "git")]` in commands like `baseline`, `check_ignore`, and `handoff` when calling out to `tokmd_git`. When the `git` feature is not enabled, use fallback logic (e.g. returning `None`, `false`, or assuming Git is not available) so that `cargo build --no-default-features` or missing the `git` feature compiles successfully.

# Option B: Always Require Git Feature
Force the `git` feature to always be enabled by modifying `Cargo.toml`. This prevents the compatibility issue but defeats the purpose of the feature flags which allow the crate to be built minimally without `tokmd_git` dependencies.

## Decision
I will proceed with Option A because it correctly addresses the `--no-default-features` compilation failure while respecting the crate's modular feature flags. This is the recommended approach for Rust compatibility across feature matrices.
