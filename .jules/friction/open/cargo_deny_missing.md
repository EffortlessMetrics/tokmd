# Friction Item: cargo-deny missing

The tool `cargo-deny` was required by the gate profile `deps-hygiene` (specifically `cargo deny --all-features check`), but it was not installed in the environment. Treating as friction instead of a blocker per runbook.
