## 💡 Summary
Align documentation and roadmap artifacts with the actual shipped capabilities of tokmd versions 1.12.0 and 1.13.0.

## 🎯 Why
The `ROADMAP.md` and `docs/implementation-plan.md` still framed `v1.12.x` as future work and entirely omitted `v1.13.x`. Similarly, `docs/design.md` was missing the `bun-ub` analysis preset in the architectural documentation table, and `docs/requirements.md` was missing the `syntax` and `evidence-packet` commands from its CLI interface listing. Keeping docs aligned with shipped releases avoids contributor confusion and prevents strategy drift.

## 🔎 Evidence
- **Observed Behavior**: `tokmd --help` returns `syntax` and `evidence-packet` commands, and `CHANGELOG.md` explicitly references the `bun-ub` preset added in 1.12.0.
- `cargo xtask publish --plan --verbose` lists exactly 16 crates in the active publish plan.
- The `v1.12.0` and `v1.13.0` releases already landed but their features were missing from architectural definitions.

## 🧭 Options considered
### Option A (recommended)
- Explicitly integrate the `v1.12.0` and `v1.13.0` milestones into the completed phases of `ROADMAP.md` and `docs/implementation-plan.md`.
- Register the missing `bun-ub` preset in `docs/design.md`.
- Register `syntax` and `evidence-packet` in `docs/requirements.md`.
- **Trade-offs**: Improves Governance and clarity; adds minimal maintenance text without structural refactors.

### Option B
- Write a learning PR noting the documentation gaps without making edits.
- **When to choose it**: If the features were still experimental or explicitly deferred from public docs.
- **Trade-offs**: Leaves confusing artifacts in place for future readers.

## ✅ Decision
Option A was chosen because updating the roadmap, design, and CLI requirement artifacts represents a low-risk, high-value governance alignment required for accurate codebase telemetry.

## 🧱 Changes made (SRP)
- `docs/design.md`: Added `bun-ub` to the Analysis preset system table.
- `docs/requirements.md`: Added `syntax` and `evidence-packet` to the CLI interface table.
- `ROADMAP.md`: Marked `v1.12.0` and `v1.13.0` as ✅ Complete and removed `v1.12.x` from Future Horizons.
- `docs/implementation-plan.md`: Integrated Phase 5e (bun-ub) and Phase 5f (syntax/evidence-packets) as completed.

## 🧪 Verification receipts
```text
Running `target/debug/xtask publish --plan --verbose`
Publish order (16 crates):
   1. tokmd-gate
   ...
  16. tokmd
Checking version consistency against workspace version 1.13.1
  ✓ Cargo crate versions match 1.13.1.
Version consistency checks passed.
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Documentation update (Architecture and Roadmap Alignment)
- Blast radius: None (docs only)
- Risk class: Low
- Rollback: `git revert`
- Gates run: `cargo xtask docs --check`, `cargo xtask version-consistency`, `cargo xtask publish --plan`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
