## 💡 Summary
This is a learning PR. The initial attempt to regenerate the Jules index rollups was superseded by PR #1651. Furthermore, an initial mistake overwrote a historical 2023 provenance packet (`archivist_jules`). I have reverted the index updates, restored the overwritten historical files to preserve provenance, and created a friction item to track the workflow collision risk.

## 🎯 Why
The Archivist persona is responsible for consolidating run packets into generated indexes/rollups. During execution, it was noted that PR #1651 already updated the indexes on `main`. Continuing with the local index regeneration would cause unnecessary merge conflicts. Additionally, Jules strict provenance rules require preserving historical packets ("Never rewrite history"). Therefore, following the Jules runbook, I pivoted to a learning PR in a completely new run packet directory.

## 🔎 Evidence
- file path: `.jules/friction/open/index_generation_collision.md`
- finding: Concurrent agent index generation causes PR collisions and potential provenance loss if directories are reused incorrectly.
- command: Received instruction: "Superseded by #1651, which regenerated the current Jules rollup indexes on current main"

## 🧭 Options considered
### Option A (recommended)
- what it is: Pivot to a Learning PR in a new packet directory, discard the index changes, restore the overwritten historical packet, and log a friction item.
- why it fits this repo and shard: Avoids merge conflicts on `main` and strictly adheres to both the Jules learning PR rule for superseded work and the archival rule against rewriting history.
- trade-offs: Structure: High (preserves `main` integrity and historical provenance). Velocity: Fast. Governance: Follows provenance rules.

### Option B
- what it is: Push the index regeneration anyway or leave the historical packet overwritten.
- when to choose it instead: Never.
- trade-offs: High risk of merge conflicts, overriding upstream changes, and destroying critical audit trails.

## ✅ Decision
Option A was chosen. I reverted the changes to `.jules/index/generated/RUNS_ROLLUP.md`, restored `.jules/runs/archivist_jules/`, created a friction item, and initialized a fresh packet (`archivist_index_learning`) to log this outcome.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/index_generation_collision.md`.
- Created `.jules/runs/archivist_index_learning/envelope.json`.
- Created `.jules/runs/archivist_index_learning/decision.md`.
- Created `.jules/runs/archivist_index_learning/receipts.jsonl`.
- Created `.jules/runs/archivist_index_learning/result.json`.
- Created `.jules/runs/archivist_index_learning/pr_body.md`.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.54s
    Running `target/debug/xtask docs --check`
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: Jules internal friction tracking
- Risk class: None
- Rollback: `git restore .jules/friction/open/index_generation_collision.md`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_index_learning/envelope.json`
- `.jules/runs/archivist_index_learning/decision.md`
- `.jules/runs/archivist_index_learning/receipts.jsonl`
- `.jules/runs/archivist_index_learning/result.json`
- `.jules/runs/archivist_index_learning/pr_body.md`
- `.jules/friction/open/index_generation_collision.md`

## 🔜 Follow-ups
Determine if `build_index.py` should be run via a centralized CI action.
