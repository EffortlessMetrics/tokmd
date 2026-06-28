## 💡 Summary
Added executable doctests for configuration parsing logic in `tokmd-settings` and config loading logic in `tokmd`.

## 🎯 Why
Configuration surfaces lacked compiling and testable documentation. By adding these doctests, we ensure the public API examples are guaranteed to run and do not silently drift from the actual code structure.

## 🔎 Evidence
- File: `crates/tokmd-settings/src/config.rs` (missing doctests for `TomlConfig::parse` and `TomlConfig::from_file`)
- File: `crates/tokmd/src/config.rs` (missing doctests for `load_config()`)
- Proof: Running `cargo test --doc` confirms these examples compile and execute.

## 🧭 Options considered
### Option A (recommended)
- Add complete and compiling doctests for TOML parsing in `crates/tokmd-settings/src/config.rs` (`parse` and `from_file`).
- Add complete and compiling doctests for `load_config` in `crates/tokmd/src/config.rs`.
- Ensure all added code snippets are runnable and pass `cargo test --doc`.

- trade-offs:
  - Structure: Directly targets the public API docs and guarantees accuracy using Rust's built-in testing framework.
  - Velocity: Fast implementation.
  - Governance: Adheres to `docs-executable` gate profile perfectly by ensuring docs are executable code.

### Option B
- Modify the `docs/reference-cli.md` and `docs/tutorial.md` files to include explicit commands that run.
- Convert snippets in markdown files to integration tests so they don't drift.

- when to choose it instead: If the primary source of drift is in the Markdown reference guides.
- trade-offs: Requires setting up `assert_cmd` integration tests which takes more time and may touch areas less closely related to the Rust library surfaces (Core/Settings).

## ✅ Decision
I choose Option A. The library surfaces have obvious doctest gaps for the configuration loading logic (which users of the `tokmd-core` or `tokmd` library would need). Adding executable doctests to `crates/tokmd-settings/src/config.rs` and `crates/tokmd/src/config.rs` directly solves this within the `interfaces` shard and perfectly satisfies the `docs-executable` gate.

## 🧱 Changes made (SRP)
- `crates/tokmd-settings/src/config.rs`: Added executable doctests for `TomlConfig::parse` and `TomlConfig::from_file`.
- `crates/tokmd/src/config.rs`: Added an executable doctest for `load_config()`.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-settings --doc
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo test -p tokmd --doc
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: New doctests
- Blast radius: Docs only
- Risk class: None
- Rollback: Revert docstring additions
- Gates run: `docs-executable` expectations (`cargo test --doc`, `cargo xtask docs --check`, `cargo fmt`, `cargo clippy`)

## 🗂️ .jules artifacts
- `.jules/runs/run-librarian-doctests-001/envelope.json`
- `.jules/runs/run-librarian-doctests-001/decision.md`
- `.jules/runs/run-librarian-doctests-001/receipts.jsonl`
- `.jules/runs/run-librarian-doctests-001/result.json`
- `.jules/runs/run-librarian-doctests-001/pr_body.md`

## 🔜 Follow-ups
None
