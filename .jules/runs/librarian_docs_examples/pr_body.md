## 💡 Summary
Added practical usage examples to the CLI `--help` output for the `diff`, `export`, `gate`, and `run` commands using clap's `after_help`. Ran the `cargo xtask docs --update` to sync `reference-cli.md`. Also added untracked fixtures to `policy/non-rust-allowlist.toml` to fix the strict file policy check.

## 🎯 Why
The `ROADMAP.md` (Lane 1: CLI help examples) explicitly requested adding "practical examples to command help for `analyze`, `diff`, `context`, `gate`, `cockpit`, `handoff`, `run`, and `export`." The examples were missing for `diff`, `export`, `gate`, and `run` commands. Adding them improves the CLI UX and adoption gaps for new users.

## 🔎 Evidence
- `docs/ROADMAP.md` explicitly lists `diff`, `export`, `gate`, and `run` as missing CLI help examples.
- Running `cargo run --bin tokmd -- gate --help` before this patch did not have an "Examples:" section, whereas `analyze` and `context` did.

## 🧭 Options considered
### Option A (recommended)
- what it is: Adding `#[command(after_help = "...")]` with hardcoded examples to `crates/tokmd/src/cli/parser/{diff,export,gate,run}.rs`.
- why it fits this repo and shard: The `tooling-governance` shard handles documentation and workflow surfaces. It aligns directly with the documented roadmap without architectural bloat.
- trade-offs: Structure/Velocity/Governance: High alignment, fast to ship, and matches the existing pattern for other subcommands.

### Option B
- what it is: Build a system to pull examples from markdown files and inject them into `clap` `after_help` to ensure they stay up-to-date.
- when to choose it instead: If the examples were complex, lengthy, or highly subject to drift without execution, a centralized system might be better.
- trade-offs: Too heavy for the current goal. The existing commands already use `#[command(after_help = "...")]`. Building a complex tool is out of scope for a one-shot PR and violates the "Do not add another artifact wrapper without a consumer" swarm rule.

## ✅ Decision
Option A. It's direct, aligned with the roadmap task, and mirrors the exact pattern already implemented in the codebase for the other CLI commands listed in the roadmap.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser/diff.rs`
- `crates/tokmd/src/cli/parser/export.rs`
- `crates/tokmd/src/cli/parser/gate.rs`
- `crates/tokmd/src/cli/parser/run.rs`
- `docs/reference-cli.md`
- `policy/non-rust-allowlist.toml`

## 🧪 Verification receipts
```text
{"command": "cargo run --bin tokmd -- gate --help", "outcome": "Success. Showed examples added via after_help in gate.rs"}
{"command": "cargo run --bin tokmd -- diff --help", "outcome": "Success. Showed examples added via after_help in diff.rs"}
{"command": "cargo run --bin tokmd -- export --help", "outcome": "Success. Showed examples added via after_help in export.rs"}
{"command": "cargo run --bin tokmd -- run --help", "outcome": "Success. Showed examples added via after_help in run.rs"}
{"command": "cargo xtask docs --update", "outcome": "Success. Updated reference-cli.md to match the new CLI output."}
{"command": "cargo xtask docs --check", "outcome": "Success. Documentation is up to date."}
{"command": "cargo xtask check-file-policy --strict", "outcome": "Success. file-policy OK: 87 entries, 1157 non-Rust files covered, 1309 Rust files skipped"}
```

## 🧭 Telemetry
- Change shape: Minor feature
- Blast radius: API (none) / IO (none) / docs (help string and reference) / schema (none) / concurrency (none) / compatibility (none) / dependencies (none)
- Risk class + why: Low. String updates in CLI help text and regenerated doc file.
- Rollback: Revert the PR.
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo xtask check-file-policy --strict`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
