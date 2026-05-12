# Decision

## Option A (Update bindings documentation and types)
Update `crates/tokmd-node/src/lib.rs` and `crates/tokmd-python/src/lib.rs` to include the `meta` and `strip_prefix` parameters in their docstrings/signatures for the `export` function, aligning them with the `ExportSettings` defined in `crates/tokmd-settings/src/commands.rs` and parsed in `crates/tokmd-core/src/ffi/settings_parse.rs`. The node JS docstring should document `options.meta` and `options.strip_prefix`. Python signature should add `meta=true, strip_prefix=None` and pass them to JS args.

## Option B (Leave bindings out of sync)
Do nothing, leaving the bindings docstrings and/or signatures incomplete and drifting from the actual underlying Rust implementation.

## Decision
Option A. It perfectly fits the persona's goal to "Reduce drift across interfaces and targets", specifically "Rust core <-> Python/Node drift" and "binding docs/examples/tests out of sync with real behavior".
