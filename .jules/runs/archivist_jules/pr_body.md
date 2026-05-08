## 💡 Summary
Fixed malformed metadata frontmatter in two existing friction items and regenerated the Jules indexes. This ensures the `.jules/index/generated/FRICTION_ROLLUP.md` accurately tracks personas, styles, and shards for open friction rather than listing them as "Unknown", and updates the `RUNS_ROLLUP.md` with missing runs.

## 🎯 Why
The Archivist persona is responsible for consolidating recurring friction themes into better templates and summarizing run packets into generated indexes. Because `librarian_doctest_git_dependency.md` and `steward-release-clean-state.md` lacked standard `id/persona/style/shard/status` frontmatter, the `cargo xtask jules-index` aggregator could not properly identify them.

## 🔎 Evidence
- files: `.jules/index/generated/FRICTION_ROLLUP.md`
- observed behavior before: `librarian_doctest_git_dependency` showed `Unknown` persona and style.
- command receipt: `cargo xtask jules-index` successfully regenerates with corrected metadata.

## 🧭 Options considered
### Option A (recommended)
- what it is: Fix the metadata frontmatter in the friction items and run `cargo xtask jules-index`.
- why it fits this repo and shard: It directly satisfies Archivist targets #1 and #2 (consolidating friction templates and generating indexes) in the `workspace-wide` shard.
- trade-offs: Structure: High. Governance: High. Velocity: Neutral.

### Option B
- what it is: Only regenerate the indexes without fixing metadata.
- when to choose it instead: If the metadata formats were intentionally non-standard (they weren't).
- trade-offs: We would leave broken metadata rendering as "Unknown" in the generated docs.

## ✅ Decision
Option A. It's an honest patch that directly improves the Jules scaffolding and indexing health by fixing the root cause of the "Unknown" rows.

## 🧱 Changes made (SRP)
- Re-formatted `.jules/friction/open/librarian_doctest_git_dependency.md` to include valid frontmatter.
- Re-formatted `.jules/friction/open/steward-release-clean-state.md` to include valid frontmatter.
- Ran `cargo xtask jules-index` to update `.jules/index/generated/RUNS_ROLLUP.md` and `.jules/index/generated/FRICTION_ROLLUP.md`.

## 🧪 Verification receipts
```text
{"ts_utc": "2024-05-08T20:55:00Z", "phase": "investigation", "cwd": "/app", "cmd": "cat .jules/index/generated/FRICTION_ROLLUP.md", "status": 0, "summary": "Found that friction items librarian_doctest_git_dependency and steward-release-clean-state had Unknown metadata in the rollup."}
{"ts_utc": "2024-05-08T20:56:00Z", "phase": "implementation", "cwd": "/app", "cmd": "cat << 'EOF' > .jules/friction/open/librarian_doctest_git_dependency.md ...", "status": 0, "summary": "Fixed frontmatter for librarian_doctest_git_dependency.md and steward-release-clean-state.md to match schema."}
{"ts_utc": "2024-05-08T20:57:00Z", "phase": "implementation", "cwd": "/app", "cmd": "cargo xtask jules-index", "status": 0, "summary": "Regenerated the indexes, which correctly updated FRICTION_ROLLUP.md and RUNS_ROLLUP.md"}
```

## 🧭 Telemetry
- Change shape: Documentation and metadata indexing
- Blast radius: Jules documentation / scaffolding
- Risk class: Low
- Rollback: `git restore .jules/friction/open/ .jules/index/generated/`
- Gates run: `cargo xtask jules-index`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
