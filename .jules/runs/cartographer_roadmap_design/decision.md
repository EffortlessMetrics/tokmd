## Options Considered

### Option A: Clean up v1.10.0 roadmap/implementation-plan drift and outline missing v1.11.0 sections
- **What it is**:
  - Since v1.10.0 was completed in RC and stable, update ROADMAP.md and implementation-plan.md to reflect v1.10.0 completion properly in sections not just the status table.
  - v1.11.0 section in ROADMAP.md is missing in detailed phase mapping.
  - Outline Phase 5c for Browser Runtime Polish (v1.11.0) in implementation-plan.md, which is currently just listed as "Follow-up" or missing in some roadmap parts.
  - The status table has `v1.10.0-rc.1` and `v1.10.0` but the detailed milestones below don't have the final v1.10.0.
  - Wait, let me check ROADMAP.md detailed milestones for v1.10.0.
- **Why it fits**: Directly addresses roadmap and implementation plan drift and clarity.

Let's look at `ROADMAP.md` detailed milestones.

Looking at `ROADMAP.md` and `docs/implementation-plan.md`, `v1.10.0` is marked as "Completed" in `ROADMAP.md` status table (`v1.10.0-rc.1` and `v1.10.0`).
However, in `ROADMAP.md` detailed sections, it says:
`## v1.10.0 — CI Control Plane, Trust Hardening, and Proof Stability`
but does not have `✅ Complete` in the header, while earlier ones (like `## Completed: v1.8.0 ...`) do.
Wait, looking closely at `get_sections.py` output:
`## v1.10.0 — CI Control Plane, Trust Hardening, and Proof Stability` is not prefixed with `Completed: ` like `v1.8.0`, nor does it have `✅ Complete`.
Also `v1.9.0` says `## v1.9.0 — Browser/WASM Productization` (missing `Completed: ` or `✅ Complete`). Wait, in the status table v1.9.0 is ✅ Complete.

In `docs/implementation-plan.md`:
`## Phase 5b: Release Train Hardening (v1.10.0) ✅ Complete` -> It actually does have `✅ Complete`.
But `Phase 5c: Browser Runtime Polish (v1.11.0)` is missing as a proper phase section in `implementation-plan.md` - it's just listed as a "Follow-Up".

Also, there is no detailed milestone for `v1.11.0` in `ROADMAP.md`. It just lists "Deferred to v1.11.0" under v1.10.0, and "v1.11.0" is in the table as `🔭 Planned`.
Wait, looking at `ROADMAP.md` for `v1.9.0`: `What shipped in v1.9.0` implies it's complete.

Let's do Option A: Align the headers in `ROADMAP.md` to reflect `v1.9.0` and `v1.10.0` as completed (they shipped), and add a formal `v1.11.0` section to `ROADMAP.md` and `docs/implementation-plan.md` outlining the planned work so future reviewers have a clear target, removing the "Deferred to v1.11.0" from v1.10.0 and moving it to its own heading.

### Option B: Create a Learning PR indicating that there are no significant design drifts to fix.
- **What it is**: No code changes. Just create a run packet and open a learning PR.
- **Why**: Maybe the drift is too small to warrant a change.

### Decision
Option A. The `ROADMAP.md` and `docs/implementation-plan.md` are actively read to understand what the next major efforts are. A `v1.11.0` section doesn't exist yet, but it's the immediate next planned milestone. Leaving it hidden as "Deferred to v1.11.0" inside the `v1.10.0` section makes the roadmap confusing, and `v1.9.0`/`v1.10.0` are structurally missing the `Completed: ` header prefix that `v1.8.0` and earlier have. This is exactly what the `Cartographer` persona should fix.
