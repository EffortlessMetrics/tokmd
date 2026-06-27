## 💡 Summary
Fixed malformed metadata frontmatter in three historical friction items (`cargo_fuzz_asan_linker_failure.md`, `cargo_mutants_schema_drift.md`, and `surveyor_workspace_learning.md`) and regenerated the Jules indexes.

## 🎯 Why
The Archivist persona is responsible for consolidating recurring friction themes into better templates and summarizing run packets into generated indexes. Because these older items lacked standard `id/persona/style/shard/status` frontmatter, they couldn't be parsed correctly by the standard schema, missing context on past friction.

## 🔎 Evidence
- files: `.jules/friction/done/*.md`
- observed behavior before: Items lacked `id:`, `persona:`, `style:`, `shard:`, and `status:` headers matching the standard `FRICTION_ITEM.md`.
- command receipt: `cargo xtask jules-index --check` verifies the rollups match the corrected state.

## 🧭 Options considered
### Option A (recommended)
- what it is: Fix the metadata frontmatter in the friction items and run `cargo xtask jules-index`.
- why it fits this repo and shard: It directly satisfies Archivist targets #1 and #2 (consolidating friction templates and generating indexes) in the `workspace-wide` shard.
- trade-offs: Structure: High. Governance: High. Velocity: Neutral.

### Option B
- what it is: Only regenerate the indexes without fixing historical metadata.
- when to choose it instead: If the metadata formats were intentionally non-standard.
- trade-offs: We would leave broken metadata in the system.

## ✅ Decision
Option A. It's an honest patch that directly improves the Jules scaffolding and indexing health by fixing the root cause of malformed historical friction items and updating the generated rollup files.

## 🧱 Changes made (SRP)
- Re-formatted `.jules/friction/done/cargo_fuzz_asan_linker_failure.md` to include valid frontmatter.
- Re-formatted `.jules/friction/done/cargo_mutants_schema_drift.md` to include valid frontmatter.
- Re-formatted `.jules/friction/done/surveyor_workspace_learning.md` to include valid frontmatter.
- Ran `cargo xtask jules-index` to update `.jules/index/generated/RUNS_ROLLUP.md`.

## 🧪 Verification receipts
```text
{"ts_utc": "2024-06-27T16:15:00Z", "phase": "investigation", "cwd": "/app", "cmd": "grep -L \"id:\" .jules/friction/done/*.md", "status": 0, "summary": "Found historical friction items missing frontmatter."}
{"ts_utc": "2024-06-27T16:16:00Z", "phase": "implementation", "cwd": "/app", "cmd": "cat << 'EOF' > .jules/friction/done/... (cargo_fuzz_asan_linker_failure, cargo_mutants_schema_drift, surveyor_workspace_learning)", "status": 0, "summary": "Corrected the friction item metadata to match standard schema."}
{"ts_utc": "2024-06-27T16:17:00Z", "phase": "verification", "cwd": "/app", "cmd": "cargo xtask jules-index", "status": 0, "summary": "Regenerated indexes"}
{"ts_utc": "2024-06-27T16:18:00Z", "phase": "verification", "cwd": "/app", "cmd": "cargo xtask jules-index --check", "status": 0, "summary": "Verified indexes are up to date"}
```

## 🧭 Telemetry
- Change shape: Documentation and metadata indexing
- Blast radius: Jules documentation / scaffolding
- Risk class: Low
- Rollback: `git restore .jules/friction/done/ .jules/index/generated/`
- Gates run: `cargo xtask jules-index`, `cargo xtask jules-index --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/<run-id>/envelope.json`
- `.jules/runs/<run-id>/decision.md`
- `.jules/runs/<run-id>/receipts.jsonl`
- `.jules/runs/<run-id>/result.json`
- `.jules/runs/<run-id>/pr_body.md`

## 🔜 Follow-ups
None.
