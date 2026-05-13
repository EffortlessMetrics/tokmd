## 💡 Summary
Updated `ROADMAP.md` to reflect that Tree-sitter AST integration shadow work has actively begun.

## 🎯 Why
`ROADMAP.md` claimed AST integration was completely deferred beyond the v2.x roadmap. However, `docs/implementation-plan.md` (Phase 7) and `docs/adr/0008-ast-foundation.md` confirm that the foundation shadow work has shipped behind a feature flag. This PR fixes the factual drift.

## 🔎 Evidence
- `ROADMAP.md`
- Observed `v3.0` deferred language mismatching `docs/implementation-plan.md`.
- `cargo xtask docs --check` passes.

## 🧭 Options considered
### Option A (recommended)
- Update `ROADMAP.md` to match reality where shadow mode is active.
- Fits the repo and shard by ensuring design/roadmap docs match shipped state.
- Trade-offs: Structure is improved, velocity is fast, governance aligns public roadmap with internal plans.

### Option B
- Ignore the drift and wait for v3.0 to actually launch.
- Fails to fix the prompt requirement for factual alignment.
- Trade-offs: Lower accuracy in public documents.

## ✅ Decision
Option A was selected to fix the factual drift.

## 🧱 Changes made (SRP)
- `ROADMAP.md`

## 🧪 Verification receipts
```text
Documentation is up to date.
doc artifacts ok: 1 required doc(s), 11 family file(s), 1 active goal(s)
```

## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius: docs
- Risk class: low
- Rollback: git revert
- Gates run: `cargo xtask docs --check`, `cargo clippy`, `cargo fmt`

## 🗂️ .jules artifacts
- `.jules/runs/carto-roadmap-design-1/envelope.json`
- `.jules/runs/carto-roadmap-design-1/decision.md`
- `.jules/runs/carto-roadmap-design-1/receipts.jsonl`
- `.jules/runs/carto-roadmap-design-1/result.json`
- `.jules/runs/carto-roadmap-design-1/pr_body.md`

## 🔜 Follow-ups
None.
