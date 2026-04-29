# Option A: Fix CLI help examples in `tokmd/src/cli/parser.rs` and improve `tokmd/src/config.rs` doctests.

The CLI help text has missing examples for many commands, and some examples are poorly formatted. We can fix these to generate better executable examples via the CLI and the markdown generator. At the same time, we can ensure doctests in `tokmd/src/config.rs` and `tokmd-core/src/ffi.rs` match the behavior and remain robust.

# Option B: Add comprehensive doctests to `tokmd-core/src/lib.rs` covering all missing variants.

Expand the existing doctests in `tokmd-core/src/lib.rs` and `tokmd/src/config.rs` with additional cases. Make sure all public API endpoints have explicit, executable coverage.

Decision: Option A is strongly preferred. The prompt emphasizes finding missing executable coverage or clearly misleading omissions. By improving `tokmd/src/cli/parser.rs` clap doc comments, we directly improve the `reference-cli.md` output and user experience. We will add more executable doctests to `tokmd/src/config.rs` for under-tested resolving logic and fix `tokmd-core/src/ffi.rs` edge cases if any.
