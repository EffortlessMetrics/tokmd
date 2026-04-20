## 💡 Summary
Removed manual markdown parameter tables from `docs/reference-cli.md` and replaced them with `<!-- HELP: <command> -->` markers. This delegates documentation generation to `cargo xtask docs`, locking in deterministic CLI usage tracking and closing a vulnerability to documentation drift.

## 🎯 Why
Manual parameter tables in documentation frequently become outdated when CLI arguments change. `tokmd` provides `cargo xtask docs --update` which relies on `<!-- HELP: <command> -->` markers. Several commands were still using hand-maintained markdown tables, meaning updates to CLI args could easily be missed by the xtask check, causing drift between the tool behavior and reference documentation.

## 🔎 Evidence
- File: `docs/reference-cli.md`
- Observation: Many subcommands like `module`, `export`, `run`, `handoff` did not have `<!-- HELP: <command> -->` markers and used manual `| Argument | Description |` tables.
- Verification: Running `cargo xtask docs --check` before the change ignored drift in these subcommands because they had no markers.

## 🧭 Options considered
### Option A (recommended)
Replace manual parameter tables with `<!-- HELP: <command> -->` markers for all commands, letting `cargo xtask docs --update` populate them correctly from the clap AST output.
- Trade-offs:
  - **Structure**: Eliminates duplication of parameter details.
  - **Velocity**: Developers no longer have to manually edit markdown tables when updating CLI parameters.
  - **Governance**: Fits perfectly within the `tooling-governance` shard.

### Option B
Manually verify and keep parameter tables in `docs/reference-cli.md` in sync by hand.
- When to choose it: Only if custom columns are needed that clap does not output.
- Trade-offs: Extreme risk of drift and maintenance burden.

## ✅ Decision
Option A. The `tokmd` codebase explicitly discourages manually maintaining parameter tables. We used a script to replace the remaining manual markdown tables with the appropriate `<!-- HELP: <command> -->` markers, and verified the output with `cargo xtask docs --check`. This effectively fixes a structural contract drift issue.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`: Removed manual parameter tables for 13 subcommands (`module`, `export`, `run`, `analyze`, `baseline`, `badge`, `diff`, `init`, `context`, `handoff`, `check-ignore`, `tools`, `completions`) and replaced them with `<!-- HELP: <command> -->` markers.

## 🧪 Verification receipts
```text
$ cargo xtask docs --update
Documentation is up to date.

$ cargo xtask docs --check
Documentation is up to date.

$ cargo test -p tokmd --test docs
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
```

## 🧭 Telemetry
- Change shape: Replacement of manual documentation content with auto-generated sync blocks.
- Blast radius: Docs only.
- Risk class: Low - Does not change application runtime behavior.
- Rollback: Revert the commit.
- Gates run: `cargo xtask docs --check`, `cargo test -p tokmd --test docs`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None. All CLI references are now correctly managed via xtask.
