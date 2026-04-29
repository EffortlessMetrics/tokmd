## 💡 Summary
Improved `tokmd`'s CLI and configuration public API documentation. Fixed `/// Example:` and `/// Examples:` directives in clap arg structs to correctly render in Rustdoc as `# Examples`. Also expanded `tokmd/src/config.rs` with new executable doctests for configuration resolvers to prevent silent documentation drift.

## 🎯 Why
Missing or malformed doctests fail the gate expectation for executable docs (`docs-executable` gate profile). The prompt highlighted missing executable coverage for common usage on core/config/CLI public APIs. By standardizing the `# Examples` headers, we get better Rustdoc coverage, and adding missing doctests directly hardens our public API behavior.

## 🔎 Evidence
- Found `Example:` instead of `# Examples` in `crates/tokmd/src/cli/parser.rs`, causing rustdoc to not treat them as explicit headers, although clap displayed them correctly.
- Found `get_profile_name` and `resolve_profile` in `crates/tokmd/src/config.rs` completely lacking doctests.
- `cargo test --doc` execution proved these gaps could be filled.

## 🧭 Options considered
### Option A (recommended)
- Fix rustdoc headers and add the missing doctest cases directly to `crates/tokmd/src/config.rs`.
- Why it fits: Directly satisfies the `Librarian` mission of improving factual docs quality and executable examples within the `interfaces` shard.
- Trade-offs: Minor doc diff, but provides solid compile-time guarantees against drift.

### Option B
- Add comprehensive doctests to `crates/tokmd-core/src/lib.rs` covering all missing variants.
- When to choose: If the core API itself was under-documented.
- Trade-offs: `lib.rs` is already well-covered; `config.rs` had obvious gaps that could silently drift.

## ✅ Decision
Option A. It's the most direct and aligned fix for missing executable examples for CLI interfaces and config API resolvers.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser.rs`: Updated `/// Examples:` to `/// # Examples` on `GlobalArgs`, `CliLangArgs`, `CliModuleArgs`, `CliExportArgs` for correct rustdoc parsing.
- `crates/tokmd-core/src/ffi.rs`: Standardized `# Example` to `# Examples`.
- `crates/tokmd-core/src/lib.rs`: Standardized `# Example` to `# Examples`.
- `crates/tokmd/src/config.rs`: Added executable doctests for `get_profile_name` and `resolve_profile`.
- `crates/tokmd/tests/snapshots/cli_snapshot_golden__help.snap`: Updated snapshot tests to reflect the updated documentation string.

## 🧪 Verification receipts
```text
cargo test -p tokmd -p tokmd-core -p tokmd-settings --doc
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
all doctests ran in 0.99s; merged doctests compilation took 0.95s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s

cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Documentation and Doctest updates
- Blast radius: Docs only
- Risk class: Low
- Rollback: Revert doc changes
- Gates run: `cargo xtask docs --check`, `cargo test --doc`, `cargo test cli_snapshot_golden`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_api_doctests_run/envelope.json`
- `.jules/runs/librarian_api_doctests_run/decision.md`
- `.jules/runs/librarian_api_doctests_run/receipts.jsonl`
- `.jules/runs/librarian_api_doctests_run/result.json`
- `.jules/runs/librarian_api_doctests_run/pr_body.md`

## 🔜 Follow-ups
None.
