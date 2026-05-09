## 💡 Summary
Updated `docs/architecture-consolidation-plan.md` to resolve factual drift between the document and the shipped codebase. Removed completed tasks from the active pressure points and suggested PR list, and updated file line counts to accurately reflect current state.

## 🎯 Why
The `docs/architecture-consolidation-plan.md` was out-of-date compared to reality. It claimed that `crates/tokmd-cockpit/src/gates.rs` was 1196 lines and needed splitting, but it had already been split into `crates/tokmd-cockpit/src/gates/` and is now only 118 lines. Other files had inaccurate line counts. Keeping this document aligned with reality ensures that future contributors and bots aren't misled.

## 🔎 Evidence
- file path(s): `docs/architecture-consolidation-plan.md`
- observed behavior / finding: The `gates.rs` file was listed as a pressure point with ~1196 lines. Running `wc -l crates/tokmd-cockpit/src/gates.rs` yielded 118 lines. `ls -la crates/tokmd-cockpit/src/gates/` showed the extracted modules.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `docs/architecture-consolidation-plan.md` to fix the factual drift by removing the completed tasks (Cockpit gates) and correcting the line counts for the others.
- why it fits this repo and shard: Directly aligns with Cartographer's mission to fix factual drift between shipped reality and roadmap/design docs.
- trade-offs: Structure / Velocity / Governance: Improves governance and clarity by cleaning up stale instructions.

### Option B
- what it is: Ignore the specific line numbers and just remove the completed items from "First Suggested PRs".
- when to choose it instead: If the line counts fluctuated constantly and weren't meant to be accurate indicators of size.
- trade-offs: Leaves the pressure points table factually incorrect and misaligned with reality.

## ✅ Decision
Chosen Option A. It completely addresses the drift between the documented plan and the actual codebase, preventing future work from targeting already-completed migrations.

## 🧱 Changes made (SRP)
- `docs/architecture-consolidation-plan.md`: Updated line counts for pressure point files, removed `crates/tokmd-cockpit/src/gates.rs` from pressure points and "First Suggested PRs", and added a note that Batch A is largely complete.

## 🧪 Verification receipts
```text
{"command": "wc -l crates/tokmd-cockpit/src/gates.rs", "outcome": "success"}
{"command": "cargo xtask docs --check && cargo test -p xtask", "outcome": "success"}
{"command": "cargo xtask version-consistency && cargo xtask publish --plan --verbose", "outcome": "success"}
{"command": "cargo fmt -- --check && cargo clippy -- -D warnings", "outcome": "success"}
```

## 🧭 Telemetry
- Change shape: Docs update.
- Blast radius: Internal documentation alignment only.
- Risk class + why: Low risk. Modifies only a markdown file (`docs/architecture-consolidation-plan.md`).
- Rollback: Revert the git commit.
- Gates run: `cargo xtask docs --check`, `cargo test -p xtask`, `cargo xtask version-consistency`, `cargo xtask publish --plan --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
