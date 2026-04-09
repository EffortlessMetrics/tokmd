# Decision: Create a learning PR detailing friction.

## Problem
The assigned shard is `bindings-targets` (`crates/tokmd-wasm`, `crates/tokmd-node`, `crates/tokmd-python`, `web/runner`).
However, testing `--no-default-features` on the workspace revealed a failure in `crates/tokmd` (which is outside the assigned shard). `crates/tokmd` incorrectly imports `tokmd_git::git_cmd` without `#[cfg(feature = "git")]` checks.

The prompt requires:
"If the strongest target you find is outside the shard, record it as friction instead of chasing it. If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix. ... Do not touch: unrelated interface ergonomics or docs-only rewrites"

Wait, is there any actual issue I can patch in `bindings-targets`?
I did notice `crates/tokmd-python/Cargo.toml` has `test = false` already applied as per `AGENTS.md` rules.
`tokmd-wasm` compiles to `wasm32-unknown-unknown` without issues, and its test suite passes both with `--no-default-features` and `--all-features`.
`tokmd-node` compiles and tests pass with `--no-default-features` and `--all-features`.
`web/runner` passes its tests (`npm run test`).
`tokmd-python` passes its tests with `--no-default-features` and `--all-features`.
The only problem I've discovered is `tokio` complaining about wasm in `tokmd-node`, but `tokmd-node` is a Node addon which relies on N-API and isn't meant for `wasm32-unknown-unknown`. Similarly, `pyo3` in `tokmd-python` does not support wasm out-of-the-box in this configuration and isn't intended to.

Given the constraints, I must create a learning PR documenting the `--no-default-features` compilation failure in `crates/tokmd` as a friction item, and avoid touching files outside the `bindings-targets` shard.
