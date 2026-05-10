# Decision

## Option A (recommended)
- **What it is**: Update the property test `cli_parser_never_panics_on_subcommand_with_arbitrary_args` in `crates/tokmd/tests/cli_parser_properties.rs` to include all currently supported subcommands defined in `tokmd::cli::parser::Commands`.
- **Why it fits this repo and shard**: The assignment specifically requests improving scenario coverage and regression testing within the `interfaces` shard (CLI interfaces). The existing CLI parser property test covers a limited subset of subcommands (only `lang`, `module`, `export`, `diff`, `version`, `analyze`, `cockpit`, `context`, `handoff`). It misses important commands like `run`, `check-ignore`, `tools`, `gate`, `baseline`, `badge`, `init`, `completions`, and `sensor`. By including all defined subcommands, we harden input handling across the complete CLI surface and ensure invariant preservation without parser panics.
- **Trade-offs**:
  - *Structure*: Increases test coverage of edge cases without altering production code structures.
  - *Velocity*: Slightly increases test execution time for this specific property test, but execution time is already fast (< 4s).
  - *Governance*: Secures the fuzzing and property boundaries against regressions.

## Option B
- **What it is**: Identify a specific edge case in the parsing logic for one of the missing commands and write a targeted unit test instead.
- **Why it fits this repo and shard**: While targeted tests are good, they do not guarantee that the CLI parser won't panic on other unexpected arbitrary inputs across all subcommands.
- **Trade-offs**:
  - Misses the opportunity to leverage property-based testing to verify broad invariance.
  - Fails to comprehensively lock down the invariant (parser never panics on arbitrary string arguments) across the entire CLI surface.

## Decision
I have chosen **Option A**. Updating the property test provides high-signal coverage and strictly follows the "Specsmith" mission of improving regression coverage and edge-case polish in the CLI interfaces, expanding deterministic input hardening (no panics) across all currently defined subcommands.
