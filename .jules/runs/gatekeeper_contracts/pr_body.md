## 💡 Summary
Removed manual markdown parameter tables from `docs/reference-cli.md` and replaced them with `<!-- HELP: <command> -->` markers. This delegates documentation generation to `cargo xtask docs`, locks in deterministic CLI usage tracking, and closes the silent-skip case where a missing marker pair could make docs checks pass without checking a command.

The restack also updates the xtask gate regression test to match the current queue policy: Jules provenance under `.jules/**` can be intentional PR state, so gate should not blanket-block `.jules/runs/**` while still guarding actual runtime/cache/transcript directories.

## 🎯 Why
Manual parameter tables in documentation frequently become outdated when CLI arguments change. `tokmd` provides `cargo xtask docs --update` which relies on `<!-- HELP: <command> -->` markers. Several commands were still using hand-maintained markdown tables, meaning updates to CLI args could easily be missed by the xtask check. The final patch also makes missing marker pairs an explicit docs-check failure.

## 🔎 Evidence
- File: `docs/reference-cli.md`
- Observation: Many subcommands like `module`, `export`, `run`, `handoff` did not have `<!-- HELP: <command> -->` markers and used manual `| Argument | Description |` tables.
- Verification: Running `cargo xtask docs --check` before the change ignored drift in these subcommands because they had no markers.

## 🧭 Options considered
### Option A (recommended)
Replace manual parameter tables with `<!-- HELP: <command> -->` markers for all commands, letting `cargo xtask docs --update` populate them correctly from the clap help output. Also fail `cargo xtask docs --check` when an expected marker pair is absent.
- Trade-offs:
  - **Structure**: Eliminates duplication of parameter details.
  - **Velocity**: Developers no longer have to manually edit markdown tables when updating CLI parameters.
  - **Governance**: Fits perfectly within the `tooling-governance` shard.

### Option B
Manually verify and keep parameter tables in `docs/reference-cli.md` in sync by hand.
- When to choose it: Only if custom columns are needed that clap does not output.
- Trade-offs: Extreme risk of drift and maintenance burden.

## ✅ Decision
Option A. The `tokmd` codebase explicitly discourages manually maintaining parameter tables. The final patch keeps all synchronization in `cargo xtask docs --update` / `cargo xtask docs --check` and verifies that every expected command marker exists.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`: Removed manual parameter tables for the CLI command surface and replaced them with `<!-- HELP: <command> -->` markers.
- `xtask/src/tasks/docs.rs`: Treats missing marker pairs as documentation drift instead of silently skipping those commands.
- `xtask/src/tasks/gate.rs`, `xtask/tests/xtask_deep_w74.rs`: Document and test that `.jules/runs/**` is not treated as forbidden runtime state because it may be intentional PR provenance.

## 🧪 Verification receipts
```text
$ cargo xtask docs --update
Updated docs/reference-cli.md

$ cargo xtask docs --check
Documentation is up to date.

$ cargo test -p tokmd --test docs
test result: ok

$ cargo test -p xtask
test result: ok
```

## 🧭 Telemetry
- Change shape: Replacement of manual documentation content with auto-generated sync blocks plus docs-check and provenance-policy guardrails.
- Blast radius: Docs and xtask validation only.
- Risk class: Low - Does not change application runtime behavior.
- Rollback: Revert the commit.
- Gates run: `cargo xtask docs --update`, `cargo xtask docs --check`, `cargo test -p xtask`, `cargo test -p tokmd --test docs`, `cargo fmt-check`, `git diff --check`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None. All CLI references are now correctly managed via xtask.
