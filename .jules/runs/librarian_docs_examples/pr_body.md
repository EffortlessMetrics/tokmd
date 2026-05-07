## 💡 Summary
Updated the `Global Arguments` section in `docs/reference-cli.md` to include `--format`, `--top`, `--files`, and `--children`, matching the output of `tokmd --help` without subcommands.

## 🎯 Why
The factual documentation in `docs/reference-cli.md` was drifting away from the actual CLI outputs. Specifically, the "Global Arguments" section missing several flags such as `--format`, `--top`, `--files`, and `--children`, which are globally available when running `tokmd --help` directly without an explicit subcommand.

## 🔎 Evidence
Minimal proof:
- `docs/reference-cli.md`
- Running `tokmd --help` outputs four extra flags (`--format`, `--top`, `--files`, `--children`) that were missing from the "Global Arguments" documentation table.

## 🧭 Options considered
### Option A (recommended)
- Add the missing flags (`--format`, `--top`, `--files`, and `--children`) into the `docs/reference-cli.md` "Global Arguments" table.
- Fixes factual docs drift aligning perfectly with the Librarian persona's mission.
- Trade-offs: Structure / Velocity / Governance: Extremely low risk, high governance alignment. No runtime risk since only documentation is changed.

### Option B
- Add `baseline` or `gate` CLI command tests in `xtask` instead of fixing docs.
- Choose this if the primary goal is adding new test coverage rather than addressing missing factual documentation.
- Trade-offs: Higher risk of introducing test flakiness or brittleness without necessarily improving factual documentation.

## ✅ Decision
Option A. It directly addresses a known drift gap where `xtask docs --check` cannot help, fulfilling the Librarian persona's mission precisely by fixing a missing factual coverage on common usage in reference drift.

## 🧱 Changes made (SRP)
- `docs/reference-cli.md`: Added `--format`, `--top`, `--files`, and `--children` into the Global Arguments table.

## 🧪 Verification receipts
```text
{"ts_utc": "2026-05-07T11:31:39Z", "phase": "investigation", "cwd": "/app", "cmd": "cargo run -- --help", "status": "success", "summary": "Discovered that --format, --top, --files, and --children are globally available when running without subcommand", "artifacts": []}
{"ts_utc": "2026-05-07T11:31:39Z", "phase": "investigation", "cwd": "/app", "cmd": "cat docs/reference-cli.md", "status": "success", "summary": "Found that Global Arguments section in reference docs was missing these arguments", "artifacts": []}
{"ts_utc": "2026-05-07T11:31:39Z", "phase": "fix", "cwd": "/app", "cmd": "sed", "status": "success", "summary": "Updated docs/reference-cli.md to include the missing arguments in the Global Arguments table", "artifacts": ["docs/reference-cli.md"]}
```

## 🧭 Telemetry
- Change shape: Docs update.
- Blast radius: Only `docs/reference-cli.md` is changed. No APIs, logic, concurrency, schemas, or dependencies were affected.
- Risk class: None. Documentation change only.
- Rollback: `git checkout -- docs/reference-cli.md`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
