## 💡 Summary
Added `".jules/runs"` to `TRACKED_AGENT_RUNTIME_PATHS` in `xtask` gate checks. This fixes a failing regression test and properly gatekeeps run artifacts to prevent them from being committed as runtime state.

## 🎯 Why
The test `gate_runtime_guard_keeps_curated_jules_deps_history` checks that the `gate` command will enforce that `.jules/runs` files are not unintentionally tracked. This was failing because `.jules/runs` was missing from the check arrays.

## 🔎 Evidence
The test output explicitly complained about the missing `".jules/runs"` text in the `gate.rs` source file:
```
thread 'gate_runtime_guard_keeps_curated_jules_deps_history' panicked at xtask/tests/xtask_deep_w74.rs:358:5:
gate should treat root .jules/runs as runtime state
```

## 🧭 Options considered
### Option A (recommended)
- Add `".jules/runs"` to `TRACKED_AGENT_RUNTIME_PATHS` in `xtask/src/tasks/gate.rs`.
- This respects the deterministic output requirements, explicitly preventing per-run artifacts from being incorrectly committed while properly passing the contract test.
- Trade-offs: Simple and directly aligned with the governance requirements for the `xtask` shard.

### Option B
- Change the test to ignore `.jules/runs`.
- When to choose it instead: If run packets were meant to be committed to the repo history.
- Trade-offs: This contradicts governance documents which dictate run packets should stay local or be explicitly managed differently, and violates the explicit requirement for checking tracked agent state.

## ✅ Decision
Option A. Added `".jules/runs"` to the tracked paths check to ensure the gate prevents accidental commits of agent runtime state and passes its regression tests.

## 🧱 Changes made (SRP)
- `xtask/src/tasks/gate.rs`: Appended `".jules/runs"` to `TRACKED_AGENT_RUNTIME_PATHS`.

## 🧪 Verification receipts
```text
cargo test -p xtask
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.33s
```

## 🧭 Telemetry
- Change shape: Minor array update
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): IO/Governance tool
- Risk class + why: Low, only affects development `xtask gate` check.
- Rollback: Revert array addition.
- Gates run: `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
