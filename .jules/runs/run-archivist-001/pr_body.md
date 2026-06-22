## 💡 Summary
Updated `cargo xtask jules-index` to parse resolved friction items from `.jules/friction/done/` in addition to `.jules/friction/open/`. Regenerated `FRICTION_ROLLUP.md` to include historical closed items, improving the single pane of glass for all friction metadata.

## 🎯 Why
The Jules indexing system memory requirement specifies that `cargo xtask jules-index` should parse both `.jules/friction/open/` and `.jules/friction/done/` to generate `FRICTION_ROLLUP.md`. Prior to this change, only the `open` directory was being collected. Incorporating closed items helps consolidate run learnings and prevents losing track of resolved friction themes.

## 🔎 Evidence
- Found that `xtask/src/tasks/jules_index.rs` only read from `.jules/friction/open/` (`let friction_dir = root.join(".jules/friction/open");`).
- Found 13 items in `.jules/friction/done/` that were excluded from `FRICTION_ROLLUP.md` by observing `cat .jules/index/generated/FRICTION_ROLLUP.md` missing them.
- Ran `cargo xtask jules-index` after modifying `jules_index.rs` and saw the 13 items appear in `FRICTION_ROLLUP.md` with status `done` and `closed`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Modify `xtask/src/tasks/jules_index.rs` to collect friction items from both `.jules/friction/open/` and `.jules/friction/done/`, and update the `FRICTION_ROLLUP.md` generation logic to reflect this.
- why it fits this repo and shard: It consolidates run learnings into generated indexes, aligning with the Archivist's mission and satisfying the rule that both `open` and `done` directories should be parsed for `FRICTION_ROLLUP.md`.
- trade-offs: Structure/Velocity/Governance - Increases the size of the generated index but ensures complete historical tracking of friction resolution.

### Option B
- what it is: Create a separate DONE_FRICTION_ROLLUP.md specifically for resolved friction.
- when to choose it instead: If the main rollup must strictly be only actionable/open items.
- trade-offs: Duplicates indexing logic and splits historical context.

## ✅ Decision
Option A was chosen as memory explicitly dictates that `FRICTION_ROLLUP.md` should parse both directories, and doing so improves the single pane of glass for all friction items without requiring new artifacts.

## 🧱 Changes made (SRP)
- Modified `xtask/src/tasks/jules_index.rs` to parse both `open/` and `done/` directories.
- Modified `xtask/src/tasks/jules_index.rs` to use directory name for `status` (open or done).
- Regenerated `.jules/index/generated/FRICTION_ROLLUP.md` and `.jules/index/generated/RUNS_ROLLUP.md` via `cargo xtask jules-index`.

## 🧪 Verification receipts
```text
cargo xtask jules-index
Jules indexes written under /app/.jules/index/generated

cargo xtask check-file-policy --strict
file-policy OK: 83 entries, 1155 non-Rust files covered, 1306 Rust files skipped

cargo xtask publish --plan --verbose
Workspace version: 1.13.1
Publish order (16 crates)

cargo xtask version-consistency
Version consistency checks passed

cargo xtask docs --check
Documentation is up to date

cargo test -p xtask
test result: ok. 54 passed; 0 failed; ...
```

## 🧭 Telemetry
- Change shape: Feature patch
- Blast radius: Jules tooling (`xtask/` and `.jules/index/`)
- Risk class: Low - Does not touch product code or platform bindings.
- Rollback: Revert the PR.
- Gates run: `cargo xtask jules-index`, `cargo xtask check-file-policy --strict`, `cargo xtask publish --plan --verbose`, `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test -p xtask`.

## 🗂️ .jules artifacts
- `.jules/runs/run-archivist-001/envelope.json`
- `.jules/runs/run-archivist-001/decision.md`
- `.jules/runs/run-archivist-001/receipts.jsonl`
- `.jules/runs/run-archivist-001/result.json`
- `.jules/runs/run-archivist-001/pr_body.md`

## 🔜 Follow-ups
None
