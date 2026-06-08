# Decision: Add Doctests to CLI Interface

## Inspected
- `docs/reference-cli.md` and `docs/tutorial.md` (CLI examples).
- `crates/tokmd/tests/docs.rs` (which already verifies markdown CLI recipes using `assert_cmd`).
- `crates/tokmd/src/cli/parser/*.rs` (the actual code defining CLI options).

## Findings
The `tokmd` crate's CLI parser modules (`lang.rs`, `module.rs`, `export.rs`, `analysis.rs`, `cockpit.rs`, `sensor.rs`, `gate.rs`, `context.rs`, and `commands.rs`) lacked executable doctests to prove the argument parsing behaves as expected and documented. This meant the programmatic definition of the CLI was at risk of drifting without immediate, localized proof.

## Options Considered

### Option A (recommended)
- **What it is**: Add executable ````rust` doctests directly to the `clap::Args` struct definitions in `crates/tokmd/src/cli/parser/*.rs`.
- **Why it fits**: The Librarian persona focuses on missing doctest/example coverage for core/config/CLI public APIs. Testing `try_parse_from` ensures that valid CLI invocations actually parse to the expected structs.
- **Trade-offs**: High confidence in CLI parsing, small increase in test compilation time.

### Option B
- **What it is**: Modify `tests/docs.rs` to extract code blocks dynamically from markdown docs.
- **When to choose it instead**: If we want full end-to-end command execution (which `docs.rs` already partly does via hardcoded `tokmd()` calls).
- **Trade-offs**: Slower to run, harder to maintain string-matching in tests.

## Decision
Chosen Option A. Adding doctests directly on the CLI structs is the idiomatic Rust way to prevent API usage drift.
