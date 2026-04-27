# Decision

## Option A (recommended)
Update the `TRACKED_AGENT_RUNTIME_PATHS` array in `xtask/src/tasks/gate.rs` to include `".jules/runs"`. This fixes the failing test `gate_runtime_guard_keeps_curated_jules_deps_history` and fulfills the deterministic output requirement of preventing per-run artifacts from being incorrectly committed when they shouldn't be (or correctly gatekeeping agent runtime state).
- **Fits shard**: "tooling-governance" includes `xtask/**`.
- **Trade-offs**: Simple fix directly targeted to the failing contract test for gatekeeping agent runtime state.

## Option B
Update the test to not expect `".jules/runs"` in `xtask/src/tasks/gate.rs`.
- **When to choose**: If `".jules/runs"` is actually meant to be committed.
- **Trade-offs**: But memory explicitly states: "The `.jules/runs` directory is ignored by `.gitignore`. While `cargo xtask gate` checks may fail if per-run artifacts are staged, the `request_code_review` MCP tool strictly requires these artifacts to be present in the git diff to validate PR packet completeness." Wait, if `.jules/runs` is ignored, should it be checked by `gate`? The memory says: "The `cargo xtask gate` command checks for tracked agent runtime state and will fail if any files under `.jules/runs/` are staged in the git index. Always run `cargo xtask gate` *before* running `git add -f` to stage your final per-run artifacts."
This strongly implies `.jules/runs` SHOULD be in the gate check, and the test is correct that the gate should enforce it. So Option A is the right one.

## Decision
Option A. I will add `".jules/runs"` to `TRACKED_AGENT_RUNTIME_PATHS` in `xtask/src/tasks/gate.rs` to fix the test and correctly gate agent runtime state.
