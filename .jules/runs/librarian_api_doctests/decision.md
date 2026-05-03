# Decision

## Inspected
- `crates/tokmd/src/cli/parser.rs` documentation comments.
- `crates/tokmd-core/src/lib.rs` workflow function doctests.

## Option A
Replace `/// Example:` with `/// # Examples` in `crates/tokmd/src/cli/parser.rs` to fix potential clap formatting drift.

## Option B
Add executable doctest code blocks to `crates/tokmd-core/src/lib.rs` for `lang_workflow`, `export_workflow`, `module_workflow`, `diff_workflow`, and `analyze_workflow`.

## Decision
Selected Option B. Adding actual executable doctests to the core public APIs ensures that the library facade documentation cannot silently drift from the actual implementation. Option A was rejected during code review because it was found to cause clap formatting drift in the generated markdown reference.
