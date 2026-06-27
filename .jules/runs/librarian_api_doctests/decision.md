# Decision

## Problem
The `tokmd` documentation (`docs/reference-cli.md`) provides examples for the `context`, `analyze`, and `gate` commands. However, the exact output format, options, and behaviors of these commands are subject to change. Executable doctests and examples will ensure the CLI documentation accurately reflects the application's actual behavior and fails tests if drift occurs.

## Options Considered

### Option A: Create integration tests based on CLI documentation examples (Recommended)
Extract the examples from `docs/reference-cli.md` and convert them into a new integration test suite (`crates/tokmd/tests/cli_docs_examples.rs`). This validates the exact commands shown in the documentation using `assert_cmd`, ensuring that the `context`, `analyze`, and `gate` examples are actually executable and succeed.
- **Structure**: Adds a new test file strictly focused on validating documentation examples.
- **Velocity**: High, as it leverages existing test infrastructure (`assert_cmd`).
- **Governance**: Prevents future CLI flag changes from breaking documented examples without failing the test suite.

### Option B: Transform the markdown file into a Rust doctest directly
Attempt to parse and run the markdown file using `rustdoc` or a custom markdown test runner.
- **When to choose**: When the primary goal is executing every single code block in every markdown file.
- **Trade-offs**: High complexity. Rust's built-in `doctest` is primarily designed for Rust code blocks, not generic bash scripts. Custom runners require significant new infrastructure.

## Decision
**Option A** is chosen. It directly addresses the "missing executable coverage" target for documentation examples by writing dedicated integration tests that mirror the CLI examples exactly.
