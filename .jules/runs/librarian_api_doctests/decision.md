# Decision

## Option A (recommended)
Fix the `tokmd::` doctest imports in `crates/tokmd/src/config.rs` to correctly use `tokmd::config::`.
- **Why it fits:** The doctests are failing to compile when run individually or in some test scenarios because the items in `config.rs` are not re-exported from the `tokmd` crate root in a way that allows `use tokmd::func` inside `config.rs` doctests. The memory clearly states: "When writing doctests in `crates/tokmd/src/config.rs`, imported items must be referenced using their full path `use tokmd::config::<item>;` (e.g., `use tokmd::config::resolve_profile;`), as they are not re-exported at the `tokmd` crate root."
- **Trade-offs:** Fast, zero-risk fix. Directly adheres to the `docs-executable` gate profile by ensuring doctests compile and pass. Improves proof quality.

## Option B
Re-export the items in `crates/tokmd/src/lib.rs` to make `use tokmd::` work.
- **Why it fits:** It makes the imports in the doctests work without modifying the doctests themselves.
- **Trade-offs:** This changes the public API surface of the `tokmd` crate, which is risky and might have unintended side effects. The memory explicitly advises against this.

## Decision
Option A. It's safe, fixes the issue perfectly, and follows the explicit guideline from memory.
