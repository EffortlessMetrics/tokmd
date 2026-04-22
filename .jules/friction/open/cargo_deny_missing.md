# `cargo-deny` missing in execution environment

The `steward` persona instructions explicitly require running `cargo deny --all-features check` as a fallback check for manifestation modifications. However, `cargo-deny` is not installed in the system/Jules environment (`error: no such command: deny`), which causes the gate checks to error.

To reduce friction in the run pipeline, `cargo-deny` should be added to the environment builder.
