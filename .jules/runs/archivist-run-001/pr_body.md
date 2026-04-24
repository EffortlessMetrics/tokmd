## 💡 Summary
Migrated deprecated legacy ledgers from `.jules/docs/` and `.jules/quality/` to the centralized `.jules/runs/<run-id>` format and updated the generated run index.

## 🎯 Why
The repository memory explicitly states: "Legacy ledgers in .jules/docs/ or .jules/quality/ are deprecated." Maintaining old, fragmented formats makes indexing harder and creates confusion. This centralizes all historical runs into the format expected by `.jules/bin/build_index.py`.

## 🔎 Evidence
- `.jules/docs/ledger.json` and `.jules/quality/ledger.json` contained disjoint tracking data.
- `.jules/bin/build_index.py` only scans `.jules/runs/`.
- Memory explicitly requires consolidating run packets to the current format.

## 🧭 Options considered
### Option A (recommended)
- Migrate all legacy packets to `.jules/runs/` and remove `.jules/docs/` and `.jules/quality/`.
- **Why it fits this repo and shard**: Align with the "workspace-wide" and "governance-release" shard by reducing structural drift and standardizing metadata.
- **Trade-offs**:
  - Structure: Improves consistency.
  - Velocity: Small upfront cost to migrate.
  - Governance: Unifies truth source.

### Option B
- Modify `build_index.py` to support multiple legacy formats.
- **When to choose it instead**: Only if migrating data is impossible or forbidden.
- **Trade-offs**: Permanent structural debt and fragmented parsers.

## ✅ Decision
Option A was chosen. It directly satisfies the Archivist persona's mission to consolidate run packets, learnings, and shared scaffolding, removing deprecated ledgers from the repository.

## 🧱 Changes made (SRP)
- Created `.jules/runs/09c9d819-02cd-4f63-b662-921c812f93dd/` with migrated packet.
- Created `.jules/runs/36cec87d-2836-42ed-9ae1-33dbf2702319/` with migrated packet.
- Created `.jules/runs/d657338a-caa9-4ccf-93a1-4733ada7154c/` with migrated packet.
- Deleted `.jules/docs/` and `.jules/quality/` legacy directories.
- Executed `.jules/bin/build_index.py` to regenerate `.jules/index/generated/RUNS_ROLLUP.md`.

## 🧪 Verification receipts
```text
{"command": "mkdir -p .jules/runs/archivist-run-001"}
{"command": "python3 migrate_ledgers.py"}
{"command": "rm -rf .jules/docs/ .jules/quality/"}
{"command": "python3 .jules/bin/build_index.py"}
{"command": "cargo fmt -- --check"}
{"command": "cargo clippy -- -D warnings"}
```

## 🧭 Telemetry
- **Change shape**: Migration of JSON/MD run data + directory pruning.
- **Blast radius**: Minimal. No product code touched. Only Jules metadata (`.jules/*`).
- **Risk class**: Low. Pure metadata refactoring.
- **Rollback**: `git revert`.
- **Gates run**: docs/quality checks (`fmt`, `clippy`).

## 🗂️ .jules artifacts
- `.jules/runs/archivist-run-001/envelope.json`
- `.jules/runs/archivist-run-001/decision.md`
- `.jules/runs/archivist-run-001/receipts.jsonl`
- `.jules/runs/archivist-run-001/result.json`
- `.jules/runs/archivist-run-001/pr_body.md`

## 🔜 Follow-ups
None.