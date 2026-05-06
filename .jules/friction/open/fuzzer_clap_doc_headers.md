---
id: fuzzer_clap_doc_headers
persona: Fuzzer
style: Prover
shard: interfaces
status: open
---

# `clap` Docstring Formatting Conflict

## Context
When trying to fix documentation markdown drift for the `cargo xtask docs --check` command by replacing `/// Examples:` with standard `/// # Examples` on `clap` parser structs (specifically `tokmd/src/cli/parser.rs`), the literal `#` character ends up being rendered in the `tokmd --help` command line output.

## Why this is a friction item
There is a conflict between adhering to standard `rustdoc` conventions (which expects `# Examples` to be treated as a header) and the way `clap` derives CLI help strings from these docstrings (which does not strip the `#`). Attempting to "harden" these doc surfaces inadvertently damages the user-facing CLI UX.

## Impact
This limits the ability to use standard markdown headers within `clap` argument documentation and caused a learning PR fallback since the expected patch worsened the output.
