## 💡 Summary
This is a learning PR. The intended patch to fix the `tokmd handoff` parameter drift in `docs/reference-cli.md` and add corresponding executable tests in `crates/tokmd/tests/docs.rs` was superseded by PR #1592, which merged identical coverage on current main.

## 🎯 Why
The workflow encountered an edge case where the intended fix was superseded by another merged PR (#1592) during execution. Instead of forcing a redundant or fake fix, the process gracefully aborted and created this learning PR to document the workflow edge case as friction.

## 🔎 Evidence
- Pull request comment: "Superseded by #1592, which merged the current executable docs and handoff example coverage on current main."

## 🧭 Options considered
### Option A (recommended)
- what it is: Abort the current fix and fall back to creating a 'learning PR' containing the full per-run packet and recording a friction item.
- why it fits this repo and shard: Directly adheres to the instruction for handling superseded patches in the `tooling-governance` shard.
- trade-offs: Structure / Velocity / Governance: Prioritizes velocity by gracefully acknowledging superseded work rather than fighting merge conflicts or forcing redundant commits.

### Option B
- what it is: Continue trying to push the redundant fix.
- when to choose it instead: Never, if the work is verifiably superseded by main.
- trade-offs: Increases review burden and risks introducing merge conflicts or duplicate tests.

## ✅ Decision
Option A was chosen to properly document the workflow edge case without introducing a redundant fix.

## 🧱 Changes made (SRP)
- None (Learning PR)

## 🧪 Verification receipts
```text
N/A - Work superseded by #1592
```

## 🧭 Telemetry
- Change shape: None (Learning PR).
- Blast radius: None. (API / IO / docs / schema / concurrency / compatibility / dependencies)
- Risk class: Low, no runtime or docs modified.
- Rollback: None.
- Gates run: None.

## 🗂️ .jules artifacts
- `.jules/runs/librarian_docs_examples/envelope.json`
- `.jules/runs/librarian_docs_examples/decision.md`
- `.jules/runs/librarian_docs_examples/receipts.jsonl`
- `.jules/runs/librarian_docs_examples/result.json`
- `.jules/runs/librarian_docs_examples/pr_body.md`
- `.jules/friction/open/superseded_by_1592.md`

## 🔜 Follow-ups
See friction item `.jules/friction/open/superseded_by_1592.md`
