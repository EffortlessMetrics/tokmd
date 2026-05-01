## 🧭 Options considered
### Option A (recommended)
- Update `.jules/bin/build_index.py` to also parse and summarize friction items in `.jules/friction/open/` into a `FRICTION_ROLLUP.md` file, alongside `RUNS_ROLLUP.md`.
- Ensure it respects the `FRICTION_ITEM.md` template schema (frontmatter/metadata fields).
- **Why it fits**: The prompt strictly instructs the Archivist persona to "consolidate recurring friction themes into better templates/policy/docs" and "summarize per-run packets into generated indexes/rollups". This directly implements the missing functionality for `FRICTION_ROLLUP.md` that is expected in memory.
- **Trade-offs**:
  - **Structure**: High. Formalizes friction tracking.
  - **Velocity**: Medium. Requires scripting but automates future manual tracking.
  - **Governance**: High. Improves observability of recurring problems.

### Option B
- Just manually write a `FRICTION_ROLLUP.md` one time.
- **Why to choose it instead**: Less script writing.
- **Trade-offs**: Will quickly drift and become outdated as future runs generate more friction items, missing the goal of "generated indexes/rollups".
