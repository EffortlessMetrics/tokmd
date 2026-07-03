## 💡 Summary
Fixed example mismatch for `tokmd handoff` between CLI help and the reference documentation. The `--no-git` example was present in `docs/handoff.md` and the `reference-cli.md` examples block, but missing from the actual CLI `after_help` output.

## 🎯 Why
To resolve factual docs drift and ensure the CLI help examples remain aligned with the official documentation and roadmap requirements.

## 🔎 Evidence
- `crates/tokmd/src/cli/parser/context.rs`: Missing `--no-git` example in `HandoffArgs` `after_help`.
- `docs/reference-cli.md` and `docs/handoff.md`: Contain the `--no-git` example.
- Mismatch between `tokmd handoff --help` stdout and `docs/reference-cli.md` when running `cargo xtask docs --check`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Fix the `--help` example for `tokmd handoff` in `crates/tokmd/src/cli/parser/context.rs` by adding the missing `tokmd handoff --no-git` example, and update `docs/reference-cli.md` using `cargo xtask docs --update`.
- why it fits this repo and shard: Meets the librarian shard mandate to resolve missing docs or examples for common usage.
- trade-offs: Structure: Low risk, localized change / Velocity: Fast / Governance: Complies with the single PR policy and anti-drift docs sync.

### Option B
- what it is: Instead of adding the missing CLI help example, rewrite the `docs/reference-cli.md` manually to remove the `--no-git` example, keeping it in sync with the CLI help string.
- when to choose it instead: If the `--no-git` flag is deprecated or considered uncommon/undesirable usage.
- trade-offs: This is worse because `--no-git` is explicitly listed in `docs/ROADMAP.md` as a valid scenario and the `handoff` command natively supports disabling git enrichment.

## ✅ Decision
Option A. The missing CLI help example was restored in `crates/tokmd/src/cli/parser/context.rs` to align the CLI with the documentation.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/cli/parser/context.rs`: Added `tokmd handoff --no-git` to the `HandoffArgs` `after_help` examples.
- `docs/reference-cli.md`: Re-rendered reference documentation using `cargo xtask docs --update`.

## 🧪 Verification receipts
```text
cargo run -p xtask docs --update
cargo run -p xtask docs --check
```

## 🧭 Telemetry
- Change shape: Docs/CLI help example update.
- Blast radius: Docs only.
- Risk class: Low - documentation only change.
- Rollback: Revert the PR.
- Gates run: `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
