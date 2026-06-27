## 💡 Summary
Updated `ROADMAP.md` and `docs/implementation-plan.md` to reflect the completed `v1.12.0`, `v1.13.x`, and `v1.14.0` releases. This moves those shipped milestones out of the "Future Horizons" sections and into the "Completed" sections, aligning the planning docs with the actual shipped reality described in `CHANGELOG.md`.

## 🎯 Why
There was factual drift between the shipped reality (the project is at `v1.14.0`) and the roadmap/implementation docs. `ROADMAP.md` still listed `v1.12.x` under "Future Horizons", and `docs/implementation-plan.md` stopped entirely at Phase 5d (`v1.11.0`). This drift misleads contributors and agents trying to understand the current state and future plans.

## 🔎 Evidence
- `ROADMAP.md` listed `v1.12.x` under "Future Horizons", while `CHANGELOG.md` showed `1.14.0` was released on 2026-06-25.
- `docs/implementation-plan.md` ended at Phase 5d / `v1.11.0` and Phase 6 (`v2.0`).
- Found multiple readiness docs (e.g., `docs/releases/1.12.md`, `1.13-readiness.md`, `1.14-ledger.md`) proving these releases were successfully shipped.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update both `ROADMAP.md` and `docs/implementation-plan.md` to move the shipped 1.12, 1.13, and 1.14 features into their respective "Completed" sections and outline the new "Future Horizon".
- why it fits this repo and shard: Directly fulfills the Cartographer mission to fix roadmap/design drift and keep docs honest.
- trade-offs:
  - Structure: Improves factual coherence across planning documents and aligns them with `CHANGELOG.md`.
  - Velocity: Unblocks clear future planning by archiving already-shipped milestones into the completed sections.
  - Governance: Directly aligns with the Cartographer mission to fix roadmap/design drift and keep docs honest.

### Option B
- what it is: Focus only on fixing the `ROADMAP.md` status summary table and leave the implementation plan and detailed roadmap sections untouched.
- when to choose it instead: If the implementation plan is intentionally left as a historical artifact that shouldn't be updated.
- trade-offs: Leaves contradictory information in the docs where v1.12 is both "complete" in the table and "future" in the text, confusing contributors and agents alike.

## ✅ Decision
Option A. It fully satisfies the primary Cartographer target of fixing factual drift between the shipped reality (`v1.14.0`) and the stale roadmap/implementation docs.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Moved `v1.12.x` content to "Completed" sections, added entries for `v1.13.0`, `v1.13.1`, and `v1.14.0`, and bumped "Future Horizons" to `v1.15.x`.
- `docs/implementation-plan.md`: Added Phase 5e (`v1.12.0`), Phase 5f (`v1.13.x`), and Phase 5g (`v1.14.0`) to document the work items completed in those releases.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.
doc artifacts ok: 2 required doc(s), 65 family file(s), 1 active goal(s), 24 spec-index artifact(s), 0 spec-index lane(s)

$ cargo xtask publish-surface --json --verify-publish
Checking version consistency against workspace version 1.14.0
  ✓ Cargo crate versions match 1.14.0.
  ✓ Cargo workspace dependency versions match 1.14.0.
  ✓ Node package manifest versions match 1.14.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo xtask version-consistency
Checking version consistency against workspace version 1.14.0
  ✓ Cargo crate versions match 1.14.0.
  ✓ Cargo workspace dependency versions match 1.14.0.
  ✓ Node package manifest versions match 1.14.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo test -p xtask
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.03s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.23s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.27s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.41s
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.24s
```

## 🧭 Telemetry
- Change shape: Documentation update (ROADMAP.md and implementation-plan.md).
- Blast radius: Docs only.
- Risk class: Low - factual updates to planning documents to reflect already shipped code. No code changes.
- Rollback: `git checkout HEAD^ ROADMAP.md docs/implementation-plan.md`
- Gates run: `cargo xtask docs --check`, `cargo xtask publish-surface`, `cargo xtask version-consistency`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
