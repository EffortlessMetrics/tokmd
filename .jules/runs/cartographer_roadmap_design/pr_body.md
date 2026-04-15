## 💡 Summary
Align the architecture design document with the shipped reality. Added missing `tokmd-analysis-format-md` to the Tier 3 orchestration table in `docs/architecture.md`. `tokmd-sensor` and `tokmd-tokeignore` were already present.

## 🎯 Why
The codebase has evolved since the architecture doc was last fully synced. `tokmd-analysis-format-md` was extracted from `tokmd-analysis-format` to enforce the Single Responsibility Principle, but was never documented in the `docs/architecture.md` tier tables, leading to factual drift between the design docs and the codebase.

## 🔎 Evidence
- `crates/tokmd-analysis-format-md/Cargo.toml` exists and states "Markdown rendering for tokmd analysis receipts."
- `docs/architecture.md` was missing this crate under the Tier 3 orchestration adapters.

## 🧭 Options considered

### Option A (recommended)
- what it is: Update `docs/architecture.md` to correctly list `tokmd-analysis-format-md` in the Tier 3 table.
- why it fits this repo and shard: It strictly addresses the tooling-governance shard's requirement to fix factual drift in architecture docs.
- trade-offs: Structure is improved, velocity is preserved, and governance rules are satisfied without regressions.

### Option B
- what it is: Update the README.md in the microcrate instead.
- when to choose it instead: If the microcrate was meant to be purely private and undocumented.
- trade-offs: `docs/architecture.md` is the central source of truth for the workspace hierarchy, so missing crates here is actively confusing.

## ✅ Decision
Option A was chosen. Adding the extracted format crate correctly documents the architecture surface.

## 🧱 Changes made (SRP)
- `docs/architecture.md`: Added `tokmd-analysis-format-md` to Tier 3.

## 🧪 Verification receipts
```text
cargo xtask docs --check
Documentation is up to date.
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: None (documentation only)
- Risk class: Low
- Rollback: Revert docs update
- Gates run: `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
