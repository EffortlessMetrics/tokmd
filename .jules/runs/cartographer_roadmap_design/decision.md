## Cartographer: Roadmap & Implementation Plan Drift

### Investigation
I compared `ROADMAP.md` and `docs/implementation-plan.md` against the real shipped reality described in `CHANGELOG.md` and `ROADMAP.md`'s own "What shipped in v1.9.0" section.

`ROADMAP.md` claims:
```markdown
| **v1.9.0** | ✅ Complete | Browser/WASM productization: parity-covered wasm entrypoints, browser runner MVP, and public repo ingestion via tree+contents |

...

## v1.9.0 — Browser/WASM Productization
### What shipped in v1.9.0
- [x] ...
```

However, `docs/implementation-plan.md` still lists Phase 5 (WASM-Ready Core + Browser Runner) as incomplete and targeting v1.9.0:
```markdown
## Phase 5: WASM-Ready Core + Browser Runner (v1.9.0)

**Goal**: Turn the new host-abstraction seam into a real in-memory/WASM execution path and ship a browser-first runner.
...
### Work Items

- [ ] Route scan and walk through host-provided I/O traits
- [ ] Add wasm CI builds and parity checks against native output
...
```

This is a clear drift: v1.9.0 shipped and completed the browser/WASM productization, but the implementation plan still lists it as pending work.

### Options considered

#### Option A: Update `docs/implementation-plan.md` to reflect v1.9.0 as ✅ Complete
- What it is: Mark Phase 5 as complete, check all its work item boxes, and update the phase heading.
- Why it fits: Aligns the implementation plan with the current reality described in ROADMAP.md and the codebase.
- Trade-offs: Minor documentation churn, but ensures future contributors aren't misled by stale work items.

#### Option B: Remove Phase 5 from `docs/implementation-plan.md`
- What it is: Delete the phase since it's already shipped.
- When to choose it instead: If the implementation plan is only meant for future unstarted work.
- Trade-offs: Loses the historical context of what was planned and accomplished in that phase, which is useful for tracking project evolution. The file currently retains completed phases (e.g. Phase 4c, 4d, 4e), so deleting Phase 5 would break consistency.

### Decision
**Option A**. I will update `docs/implementation-plan.md` to mark Phase 5 as `✅ Complete` and check off the work items, matching the pattern of previously completed phases in the document and aligning with `ROADMAP.md`.
