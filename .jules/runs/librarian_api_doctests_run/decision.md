## Options Considered

### Option A: Improve doctests for configuration resolution functions
Add executable doctest coverage to `get_profile_name` and `resolve_profile` in `crates/tokmd/src/config.rs`. These are public functions in the core configuration API that lack proper executable examples to verify behavior.

* **Pros:** Directly addresses the Librarian persona goal of improving executable example coverage for common APIs. Prevents silent API drift by tying doc examples to real code compilation and execution.
* **Cons:** Small footprint, but high value for stability.

### Option B: Document broader private load functions
Add doctests or docs to the private functions `discover_toml_config` and `try_load_toml`.

* **Pros:** Improves internal developer documentation.
* **Cons:** These are private implementation details and not public APIs. Doctests on private functions are generally less valuable than on public API surfaces because they aren't exposed in the crate documentation.

## Decision
**Option A**. The assignment specifies "Focus: Prefer executable docs and doctests for core/config/CLI public APIs." `get_profile_name` and `resolve_profile` are public functions exposed in `tokmd::config` that lack executable doctest coverage. Adding doctests ensures these examples do not drift from the actual implementation.
