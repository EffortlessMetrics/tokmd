## 💡 Summary
Added executable Rust doctests to the `clap::Args` and `Subcommand` structs in `crates/tokmd/src/cli/parser/*.rs`. This ensures that CLI arguments parse correctly to their respective structures, closing a proof gap in the CLI public API documentation.

## 🎯 Why
The CLI interface definitions (like `tokmd lang`, `tokmd export`, etc.) lacked localized, executable doctests. This poses a risk of the API drifting from how it is documented to be used. Adding doctests provides immediate, testable proof of correctness directly alongside the structures.

## 🔎 Evidence
Minimal proof:
- file paths: `crates/tokmd/src/cli/parser/*.rs`
- observed behavior: Doctests were previously absent from these structures.
- receipt: `cargo test --doc -p tokmd` passes with 21 total doctests.

## 🧭 Options considered
### Option A (recommended)
- What it is: Add executable ````rust` doctests directly to the `clap::Args` struct definitions in `crates/tokmd/src/cli/parser/*.rs`.
- Why it fits this repo and this shard: The Librarian persona focuses on missing doctest/example coverage for core/config/CLI public APIs. Testing `try_parse_from` ensures that valid CLI invocations actually parse to the expected structs.
- Trade-offs: High confidence in CLI parsing, small increase in test compilation time.

### Option B
- What it is: Modify `tests/docs.rs` to extract code blocks dynamically from markdown docs.
- When to choose it instead: If we want full end-to-end command execution (which `docs.rs` already partly does via hardcoded `tokmd()` calls).
- Trade-offs: Slower to run, harder to maintain string-matching in tests.

## ✅ Decision
Chosen Option A. Adding doctests directly on the CLI structs is the idiomatic Rust way to prevent API usage drift.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser/analysis.rs`: Added doctest for `CliAnalyzeArgs`.
- `crates/tokmd/src/cli/parser/cockpit.rs`: Added doctest for `CockpitArgs`.
- `crates/tokmd/src/cli/parser/commands.rs`: Added doctest for `Commands`.
- `crates/tokmd/src/cli/parser/context.rs`: Added doctest for `CliContextArgs`.
- `crates/tokmd/src/cli/parser/export.rs`: Added doctest for `CliExportArgs`.
- `crates/tokmd/src/cli/parser/gate.rs`: Added doctest for `CliGateArgs`.
- `crates/tokmd/src/cli/parser/lang.rs`: Added doctest for `CliLangArgs`.
- `crates/tokmd/src/cli/parser/module.rs`: Added doctest for `CliModuleArgs`.
- `crates/tokmd/src/cli/parser/sensor.rs`: Added doctest for `SensorArgs`.

## 🧪 Verification receipts
```text
$ cargo test --doc -p tokmd

running 21 tests
...
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## 🧭 Telemetry
- Change shape: Additions of `/// ```rust` comment blocks.
- Blast radius: docs
- Risk class: Low (Documentation/Test only)
- Rollback: `git reset --hard HEAD`
- Gates run: `cargo test --doc -p tokmd`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests/envelope.json`
- `.jules/runs/librarian_api_doctests/decision.md`
- `.jules/runs/librarian_api_doctests/receipts.jsonl`
- `.jules/runs/librarian_api_doctests/result.json`
- `.jules/runs/librarian_api_doctests/pr_body.md`

## 🔜 Follow-ups
None.
