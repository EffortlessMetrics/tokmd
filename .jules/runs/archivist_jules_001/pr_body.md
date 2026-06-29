## 💡 Summary
This is a learning PR. It documents a gap where `.jules/index/generated/RUNS_ROLLUP.md` can drift because `cargo xtask jules-index --check` is not enforced in CI or pre-commit.

## 🎯 Why
Stale indexes defeat the purpose of having rollups for reviewers and LLMs. If `RUNS_ROLLUP.md` is out of date, future agents might hallucinate state or miss recent runs. Consolidating this recurring friction into a documented item ensures future runs can prioritize fixing it.

## 🔎 Evidence
- file path(s): `xtask/src/tasks/jules_index.rs`, `.jules/runbooks/RUN_PACKET.md`
- observed behavior / finding: `xtask jules-index --check` can detect drift, but isn't required when writing run packets.
- command: `cargo run -p xtask --bin xtask -- jules-index --check` (detects drift if a run is added but index is not updated).

## 🧭 Options considered
### Option A (recommended)
- what it is: Create a learning PR that documents this drift issue as a friction item and persona note.
- why it fits this repo and shard: It aligns with "consolidate recurring friction themes" without making risky un-requested code changes to CI logic.
- trade-offs: Structure (low risk), Velocity (high), Governance (better tracking of the gap).

### Option B
- what it is: Modify `.github/workflows/` to add `jules-index --check` to the CI pipeline immediately.
- when to choose it instead: If the prompt explicitly allowed modifying GitHub workflows and we had tested the CI impact thoroughly.
- trade-offs: Higher risk of breaking CI for unrelated PRs if not tested thoroughly.

## ✅ Decision
Option A. I will create a learning PR documenting the current state of `jules-index`, record a friction item about enforcing index freshness in CI or pre-commit, and update the run packet.

## 🧱 Changes made (SRP)
- Created `.jules/runs/archivist_jules_001/envelope.json`
- Created `.jules/runs/archivist_jules_001/decision.md`
- Created `.jules/runs/archivist_jules_001/receipts.jsonl`
- Created `.jules/runs/archivist_jules_001/result.json`
- Created `.jules/runs/archivist_jules_001/pr_body.md`
- Added `.jules/friction/open/FRIC-20231024-001.md`
- Added `.jules/personas/archivist/notes/index-freshness.md`

## 🧪 Verification receipts
```text
{"command": "write_envelope", "status": "success"}
{"command": "write_decision_and_friction", "status": "success"}
{"command": "write_persona_note", "status": "success"}
{"command": "write_result_json", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (documentation and metadata only)
- Risk class + why: Lowest - no code or workflow changes.
- Rollback: Remove the added friction item and persona notes.
- Gates run: `cargo xtask jules-index`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules_001/envelope.json`
- `.jules/runs/archivist_jules_001/decision.md`
- `.jules/runs/archivist_jules_001/receipts.jsonl`
- `.jules/runs/archivist_jules_001/result.json`
- `.jules/runs/archivist_jules_001/pr_body.md`
- Friction: `.jules/friction/open/FRIC-20231024-001.md`
- Notes: `.jules/personas/archivist/notes/index-freshness.md`

## 🔜 Follow-ups
- Implement CI check for `cargo xtask jules-index --check` as tracked in FRIC-20231024-001.
