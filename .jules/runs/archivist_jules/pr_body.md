## 💡 Summary
Updated the `cargo xtask jules-index` generator to include closed friction items. It now collects from both `.jules/friction/open/` and `.jules/friction/done/` to ensure resolved items are preserved in the generated index.

## 🎯 Why
Previously, when a friction item was marked as done and moved from `open/` to `done/`, it was removed entirely from the generated `FRICTION_ROLLUP.md`. This update preserves historical context and learnings by rolling up both active and closed friction items into the single index.

## 🔎 Evidence
- file path(s): `xtask/src/tasks/jules_index.rs`, `.jules/index/generated/FRICTION_ROLLUP.md`
- observed behavior / finding: Before this change, running `cargo xtask jules-index` only parsed `.jules/friction/open/`. Now it parses both and includes closed items like `fuzz_toolchain_blocker` and `steward-release-clean-state` in the markdown table.
- command: `cargo xtask jules-index && cat .jules/index/generated/FRICTION_ROLLUP.md`

## 🧭 Options considered
### Option A (recommended)
- what it is: Modify `xtask/src/tasks/jules_index.rs` to collect friction items from both `.jules/friction/open/` and `.jules/friction/done/`, combine the vectors, and sort them.
- why it fits this repo and shard: It improves Jules itself by consolidating learnings. The index generation is workspace-wide tooling governance. It keeps all friction history in a single, searchable place.
- trade-offs: Structure - The rollup will grow larger, but the table format remains readable. Velocity - Simple and safe structural change. Governance - Provides complete visibility into resolved issues.

### Option B
- what it is: Add a new command `xtask jules-index-done` to generate a separate `.jules/index/generated/FRICTION_DONE_ROLLUP.md`.
- when to choose it instead: If the active friction index must strictly remain small and only show open items.
- trade-offs: Adds duplicate logic and multiple files to look at when searching history.

## ✅ Decision
Option A was chosen. I modified `write_or_check_friction_rollup` to collect friction items from both `.jules/friction/open` and `.jules/friction/done`, combine them, and sort them so they all appear in `FRICTION_ROLLUP.md`. I also updated the generator's header text to reflect this ("active and closed friction metadata").

## 🧱 Changes made (SRP)
- `xtask/src/tasks/jules_index.rs`: Updated `write_or_check_friction_rollup` and `render_friction_rollup`.
- `.jules/index/generated/FRICTION_ROLLUP.md`: Regenerated the index to include closed friction items.

## 🧪 Verification receipts
```text
cargo build --verbose -p xtask
cargo test -p xtask
cargo fmt -- --check
cargo clippy -- -D warnings
cargo xtask jules-index
cargo xtask version-consistency
cargo xtask publish --plan --verbose
cargo xtask docs --check
```

## 🧭 Telemetry
- Change shape: Feature (Jules internal tooling)
- Blast radius: Internal tooling only
- Risk class: Low
- Rollback: Revert the PR
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`, `cargo xtask jules-index`, `cargo xtask version-consistency`, `cargo xtask publish`, `cargo xtask docs`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
