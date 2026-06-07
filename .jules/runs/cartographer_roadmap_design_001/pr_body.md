## 💡 Summary
Updated `ROADMAP.md` and `docs/implementation-plan.md` to accurately reflect the completion of the `v1.12.0` Bun UB evidence-readiness release, resolving drift between the docs and the actual shipped features documented in `CHANGELOG.md` and `docs/releases/1.12.md`.

## 🎯 Why
The `v1.12.0` release has already shipped (as noted in `CHANGELOG.md` and `docs/releases/1.12.md`), but `ROADMAP.md` and `docs/implementation-plan.md` still listed `v1.12.x` as a future horizon or potential lane. This factual drift creates confusion for contributors reading the roadmap to understand current project priorities. Addressing this aligns the shared planning docs with shipped reality.

## 🔎 Evidence
Minimal proof:
- `ROADMAP.md` listed `v1.12.x` under "Future Horizons" and omitted it from the "Status Summary".
- `docs/implementation-plan.md` listed it under "Future Horizons".
- `CHANGELOG.md` lists `[1.12.0]` as shipped on 2026-06-04.
- `docs/releases/1.12-ledger.md` details the shipped work items, which matches my updates to the plan.

## 🧭 Options considered
### Option A (recommended)
- Update `ROADMAP.md` and `docs/implementation-plan.md` to accurately reflect that `v1.12.0` has shipped.
- It aligns the primary documentation surfaces with the actual release state and removes misleading future planning notes.
- Trade-offs: Structure is improved, governance surfaces are aligned, no velocity cost since it's a documentation-only update.

### Option B
- Update only `docs/implementation-plan.md` and ignore `ROADMAP.md`.
- Choose this if `ROADMAP.md` is strictly for long-term vision and doesn't track minor releases.
- Trade-offs: This leaves factual drift in the Status Summary table in `ROADMAP.md`, contradicting the "Status Summary" purpose.

## ✅ Decision
Option A. It fits the Cartographer persona's mission to keep roadmap docs aligned with shipped reality.

## 🧱 Changes made (Srp)
- `ROADMAP.md`: Added `v1.12.0` to the "Status Summary" table. Updated "Current Roadmap Status" text. Moved `v1.12.x` from "Future Horizons" to "Completed: v1.12.0" with bullet points matching `1.12-ledger.md`.
- `docs/implementation-plan.md`: Added "Phase 5e: Bun UB Evidence Readiness (v1.12.0)" before Phase 6. Added work items based on shipped features.

## 🧪 Verification receipts
```text
cargo xtask docs --check
cargo xtask version-consistency
cargo xtask publish --plan --verbose
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius: Docs only
- Risk class: Low
- Rollback: `git restore ROADMAP.md docs/implementation-plan.md`
- Gates run: `cargo xtask docs --check`, `cargo xtask version-consistency`, `cargo xtask publish --plan --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design_001/envelope.json`
- `.jules/runs/cartographer_roadmap_design_001/decision.md`
- `.jules/runs/cartographer_roadmap_design_001/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design_001/result.json`
- `.jules/runs/cartographer_roadmap_design_001/pr_body.md`

## 🔜 Follow-ups
None.
