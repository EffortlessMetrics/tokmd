## Options

### Option A
Fix the test failure in `xtask_deep_w74.rs` by adding `".jules/runs"` to the allowed runtime paths checked by `gate.rs`'s `TRACKED_AGENT_RUNTIME_PATHS`, but explicitly making `cargo xtask gate` allow untracked `.jules/runs` files when checking, or checking the test expectations to verify how `gate.rs` should handle `.jules/runs`.

Actually, `xtask/tests/xtask_deep_w74.rs` expects `gate.rs` to contain `".jules/runs"`:
```rust
fn gate_runtime_guard_keeps_curated_jules_deps_history() {
    let src = read_source("xtask/src/tasks/gate.rs");
    assert!(
        src.contains("\".jules/runs\""),
        "gate should treat root .jules/runs as runtime state"
    );
    assert!(
        src.contains("Curated `.jules/deps/**` history is allowed"),
        "gate should document the curated .jules/deps allowance"
    );
}
```
But `gate.rs` currently does *not* contain `".jules/runs"` in `TRACKED_AGENT_RUNTIME_PATHS` as seen previously.

Let's modify `xtask/src/tasks/gate.rs` to add `".jules/runs"` to `TRACKED_AGENT_RUNTIME_PATHS`.
This satisfies the `gate_runtime_guard_keeps_curated_jules_deps_history` test and correctly protects the gate from tracked agent runtime state.

### Option B
Remove the test.
But Option A is clearly what the contract intends - checking that agent generated runs are protected from accidental commit by gating them.

## Decision
Proceeding with Option A: adding `".jules/runs"` to `TRACKED_AGENT_RUNTIME_PATHS` in `xtask/src/tasks/gate.rs` to satisfy the missing guard rail and restore the test suite.
