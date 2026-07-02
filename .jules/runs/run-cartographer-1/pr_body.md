## 💡 Summary
Updated `docs/implementation-plan.md` to align with the active state documented in `ROADMAP.md` and `docs/NOW.md`. Specifically, this documents the recently shipped strict GHCR runtime verification gate for the `mode: packet` Action, and introduces the `Phase 5h: Selection-First Product and Evidence Work (v1.15.x)` pause boundary to match the current roadmap.

## 🎯 Why
There was factual drift between the shipped product capabilities (like the strict GHCR verification gate) and the implementation plan. Furthermore, the `ROADMAP.md` defined a clear pause horizon (`v1.15.x`) before proceeding to `v2.0` (MCP Server Mode), but `docs/implementation-plan.md` skipped straight from `v1.14.0` (Phase 5g) to `v2.0` (Phase 6), creating a misleading planning surface.

## 🔎 Evidence
- `docs/NOW.md` explicitly states the `runtime: container` GHCR path is "now wired for verification-gated tags (currently 1.14.0, with mutable tags rejected)".
- `ROADMAP.md` shows the active planning state is selection-first (v1.15.x) before moving to v2.0 Platform Evolution.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `docs/implementation-plan.md` to accurately document the strict GHCR gate in Phase 5g, and insert the `Phase 5h` transition horizon to align with `ROADMAP.md` before Phase 6.
- why it fits this repo and shard: Directly targets roadmap/design/requirements drift from shipped reality (tooling-governance shard).
- trade-offs: Structure/Governance improves by ensuring both planning docs represent the same reality. Velocity trade-off is minimal docs-only change.

### Option B
- what it is: Ignore the implementation plan and only treat `ROADMAP.md` as truth.
- when to choose it instead: If the implementation plan was deprecated.
- trade-offs: Leaves misleading contributor docs active, violating Cartographer anti-drift rules.

## ✅ Decision
Option A. It accurately reflects the `v1.15.x` pause/selection phase and clarifies the strict GHCR gate in Phase 5g, aligning `docs/implementation-plan.md` with `ROADMAP.md` and the shipped reality.

## 🧱 Changes made (SRP)
- `docs/implementation-plan.md`: Added verification gate details to Phase 5g.
- `docs/implementation-plan.md`: Added Phase 5h for the selection-first pause before Phase 6 (MCP).

## 🧪 Verification receipts
```text
cargo xtask docs --check
cargo fmt -- --check
cargo clippy -- -D warnings
cargo xtask publish --plan --verbose
cargo xtask version-consistency
bash -c 'CI=true cargo test -p tokmd'
```

## 🧭 Telemetry
- Change shape: Docs-only
- Blast radius: docs
- Risk class: Low
- Rollback: git revert
- Gates run: docs, fmt, clippy, publish, version-consistency, test

## 🗂️ .jules artifacts
- `.jules/runs/run-cartographer-1/envelope.json`
- `.jules/runs/run-cartographer-1/decision.md`
- `.jules/runs/run-cartographer-1/receipts.jsonl`
- `.jules/runs/run-cartographer-1/result.json`
- `.jules/runs/run-cartographer-1/pr_body.md`

## 🔜 Follow-ups
None.
