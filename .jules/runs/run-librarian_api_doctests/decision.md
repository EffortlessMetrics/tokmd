# Decision

## Option A (recommended)
Add doctests directly to `resolve_profile` and `get_profile_name` in `crates/tokmd/src/config.rs`.
- **Why it fits:** The Librarian persona aims to improve executable coverage for common usage, preventing docs/schemas/help text mismatch. Adding these examples explicitly verifies the fallback resolution path.
- **Trade-offs:**
  - **Structure**: Increases documentation surface slightly in `config.rs`.
  - **Velocity**: Fast implementation.
  - **Governance**: Complies with the `docs-executable` gate profile.

## Option B
Do not add doctests to these particular config functions as they are less likely to be called externally compared to the higher-level CLI interfaces.
- **When to choose it instead:** If `tokmd-settings` or higher-level structs were explicitly tested instead.
- **Trade-offs:** Lower confidence in the underlying resolution logic directly from the doc level.

## Decision
Chose Option A to ensure the `get_profile_name` and `resolve_profile` APIs remain demonstrably working, satisfying the Librarian persona's mission to improve missing doctest or example coverage for common usage.
