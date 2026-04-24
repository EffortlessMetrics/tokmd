## 💡 Summary
Fixed documentation drift in `docs/reference-cli.md` by inserting missing CLI help markers (`<!-- HELP: <cmd> -->`) for all commands and updated `xtask/src/tasks/docs.rs` to error out explicitly if any marker is missing, preventing silent drift in the future.

## 🎯 Why
The `cargo xtask docs` command was passing successfully and returning "Documentation is up to date." even though many CLI commands (like `module`, `export`, `analyze`, `run`, `baseline`, `badge`, `diff`, `init`, `context`, `handoff`, `check-ignore`, `tools`, and `completions`) were missing their `<!-- HELP: ... -->` markers in `docs/reference-cli.md`. This meant that the command help text in the reference documentation was manually maintained and slowly drifting out of sync with actual behavior.

## 🔎 Evidence
- Running `grep -rn "<!-- HELP:" docs/reference-cli.md` showed markers only existed for `lang`, `cockpit`, `sensor`, and `gate`.
- Running `cargo xtask docs --check` and `cargo xtask docs --update` falsely reported that documentation was up to date.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add missing markers to `docs/reference-cli.md` and enforce marker existence in `xtask/src/tasks/docs.rs`.
- why it fits this repo and shard: It updates the markdown directly while preserving surrounding manually-authored context (tables, examples) and ensures automated toolings effectively enforce the sync.
- trade-offs: Structure / Velocity / Governance: Safely ensures that CLI reference does not silently drift.

### Option B
- what it is: Only update `xtask docs` to fail when markers are missing.
- when to choose it instead: If the priority is just to flag broken state.
- trade-offs: Manual fixes to docs are required to get the CI passing again.

## ✅ Decision
Selected Option A. It correctly backfills the missing blocks into `docs/reference-cli.md` ensuring up-to-date documentation and modifies the verification mechanism to prevent future regressions.

## 🧱 Changes made (SRP)
- Modified `docs/reference-cli.md`: Injected missing CLI help block pairs.
- Modified `xtask/src/tasks/docs.rs`: Enforced presence of `<!-- HELP: ... -->` pairs.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.

$ cargo test -p xtask
...
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.18s
```

## 🧭 Telemetry
- Change shape: Documentation update and tooling fix.
- Blast radius: Only affects documentation and `xtask` documentation gating; does not affect core parsing or execution logic.
- Risk class: Low
- Rollback: Revert the PR.
- Gates run: `cargo xtask docs --check` and `cargo test -p xtask`.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples_builder/envelope.json`
- `.jules/runs/librarian_docs_examples_builder/decision.md`
- `.jules/runs/librarian_docs_examples_builder/receipts.jsonl`
- `.jules/runs/librarian_docs_examples_builder/result.json`
- `.jules/runs/librarian_docs_examples_builder/pr_body.md`

## 🔜 Follow-ups
None.
