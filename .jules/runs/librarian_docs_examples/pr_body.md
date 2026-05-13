## 💡 Summary
Fixed version drift in the README.md GitHub Action example snippet. It now correctly points to the `1.11.0` release.

## 🎯 Why
The `README.md` file had an outdated GitHub Action example using `version: '1.10.0'`, whereas the current repo version and other documentation (like `docs/github-action.md` and `Cargo.toml`) use `1.11.0`.

## 🔎 Evidence
- File: `README.md`
- Finding: The GitHub action example specified `version: '1.10.0'`.
- Command run: `cargo xtask docs --check` verified docs and `cargo test --doc` passed.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update the version in `README.md` to `1.11.0`.
- why it fits this repo and shard: It falls under the `tooling-governance` shard and specifically fixes example drift.
- trade-offs: Structure / Velocity / Governance: Low risk, maintains structural consistency across docs.

### Option B
- what it is: Update the missing `cargo xtask check-docs` command.
- when to choose it instead: If the docs were genuinely broken regarding command names.
- trade-offs: Does not directly address the more obvious version drift.

## ✅ Decision
Option A was chosen because it's a clear, concrete fix to factual example drift.

## 🧱 Changes made (SRP)
- `README.md`: Updated `version: '1.10.0'` to `version: '1.11.0'` in the GitHub Action block.

## 🧪 Verification receipts
```text
cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: docs
- Risk class + why: Low, only updates a markdown file snippet.
- Rollback: Revert the commit.
- Gates run: `cargo xtask docs --check`, `cargo test --doc`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
