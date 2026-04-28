# Cartographer Decision

## Option A (recommended): Mark Phase 5 work items as completed in implementation-plan.md
- **What it is**: The `ROADMAP.md` explicitly lists `v1.9.0` (which is Phase 5) as `✅ Complete`. The `ROADMAP.md` documents what shipped: `tokmd-io-port`, `tokmd-wasm`, parity tests, and the `web/runner`. However, `docs/implementation-plan.md` still lists Phase 5 Work Items as `[ ]` (unchecked) and doesn't mark the whole section as "✅ Complete". Additionally, the `v2.0` section in `implementation-plan.md` has a typo: it says "MCP server mode (Phase 5)" instead of "Phase 6".
- **Why it fits**: This exactly aligns with target ranking #3 "stale implementation-plan sections that mislead contributors". It keeps design/roadmap docs aligned with the real system.
- **Trade-offs**:
  - *Structure*: Fixes a direct inconsistency between `ROADMAP.md` and `implementation-plan.md`.
  - *Velocity*: Quick to implement, prevents contributors from thinking Phase 5 isn't done.
  - *Governance*: Keeps our roadmap sources of truth reliable.

## Option B: Also rewrite Phase 5 in implementation-plan.md to match the exact wording in ROADMAP.md
- **What it is**: Not just check the boxes, but completely replace the section in `implementation-plan.md` with the content from `ROADMAP.md`.
- **When to choose it**: When the implementation plan completely missed what actually happened.
- **Trade-offs**: Unnecessary churn. The implementation plan was the *plan*, it's fine if the wording differs slightly from the final release notes, as long as the status is marked complete and boxes are checked.

## ✅ Decision
Option A. I will update `docs/implementation-plan.md` to:
1. Append `✅ Complete` to the `Phase 5` heading.
2. Check off the boxes in the Phase 5 "Work Items" list, as `ROADMAP.md` confirms they are done.
3. Fix the typo in Phase 7 referring to "Phase 5" instead of "Phase 6" for MCP Server Mode.
