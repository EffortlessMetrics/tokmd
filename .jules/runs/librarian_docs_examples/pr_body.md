## 💡 Summary
Added practical usage examples to the `run`, `gate`, `export`, and `diff` CLI commands. These are now integrated directly via `clap` attributes so they display in terminal `--help` and sync to generated documentation.

## 🎯 Why
The `ROADMAP.md` prioritizes adding practical, short examples to key commands (`analyze`, `diff`, `context`, `gate`, `cockpit`, `handoff`, `run`, and `export`) to reduce CLI friction. Adding these to the clap structs natively enforces that terminal help output and the canonical `reference-cli.md` documentation cannot drift from each other.

## 🔎 Evidence
- `cargo run --bin tokmd help <command>` now prominently displays examples.
- `cargo xtask docs --check` verifies that `reference-cli.md` explicitly includes these examples and is up to date.
- Modified: `crates/tokmd/src/cli/parser/gate.rs`, `crates/tokmd/src/cli/parser/run.rs`, `crates/tokmd/src/cli/parser/export.rs`, `crates/tokmd/src/cli/parser/diff.rs`, and generated `docs/reference-cli.md`.

## 🧭 Options considered
### Option A (recommended)
- Append concrete usage examples to the `tokmd <cmd> --help` output via `clap`'s `after_help`.
- Why it fits: Inherently fulfills the `ROADMAP.md` requirement while using the rust structs as the single source of truth, eliminating the risk of drift.
- Trade-offs: Structure / Velocity / Governance: Fixes the DX locally and guarantees alignment between the code and docs.

### Option B
- Add examples manually to the `reference-cli.md` markdown file.
- When to choose it instead: If the CLI tool didn't generate documentation automatically.
- Trade-offs: High risk for drift, breaks the repo contract enforced by `cargo xtask docs --check`.

## ✅ Decision
Option A. It ensures terminal output and generated documentation share a single source of truth, natively closing the gap defined in the ROADMAP.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser/gate.rs`: Added `tokmd gate` help examples.
- `crates/tokmd/src/cli/parser/run.rs`: Added `tokmd run` help examples.
- `crates/tokmd/src/cli/parser/export.rs`: Added `tokmd export` help examples.
- `crates/tokmd/src/cli/parser/diff.rs`: Added `tokmd diff` help examples.
- `docs/reference-cli.md`: Updated via `cargo xtask docs --update`.

## 🧪 Verification receipts
```text
cargo xtask docs --update
cargo xtask docs --check
cargo build --verbose
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Added `#[command(after_help = ...)]` attributes.
- Blast radius: CLI documentation, terminal help output (API/Schema/Compatibility untouched).
- Risk class + why: Lowest risk. Solely adds strings to `--help` output and updates a generated documentation file.
- Rollback: Revert the PR.
- Gates run: `cargo build`, `cargo clippy`, `cargo xtask docs --check`.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
