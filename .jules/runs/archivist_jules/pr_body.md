## 💡 Summary
Regenerated the Jules run index to clean up stale run entries. This ensures `.jules/index/generated/RUNS_ROLLUP.md` accurately tracks existing runs and does not display ghost items for runs that have been deleted.

## 🎯 Why
The Archivist persona is responsible for summarizing run packets into generated indexes. Because runs like `auditor_bindings_manifests` and `compat_interfaces_matrix_01` were removed from `.jules/runs/`, the generated indexes needed an update to stay aligned with the actual file system state.

## 🔎 Evidence
- file: `.jules/index/generated/RUNS_ROLLUP.md`
- observed behavior before: Stale entries for `auditor_bindings_manifests` and `compat_interfaces_matrix_01` were present.
- command receipt: `cargo xtask jules-index` regenerates with accurate metadata.

## 🧭 Options considered
### Option A (recommended)
- what it is: Run `cargo xtask jules-index`.
- why it fits this repo and shard: It directly satisfies Archivist target #2 (summarize into generated indexes/rollups) in the `workspace-wide` shard.
- trade-offs: Structure: High. Governance: High. Velocity: Neutral.

### Option B
- what it is: Only regenerate the indexes without fixing metadata.
- when to choose it instead: N/A
- trade-offs: We would leave ghost entries in the generated docs.

## ✅ Decision
Option A. It's an honest patch that directly improves the Jules indexing health by fixing the root cause of the stale rows.

## 🧱 Changes made (SRP)
- Ran `cargo xtask jules-index` to update `.jules/index/generated/RUNS_ROLLUP.md`.

## 🧪 Verification receipts
```text
{"ts_utc": "2024-05-08T20:55:00Z", "phase": "investigation", "cwd": "/app", "cmd": "cat .jules/index/generated/RUNS_ROLLUP.md", "status": 0, "summary": "Found that the rollup had stale entries for auditor_bindings_manifests and compat_interfaces_matrix_01."}
{"ts_utc": "2024-05-08T20:57:00Z", "phase": "implementation", "cwd": "/app", "cmd": "cargo xtask jules-index", "status": 0, "summary": "Regenerated the indexes, which removed stale metadata in RUNS_ROLLUP.md"}
```

## 🧭 Telemetry
- Change shape: Documentation and metadata indexing
- Blast radius: Jules documentation / scaffolding
- Risk class: Low
- Rollback: `git restore .jules/index/generated/`
- Gates run: `cargo xtask publish --plan --verbose`, `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
