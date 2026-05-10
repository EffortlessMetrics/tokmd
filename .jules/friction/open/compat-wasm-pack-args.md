---
id: compat-wasm-pack-args
persona: Compat
style: Builder
shard: bindings-targets
status: open
---

`wasm-pack test` does not pass down Cargo flags correctly for features, such as `--features analysis` or `--no-default-features`, if passed blindly. You must pass them accurately after the `--` but many users and workflows get confused by `wasm-pack test --node -- --features=analysis`. This is external tool friction but worth documenting for future runs.
