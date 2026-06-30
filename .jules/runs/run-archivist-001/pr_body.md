## 💡 Summary
Updated the Jules run rollup indexes. This resolves the index drift caused by newly added run packets.

## 🎯 Why
The `cargo xtask jules-index --check` validation failed, indicating that `.jules/index/generated/RUNS_ROLLUP.md` was out of sync with the actual contents of `.jules/runs/`. Regenerating the indexes ensures our run history is accurately reflected in the summary documents.

## 🔎 Evidence
- `.jules/index/generated/RUNS_ROLLUP.md`
- `cargo xtask jules-index --check` output showed: `Error: Jules index drift detected in /app/.jules/index/generated/RUNS_ROLLUP.md. Run cargo xtask jules-index to update.`

## 🧭 Options considered
### Option A (recommended)
- Run `cargo xtask jules-index` to regenerate the `.jules/index/generated/RUNS_ROLLUP.md` and related index files.
- This directly aligns with the Archivist mission to "summarize per-run packets into generated indexes/rollups".
- Trade-offs:
  - Structure: Improves consistency.
  - Velocity: Low effort, high signal fix.
  - Governance: Ensures metadata rollups remain accurate.

### Option B
- Document the drift as a learning PR instead of fixing it.
- Choose this only if the xtask generation was broken or required out-of-scope changes.
- Trade-offs: Leaves known, fixable drift in the repository.

## ✅ Decision
Option A. The index drift was straightforward to fix using the provided `cargo xtask jules-index` tool, maintaining the accuracy of our generated rollups without risking any product code stability.

## 🧱 Changes made (SRP)
- `.jules/index/generated/RUNS_ROLLUP.md`

## 🧪 Verification receipts
```text
cargo xtask jules-index
cargo xtask jules-index --check
```

## 🧭 Telemetry
- Change shape: Metadata regeneration
- Blast radius: None (docs only)
- Risk class: Low - touches only generated `.jules` artifacts
- Rollback: Revert the commit
- Gates run: `cargo xtask jules-index --check`

## 🗂️ .jules artifacts
- `.jules/runs/run-archivist-001/envelope.json`
- `.jules/runs/run-archivist-001/decision.md`
- `.jules/runs/run-archivist-001/receipts.jsonl`
- `.jules/runs/run-archivist-001/result.json`
- `.jules/runs/run-archivist-001/pr_body.md`

## 🔜 Follow-ups
None.
