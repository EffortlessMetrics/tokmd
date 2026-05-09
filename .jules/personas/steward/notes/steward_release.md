# Steward Notes: Release Hygiene

The `tokmd` repository contains an extremely hygienic setup for tracking and ensuring valid release parameters. Specifically:
1. `cargo xtask version-consistency` handles version consistency and parity seamlessly.
2. The publish plan DAG generation via `cargo xtask publish --plan --verbose` validates dependencies nicely.
3. Documentation references match implementations (verified via `cargo xtask docs --check` and `xtask` unit tests).

This means that any prompt that strictly expects to find version drift may be operating on outdated assumptions of the repository state or requires a more nuanced approach towards discovering non-obvious mismatch risks.
