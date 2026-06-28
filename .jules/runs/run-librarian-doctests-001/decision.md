## 💡 Context
The goal is to improve factual docs quality and executable examples for `tokmd`. The gate profile is `docs-executable`, prioritizing doctests and example tests so documentation does not silently drift. I have identified missing or incomplete doctests in `crates/tokmd-settings/src/config.rs`, `crates/tokmd/src/config.rs`.

## 🧭 Options considered

### Option A (recommended)
- Add complete and compiling doctests for TOML parsing in `crates/tokmd-settings/src/config.rs` (`parse` and `from_file`).
- Add complete and compiling doctests for `load_config` in `crates/tokmd/src/config.rs`.
- Ensure all added code snippets are runnable and pass `cargo test --doc`.

- **Structure**: Directly targets the public API docs and guarantees accuracy using Rust's built-in testing framework.
- **Velocity**: Fast implementation.
- **Governance**: Adheres to `docs-executable` gate profile perfectly by ensuring docs are executable code.

### Option B
- Modify the `docs/reference-cli.md` and `docs/tutorial.md` files to include explicit commands that run.
- Convert snippets in markdown files to integration tests so they don't drift.

- **When to choose**: If the primary source of drift is in the Markdown reference guides.
- **Trade-offs**: Requires setting up `assert_cmd` integration tests which takes more time and may touch areas less closely related to the Rust library surfaces (Core/Settings).

## ✅ Decision
I choose **Option A**. The library surfaces have obvious doctest gaps for the configuration loading logic (which users of the `tokmd-core` or `tokmd` library would need). Adding executable doctests to `crates/tokmd-settings/src/config.rs` and `crates/tokmd/src/config.rs` directly solves this within the `interfaces` shard and perfectly satisfies the `docs-executable` gate.
