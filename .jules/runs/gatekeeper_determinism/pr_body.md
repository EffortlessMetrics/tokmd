## 💡 Summary
This is a learning PR. I discovered a failing test in the `xtask` crate caused by an out-of-sync `scope_count` assertion, but because the target is outside the `core-pipeline` shard, I have recorded it as friction instead of fixing it.

## 🎯 Why
The determinism gate validation expectations failed during execution (`cargo test -p xtask` fails). The `proof_policy_json_reports_current_schema` test in `xtask` hardcoded the expected number of scopes to `36`, but the actual `ci/proof.toml` file contains `37` scopes. Because `xtask/tests/` is outside of the `core-pipeline` allowed paths, I am adhering to the strict anti-chasing rules.

## 🔎 Evidence
- File path: `xtask/tests/proof_policy_w90.rs`
- Observed behavior: `assert_eq!(value["scope_count"], 36);` failed because the actual count was 37.
- Command demonstrating it: `cargo test -p xtask --test proof_policy_w90`

## 🧭 Options considered
### Option A (recommended)
- Record the out-of-bounds target as a friction item.
- Why it fits: Adheres strictly to the prompt's instructions: "If the strongest target you find is outside the shard, record it as friction instead of chasing it."
- Trade-offs: Structure over Velocity. The test remains broken until an agent with the correct shard assignment can fix it.

### Option B
- Fix the `scope_count` in `xtask/tests/proof_policy_w90.rs`.
- When to choose it instead: If the prompt allowed touching any file to fix a determinism gate failure.
- Trade-offs: Velocity over Structure. Violates shard boundaries.

## ✅ Decision
Option A was chosen to respect shard boundaries and the explicit instructions against chasing out-of-shard targets.

## 🧱 Changes made (SRP)
- Recorded a friction item for the `xtask` test failure.

## 🧪 Verification receipts
```text
cargo test -p xtask --test proof_policy_w90

running 5 tests
test proof_policy_declares_coverage_executor_promotion_rule ... ok
test proof_policy_includes_current_product_scopes ... ok
test proof_policy_check_accepts_repo_policy ... ok
test xtask_help_mentions_proof_policy ... ok
test proof_policy_json_reports_current_schema ... FAILED

failures:

---- proof_policy_json_reports_current_schema stdout ----
thread 'proof_policy_json_reports_current_schema' panicked at xtask/tests/proof_policy_w90.rs:191:5:
assertion `left == right` failed
  left: Number(37)
 right: 36
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: `.jules` artifacts only
- Risk class: None
- Rollback: N/A
- Gates run: `cargo xtask docs --check`, `cargo deny --all-features check`, `cargo xtask version-consistency`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`
- `.jules/friction/open/xtask_proof_policy_out_of_sync.md`

## 🔜 Follow-ups
- Address the `xtask_proof_policy_out_of_sync.md` friction item in a run that has the `xtask` shard assigned.
