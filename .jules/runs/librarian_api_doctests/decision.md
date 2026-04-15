Option A: Add missing executable doctests to public CLI config resolution functions in `crates/tokmd/src/config.rs` (e.g. `get_profile_name`, `resolve_profile`, `get_toml_view`, `get_json_profile`).
- What it is: Expanding the doctest coverage for Tier 5 configuration APIs to ensure examples compile and demonstrate expected profile fallback behaviors.
- Why it fits: Directly addresses the assigned "missing doctest or example coverage for common usage" for public interfaces within the `interfaces` shard.
- Trade-offs: Structure is improved through tighter guarantees, Velocity is slightly reduced by added test maintenance, Governance is strengthened via deterministic documentation.

Option B: Add missing executable doctests to public FFI boundary functions in `crates/tokmd-core/src/ffi.rs` (e.g. `version`, `schema_version`, error constructors).
- What it is: Expanding doctest coverage for Tier 0-3 core FFI functions.
- When to choose: If the FFI boundary is more heavily utilized or lacks basic assertions compared to the CLI resolution logic.
- Trade-offs: Provides coverage on raw FFI but might require more complex mocking or setup for internal states compared to pure config functions.

Decision: Option A
I will focus on `crates/tokmd/src/config.rs` as it acts as a primary interface mechanism for configuration merging and CLI parsing, and currently has clear gaps in doctest coverage for methods like `get_profile_name` and `resolve_profile`. Following memory guidelines, imported items in `config.rs` doctests will be referenced using their full path `use tokmd::config::<item>;`.
