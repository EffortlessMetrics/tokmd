## 💡 Summary
Added missing `<!-- HELP: command -->` markers to `docs/reference-cli.md` for 13 commands, and removed manually maintained redundant usage tables. This allows `cargo xtask docs` to automatically keep the documentation in sync with the CLI definition and prevents silent drift.

## 🎯 Why
The `docs/reference-cli.md` file is meant to be kept in sync with the actual CLI output using `cargo xtask docs`. However, 13 commands (including `context`, `analyze`, `handoff`, etc.) were missing their `<!-- HELP: ... -->` markers. Because the markers were missing, `cargo xtask docs --check` would not report an error when the CLI changed, leading to silent documentation drift. Furthermore, even if the markers were added, leaving the old manual parameter tables creates duplicated and conflicting information.

## 🔎 Evidence
- File path: `docs/reference-cli.md`
- Observed behavior: `xtask/src/tasks/docs.rs` looks for `<!-- HELP: <command> -->` markers to check for drift. Grepping for `<!-- HELP:` in `docs/reference-cli.md` showed only 4 commands had markers (`lang`, `cockpit`, `sensor`, `gate`), while 13 other commands documented in the file did not. The manual tables were out of date (e.g. referencing `[FLAGS]`).

## 🧭 Options considered
### Option A (recommended)
- Add the missing `<!-- HELP: command -->` markers to `docs/reference-cli.md`, run `cargo xtask docs --update` to populate them, and remove the redundant manually-typed parameter tables immediately following them.
- **Why it fits**: This exactly aligns with the Librarian persona's focus on anti-drift and keeping documentation aligned with actual behavior. By adding the markers, we enable the existing `xtask docs` tooling to prevent silent drift for these commands. Removing the redundant tables makes the doc cleaner.
- **Trade-offs**:
    - Structure: High. Plugs gaps in the existing doc-generation pipeline.
    - Velocity: High. A simple script can find the missing commands and add the tags.
    - Governance: High. Ensures that future CLI changes will automatically fail the `cargo xtask docs --check` gate if docs aren't updated.

### Option B
- Manually copy-paste the CLI output into `docs/reference-cli.md` without tags.
- **When to choose**: Never. This defeats the purpose of the `xtask docs` tooling and guarantees future drift.
- **Trade-offs**: Bad for structure and governance.

## ✅ Decision
Option A. It's the structurally correct fix that uses the project's existing governance tooling to permanently prevent drift for these CLI commands, while keeping the documentation clean.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`: Added missing `<!-- HELP: <cmd> -->` markers for `module`, `export`, `run`, `analyze`, `baseline`, `badge`, `diff`, `init`, `context`, `handoff`, `check-ignore`, `tools`, and `completions`, and ran `cargo xtask docs --update` to populate them. Removed the redundant, manually-typed Markdown tables for those commands.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Docs only. No runtime impact.
- Risk class: Low risk. Fixes documentation drift tooling.
- Rollback: Revert the commit.
- Gates run: `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples_1/envelope.json`
- `.jules/runs/librarian_docs_examples_1/decision.md`
- `.jules/runs/librarian_docs_examples_1/receipts.jsonl`
- `.jules/runs/librarian_docs_examples_1/result.json`
- `.jules/runs/librarian_docs_examples_1/pr_body.md`

## 🔜 Follow-ups
None.
