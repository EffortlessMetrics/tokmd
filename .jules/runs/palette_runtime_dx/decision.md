## Options considered

### Option A (recommended)
- what it is: Add clap value_parser validation (`super::validate::positive_usize`) to the numeric flags `max_commits` and `max_commit_files` in the `analyze` and `badge` CLI subcommands.
- why it fits this repo and shard: It aligns with the existing validation of these same parameters in the `context` subcommand (found in `crates/tokmd/src/cli/parser/context.rs`), fixing a sharp edge that allowed `0` as an invalid value which would cause silent degenerate behavior or fail ungracefully, aligning perfectly with the Palette persona's runtime DX focus inside the `interfaces` shard.
- trade-offs: Structure (low, reuses existing validation fn) / Velocity (high, clear fix) / Governance (none).

### Option B
- what it is: Let `clap` parse `0` and then handle the error gracefully within the command logic itself (in `run.rs`, `analyze.rs`, etc).
- when to choose it instead: If the value `0` had some valid semantic meaning for those commands that differs from other commands, or if delayed evaluation of the parameter was necessary.
- trade-offs: Increases boilerplate and decreases consistency since we'd duplicate validation logic inside command implementations rather than doing it uniformly in the parser, which `context.rs` already does.

## Decision
I chose Option A because the `super::validate::positive_usize` value parser is already provided specifically for this purpose in `validate.rs` and is used properly by `CliContextArgs` to catch zero values at parse time. Adding it to `CliAnalyzeArgs` and `BadgeArgs` ensures consistent input validation across the CLI, preventing silent failures and delivering a better error message early.
