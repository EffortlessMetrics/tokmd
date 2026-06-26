## 💡 Summary
Synchronized the documentation maps with the current shipped reality of the `v1.14.0` release. The PR evidence packet lane is fully shipped, and the roadmap artifacts are now aligned.

## 🎯 Why
The swarm workspace docs (`docs/ROADMAP.md` and `docs/implementation-plan.md`) and the public `ROADMAP.md` were lagging behind the recent releases, erroneously showing the PR evidence packet workflow as the "Active Lane" or pending in Phase 5e when the release ledgers explicitly prove `v1.14.0` shipped it. Updating these docs aligns contributor and agent contexts.

## 🔎 Evidence
- `docs/ROADMAP.md` listed PR Evidence Packet Workflows as the "Active Lane".
- `ROADMAP.md` did not mention `v1.14` in the completed phases summary below v1.11.
- `docs/implementation-plan.md` stated completion only "through 1.11.0".
- `docs/releases/1.14-ledger.md` explicitly lists the PR packet workflow as landed and the GHCR image as `verified-public` for `v1.14.0` as of 2026-06-25.

## 🧭 Options considered
### Option A (recommended)
- Update the `ROADMAP.md`, `docs/ROADMAP.md`, and `docs/implementation-plan.md` to reflect `v1.14.0` completion.
- Fits the Cartographer mandate to fix roadmap drift.
- trade-offs: Structure (keeps docs correct), Velocity (zero risk to code), Governance (maintains the single source of truth).

### Option B
- Do nothing and create a learning PR.
- when to choose it instead: If the drift was too large or unclear.
- trade-offs: Fails to fix an easily verifiable documentation gap.

## ✅ Decision
Option A was chosen to align the planning maps with the release ledgers.

## 🧱 Changes made (SRP)
- `docs/ROADMAP.md`: Marked the PR Evidence Packet Workflows lane as closed (completed in v1.14.0) and updated the GHCR verified-public timestamps/versions.
- `ROADMAP.md`: Added `v1.14` completion to the current status block.
- `docs/implementation-plan.md`: Bumped the completed phases boundary to `1.14.0`.

## 🧪 Verification receipts
```text
git diff docs/ROADMAP.md docs/implementation-plan.md ROADMAP.md
cargo xtask docs --check
cargo fmt -- --check
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: docs
- Risk class: Low (Documentation only)
- Rollback: Revert the commit.
- Gates run: Pre-commit checks

## 🗂️ .jules artifacts
- `.jules/runs/run-cartographer-1/envelope.json`
- `.jules/runs/run-cartographer-1/decision.md`
- `.jules/runs/run-cartographer-1/receipts.jsonl`
- `.jules/runs/run-cartographer-1/result.json`
- `.jules/runs/run-cartographer-1/pr_body.md`

## 🔜 Follow-ups
None.
