## 💡 Summary
This is a learning PR. I ran `cargo xtask jules-index` to consolidate per-run packets and friction items into generated rollups. I also generated a new friction item, as I noticed the current parser in `xtask` misses accurately parsing some older friction items summaries.

## 🎯 Why
The target ranking for the Archivist persona includes summarizing per-run packets into generated indexes/rollups. Generating these rollups provides a consolidated view of runs and friction, addressing target #2 in the mission. I also added a friction item indicating drift between how `xtask` parses friction items, and how some were historically written.

## 🔎 Evidence
- file path(s): `.jules/index/generated/RUNS_ROLLUP.md`, `.jules/index/generated/FRICTION_ROLLUP.md`
- observed behavior / finding: The rollups were successfully generated using the xtask, but I noticed the friction items parser is brittle.
- command: `cargo xtask jules-index`

## 🧭 Options considered
### Option A (recommended)
- what it is: Run `cargo xtask jules-index` to regenerate rollups and add a friction item capturing the parser limitations.
- why it fits this repo and shard: It directly addresses target #2 for the Archivist persona in the workspace-wide shard, and adding the friction item satisfies the learning PR requirements.
- trade-offs: Structure / Velocity / Governance: Improves structure by providing consolidated indexes and capturing friction. No negative impact on velocity or governance.

### Option B
- what it is: Consolidate recurring friction themes into better templates/policy/docs without generating rollups.
- when to choose it instead: When there's a strong recurring theme that isn't already handled.
- trade-offs: More subjective and time-consuming, skips the immediate benefit of index generation.

## ✅ Decision
Option A was chosen to provide an immediate, concrete improvement to the workspace-wide Jules scaffolding by generating the latest indexes and cleanly recording a learning via a new friction item.

## 🧱 Changes made (SRP)
- Updated `.jules/index/generated/RUNS_ROLLUP.md`
- Updated `.jules/index/generated/FRICTION_ROLLUP.md`
- Added `.jules/friction/open/cargo_mutants_schema_drift.md`

## 🧪 Verification receipts
```text
cargo xtask jules-index
Jules indexes written under /app/.jules/index/generated
```

## 🧭 Telemetry
- Change shape: Documentation update (generated files)
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Docs only (Jules index and friction items)
- Risk class + why: Low, only generated documentation and a friction item are updated.
- Rollback: `git checkout .jules/index/generated/ && rm .jules/friction/open/cargo_mutants_schema_drift.md`
- Gates run: `cargo xtask jules-index`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`
- `.jules/friction/open/cargo_mutants_schema_drift.md`

## 🔜 Follow-ups
- Address the friction item around parsing friction item summaries in `jules-index`
