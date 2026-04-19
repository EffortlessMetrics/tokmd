## 💡 Summary
Improved the runtime developer experience by clarifying error messages for misspelled CLI subcommands. Unrecognized commands like `tokmd anlyze` now detect the typo and suggest `analyze` using Jaro-Winkler string similarity, instead of producing a confusing "Path not found" error. Additionally, removed redundant wording in the missing key error for `analyze --explain`.

## 🎯 Why
By default, the CLI uses a positional `[PATH]...` argument. If a user typos a subcommand, clap parses it as a path. The subsequent "Path not found: anlyze" error is confusing and does not point the user towards fixing their typo. This improvement adds context-aware spelling suggestions to the error formatting to guide users to the correct command. Redundant hints on `analyze --explain` were also simplified to reduce output noise.

## 🔎 Evidence
- `crates/tokmd/src/error_hints.rs`
- Running `tokmd anlyze` produces a suggestion for the misspelled command instead of a generic path error.
- Running `tokmd analyze --explain missing-key` produces a single, non-redundant hint.

## 🧭 Options considered
### Option A (recommended)
- Intercept the "Path not found" error in `error_hints.rs` and check if the single-word missing path closely matches an existing subcommand using `strsim::jaro_winkler`.
- **Structure**: Localized to the error formatter; no disruptive architectural changes to the clap definition.
- **Velocity**: Fast implementation using an existing dependency.
- **Governance**: Fits seamlessly within the `tokmd` facade.

### Option B
- Change `clap` configuration (e.g. using `allow_external_subcommands`) to catch unknown subcommands explicitly.
- **Trade-offs**: May complicate the expected positional path behavior and require more pervasive changes across the CLI parsing tier.

## ✅ Decision
Chose Option A to keep parsing robust and localize the DX improvement to the error reporting layer, leveraging `strsim` which was already available.

## 🧱 Changes made (SRP)
- `crates/tokmd/Cargo.toml`: Added `strsim` dependency to `tokmd`.
- `crates/tokmd/src/error_hints.rs`: Implemented spelling suggestions for "path not found" errors that resemble subcommands, and deduped hints for `analyze --explain`.

## 🧪 Verification receipts
```text
$ cargo run -p tokmd -- anlyze
Error: Path not found: anlyze

Hints:
- Did you mean `analyze`?

$ cargo run -p tokmd -- analyze --explain missing-key
Error: Unknown metric/finding key 'missing-key'. Use --explain list to see supported keys.

$ cargo test -p tokmd --lib
test result: ok. 206 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

$ cargo test --test cli_errors_w66
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s

$ cargo test --test cli_error_help_w73
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

## 🧭 Telemetry
- **Change shape**: Runtime DX enhancement.
- **Blast radius**: Limited to error formatting (CLI outputs).
- **Risk class**: Low. Affects error paths only; core execution remains unchanged.
- **Rollback**: Trivial revert of `error_hints.rs` and `Cargo.toml`.
- **Gates run**: `cargo check -p tokmd`, `cargo test -p tokmd --lib`, `cargo test --test cli_errors_w66`, `cargo test --test cli_error_help_w73`

## 🗂️ .jules artifacts
- `.jules/runs/run_palette_interfaces_1/envelope.json`
- `.jules/runs/run_palette_interfaces_1/decision.md`
- `.jules/runs/run_palette_interfaces_1/receipts.jsonl`
- `.jules/runs/run_palette_interfaces_1/result.json`
- `.jules/runs/run_palette_interfaces_1/pr_body.md`

## 🔜 Follow-ups
None.
