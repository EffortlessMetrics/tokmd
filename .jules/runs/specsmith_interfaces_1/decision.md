# Specsmith Path Exploration

## Option A: Migrate unguarded `unwrap`s in core and config to `expect`
There are multiple uses of `.unwrap()` in string parsing or conversion logic inside `crates/tokmd-core/src/lib.rs` and `crates/tokmd-config/src/lib.rs`.
- `crates/tokmd-core/src/lib.rs` (e.g. `parse_analysis_preset(input).unwrap()`)
- `crates/tokmd-core/src/ffi.rs` (e.g. `serde_json::from_str(&result).unwrap()`)
- `crates/tokmd-config/src/lib.rs` (e.g. `serde_json::to_string(&variant).unwrap()`)

While some of these are inside doctests (line 46), others appear to be inline static compilation or inside tight code segments. Replacing these with `expect` with highly descriptive invariants (as per the Sentinel memory rule: "replace `.unwrap()` with `.expect()` in Rust, always provide a highly descriptive message explaining the invariant") provides better guarantees and panics safely when the invariant doesn't hold.

## Option B: Fix unguarded `unwrap` usages specifically affecting CLI args and settings parsing in `crates/tokmd-config`
Focus solely on the parsing implementations in `tokmd-config` and fix `unwrap()` usages for invariants that are always true (like `to_string` for an enum without data variants or `from_str` matching the generated string) to `expect` to improve debugging and document invariants.

## Decision
I'll go with Option A and apply `.expect` with highly descriptive error messages to the unwraps in `tokmd-core/src/lib.rs` and `tokmd-config/src/lib.rs`. This aligns with the "edge-case regression not locked in by tests" target and the specific memory rule around replacing `.unwrap()` with `.expect()`.
