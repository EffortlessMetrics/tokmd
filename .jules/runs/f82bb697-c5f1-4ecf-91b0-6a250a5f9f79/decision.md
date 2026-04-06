# Option A: Expand Doctests for Context, Gate, and Handoff Subcommands

The CLI reference documentation (`docs/reference-cli.md`) and tutorial indicate usage for `tokmd context`, `tokmd gate`, and `tokmd handoff` commands, but `crates/tokmd/tests/docs.rs` lacks coverage for these commands (except a partial `recipe_handoff_bundle` and `recipe_gate_with_baseline`). There's drift/omissions around `context` usage. We can add tests to `tests/docs.rs` mirroring the CLI reference documentation examples.

# Option B: Add API Doctests for Core Structures

In `crates/tokmd-core/src/lib.rs`, there's a lack of module-level or struct-level doctests demonstrating how to invoke the programmatic API using `tokmd_core` rather than the CLI. We could add `#[doc = "```"]` examples to key public structures in `crates/tokmd-core/src/lib.rs` and `crates/tokmd-config/src/lib.rs`.

**Selection**: Option A. Option A strictly follows the "Docs as tests" principle explicitly mandated in the agent instructions (`crates/tokmd/tests/docs.rs` where CLI recipes recommended in the README and tutorials are verified) and addresses the concrete missing example coverage for common usage. This ensures recipes don't silently drift.
