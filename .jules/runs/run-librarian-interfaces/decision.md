# Decision

## Option A: Add doctests for missing ConfigContext methods
The `ConfigContext` methods like `get_toml_view` and `get_json_profile` are missing doctests. We can add straightforward executable examples to them.

## Option B: Fix legacy / incomplete doctests in `config.rs` and `resolve/*.rs`
The existing doctests in `config.rs` and `resolve/*.rs` import `tokmd_settings::Profile` or similar types, but there's a disconnect.
Wait, let me review `crates/tokmd/src/config.rs` missing doctests.

The missing doctest coverage is heavily localized in `crates/tokmd/src/config.rs` for `ConfigContext` methods:
- `get_toml_view`
- `get_json_profile`
- `load_config`

And `ResolvedConfig` methods:
- `format`
- `top`
- `files`
- `module_roots`
- `module_depth`
- `children`
- `min_code`
- `max_rows`
- `redact`
- `meta`

These missing doctests violate the `docs-executable` gate expectation of this shard, because the docs are currently lacking executable examples. I will add doctests for them.

Wait, looking at `ResolvedConfig`, `ConfigContext` and its methods, they form the core config resolution APIs.
I'll add doctests to these methods.
