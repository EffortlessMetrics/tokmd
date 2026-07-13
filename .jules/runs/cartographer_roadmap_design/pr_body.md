## 💡 Summary
Updated `ROADMAP.md` and `docs/implementation-plan.md` to reflect that the PR evidence-packet workflow lane (v1.14.0) is complete and that the active planning mode is selection-first (v1.15.x).

## 🎯 Why
These docs contained stale implementation-plan sections and drifted from the shipped reality of v1.14.0. Both files still referred to the v1.11.0 lane (Browser runtime polish) or v1.14.0 as incomplete or active, when it actually shipped. This drift misleads contributors and agents trying to understand the current product state.

## 🔎 Evidence
- `ROADMAP.md` and `docs/implementation-plan.md` stated that v1.11 was complete but `docs/implementation-plan.md` ended there, and `ROADMAP.md`'s status paragraph still stated v1.11 was the most recent.
- `docs/ROADMAP.md` listed PR evidence packets as the active product lane, but `CHANGELOG.md` shows it was shipped in v1.14.0.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update both `ROADMAP.md` and `docs/implementation-plan.md` to show v1.14.0 as complete and shift active development to v1.15.x selection-first.
- why it fits this repo and shard: Directly fulfills the Cartographer mission to fix factual drift between shipped reality and roadmap docs.
- trade-offs:
  - Structure: Aligns planning docs with `CHANGELOG.md`.
  - Velocity: Unblocks future work by accurately showing what's done.
  - Governance: Correctly sets the expectation that new work must be selection-first based on evidence.

### Option B
- what it is: Leave `docs/implementation-plan.md` alone and only fix the status text in `ROADMAP.md`.
- when to choose it instead: If the implementation plan is intentionally left as a historical artifact (which contradicts this persona's instructions).
- trade-offs: Leaves contradictory information in the docs, confusing contributors.

## ✅ Decision
Option A. It fully satisfies the primary Cartographer target of fixing factual drift.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Updated Current Roadmap Status to reflect v1.14.0 completion and selection-first planning.
- `docs/ROADMAP.md`: Updated Current Status and Near-Term Roadmap to reflect v1.14.0 completion.
- `docs/implementation-plan.md`: Updated the intro text and Phase 5h to indicate v1.14.0 is complete and v1.15.x is active.

## 🧪 Verification receipts
```text
$ patch ROADMAP.md Current Status
(exit 0)
```

```text
$ patch docs/ROADMAP.md
(exit 0)
```

```text
$ patch docs/implementation-plan.md
(exit 0)
```

```text
$ pre-commit verification
(exit 0)
```



## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius: docs
- Risk class: lowest (doc only)
- Rollback: git revert
- Gates run: `cargo xtask docs --check`, `cargo xtask doc-artifacts --check`, `cargo xtask proof-policy --check`, `cargo xtask publish --plan`, `cargo xtask version-consistency`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
