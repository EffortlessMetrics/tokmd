## 💡 Summary
Attempted to fix factual documentation drift in `docs/reference-cli.md` by updating the `Global Arguments` section to include missing flags. The fix was superseded by a merged PR #1720, so this is a learning PR documenting the collision.

## 🎯 Why
The factual documentation in `docs/reference-cli.md` was drifting away from the actual CLI outputs. Specifically, the "Global Arguments" section missing several flags such as `--format`, `--top`, `--files`, and `--children`, which are globally available when running `tokmd --help` directly without an explicit subcommand. Since this was already merged in #1720, I am abandoning the fix and creating a learning PR instead.

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
Option A was chosen to fix the docs drift. However, after making the changes, a maintainer comment indicated that this fix was superseded by PR #1720. Therefore, I am abandoning the change and creating a learning PR to document the workflow collision, adhering to the memory directive.

## 🧱 Changes made (SRP)
- None. (Reverted `docs/reference-cli.md` due to collision with PR #1720).

## 🧪 Verification receipts
```text
{"ts_utc": "2026-05-07T11:31:39Z", "phase": "investigation", "cwd": "/app", "cmd": "cargo run -- --help", "status": "success", "summary": "Discovered that --format, --top, --files, and --children are globally available when running without subcommand", "artifacts": []}
{"ts_utc": "2026-05-07T11:31:39Z", "phase": "investigation", "cwd": "/app", "cmd": "cat docs/reference-cli.md", "status": "success", "summary": "Found that Global Arguments section in reference docs was missing these arguments", "artifacts": []}
{"ts_utc": "2026-05-07T11:31:39Z", "phase": "fix", "cwd": "/app", "cmd": "sed", "status": "success", "summary": "Updated docs/reference-cli.md to include the missing arguments in the Global Arguments table", "artifacts": ["docs/reference-cli.md"]}
{"ts_utc": "2026-05-07T11:34:00Z", "phase": "rollback", "cwd": "/app", "cmd": "git checkout -- docs/reference-cli.md", "status": "success", "summary": "Reverted docs/reference-cli.md changes due to PR collision with #1720", "artifacts": []}
```

## 🧭 Telemetry
- Change shape: Learning PR.
- Blast radius: None.
- Risk class: None.
- Rollback: None needed.
- Gates run: None.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`

## 🔜 Follow-ups
None.
