## 💡 Summary
Updated the `ROADMAP.md` table to include the v1.12.0, v1.13.0, and v1.13.1 releases.

## 🎯 Why
The "Status Summary" table in `ROADMAP.md` drifted from the actual releases, completely missing versions 1.12 and 1.13. This change keeps our design and planning docs aligned with the real system as targeted by the Steward persona.

## 🔎 Evidence
- File path: `ROADMAP.md`
- Observed behavior: Missing versions in the `Status Summary` table when comparing against `CHANGELOG.md` and repo state.
- Command receipt:
```text
awk '/\| \*\*v1.11.0\*\*/ {print; print "| **v1.12.0** | ✅ Complete | Bun UB evidence-readiness and tokmd-swarm workbench. |\n| **v1.13.0** | ✅ Complete | Syntax-aware evidence packet surface. |\n| **v1.13.1** | ✅ Complete | Correction release for the syntax-aware evidence packet surface. |"; next} 1' ROADMAP.md > tmp_roadmap.md && mv tmp_roadmap.md ROADMAP.md
```

## 🧭 Options considered
### Option A (recommended)
- Update the `ROADMAP.md` table to include the 1.12.0, 1.13.0, and 1.13.1 releases.
- Fits the repo and shard perfectly because it targets release metadata or changelog mismatch.
- Structure: High. Velocity: High. Governance: High.

### Option B
- Look for another mismatched doc or configuration.
- Choose if ROADMAP.md was actually up-to-date or we wanted to make bigger refactoring changes.
- Trade-offs: Increases search time unnecessarily when a perfect target is available.

## ✅ Decision
Chose Option A to keep the release surface documents aligned, fixing the obvious version drift in the `ROADMAP.md` file.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Inserted v1.12.0, v1.13.0, and v1.13.1 into the `Status Summary` table.

## 🧪 Verification receipts
```text
cargo xtask publish --plan --verbose
cargo xtask version-consistency
cargo xtask docs --check
cargo fmt -- --check
cargo clippy --workspace --all-features --tests --benches -- -D warnings
```
All passed.

## 🧭 Telemetry
- Change shape: Documentation update.
- Blast radius: Docs. Zero logic impact.
- Risk class: None. Simple Markdown change.
- Rollback: Revert the commit.
- Gates run: `governance-release` fallback commands run.

## 🗂️ .jules artifacts
- `.jules/runs/run-steward-1/envelope.json`
- `.jules/runs/run-steward-1/decision.md`
- `.jules/runs/run-steward-1/receipts.jsonl`
- `.jules/runs/run-steward-1/result.json`
- `.jules/runs/run-steward-1/pr_body.md`

## 🔜 Follow-ups
None.
