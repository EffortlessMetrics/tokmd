## 💡 Summary
Updated `ROADMAP.md` and `docs/implementation-plan.md` to reflect the completed v1.12.0 and v1.13.0/v1.13.1 releases, moving them out of "Future Horizons".

## 🎯 Why
These core planning and architecture documents lagged behind the actual shipped reality confirmed in `CHANGELOG.md` and release ledgers, which mislead contributors relying on the documentation.

## 🔎 Evidence
- `CHANGELOG.md` notes `1.12.0` (Bun UB evidence-readiness and `tokmd-swarm` workbench) and `1.13.0/1.13.1` (syntax-aware evidence packet).
- `ROADMAP.md` and `docs/implementation-plan.md` both listed `v1.12.x` under "Future Horizons".
- `cargo xtask version-consistency` confirms the workspace version is `1.13.1`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `ROADMAP.md` and `docs/implementation-plan.md` directly.
- why it fits this repo and shard: High confidence documentation drift correction.
- trade-offs: Structure / Velocity / Governance: Direct correction keeps planning structures honest to historical artifacts quickly.

### Option B
- what it is: Do not touch the roadmap and wait for a dedicated docs sweep.
- when to choose it instead: If the releases were currently rolling out or not yet verified.
- trade-offs: Fails to fix immediate drift.

## ✅ Decision
Option A is chosen to correct factual document drift.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: added `Completed: v1.12.0` and `Completed: v1.13.0/v1.13.1` sections; bumped `v1.12.x` future section to `v1.14.x`.
- `docs/implementation-plan.md`: added `Phase 5e` (v1.12.0) and `Phase 5f` (v1.13.x) matching the `ROADMAP.md` items.

## 🧪 Verification receipts
```text
$ cargo xtask docs --update
Documentation is up to date.

$ cargo xtask docs --check
doc artifacts ok: 2 required doc(s), 54 family file(s), 1 active goal(s), 19 spec-index artifact(s), 0 spec-index lane(s)

$ cargo xtask version-consistency
Checking version consistency against workspace version 1.13.1
  ✓ Cargo crate versions match 1.13.1.
  ✓ Cargo workspace dependency versions match 1.13.1.
  ✓ Node package manifest versions match 1.13.1.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.
```

## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius: docs
- Risk class + why: low risk (docs only)
- Rollback: `git checkout origin/main -- ROADMAP.md docs/implementation-plan.md`
- Gates run: `cargo xtask docs --check`, `cargo xtask version-consistency`

## 🗂️ .jules artifacts
- `.jules/runs/20260616105953/envelope.json`
- `.jules/runs/20260616105953/decision.md`
- `.jules/runs/20260616105953/receipts.jsonl`
- `.jules/runs/20260616105953/result.json`
- `.jules/runs/20260616105953/pr_body.md`

## 🔜 Follow-ups
None.
