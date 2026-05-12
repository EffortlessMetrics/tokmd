# Decision

## Option A: Add doctests to config resolvers
Add concrete executable doctests to `tokmd`'s CLI config resolvers (`resolve_lang`, `resolve_export`, etc.) in `crates/tokmd/src/config/resolve/`. These are the public bridge between CLI arguments and core functionality, and missing coverage means config behaviour could silently drift.

## Option B: Add doctests to `tokmd_core::workflows`
Add doctests to workflow execution modules. This might duplicate the existing integration tests and focus less on public APIs than the configuration layer.

### Decision
Option A. The `resolve_*` and `resolve_*_with_config` functions are directly responsible for mapping complex user-facing configurations (CLI args + profile + toml) and currently lack execution coverage in `resolve_export.rs` and `resolve_module.rs`. I will add doctests to these resolver methods.
