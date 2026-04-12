## 💡 Summary
This is a learning PR. I investigated replacing `.then(...)` with `.then_with(|| ...)` in sorting operations in the core pipeline tests, but determined that eager vs lazy evaluation of pure `.cmp()` operations does not affect determinism. No patch is required.

## 🎯 Why
A memory guideline suggested chaining `.then_with(|| ...)` for BTreeMap sorting to guarantee deterministic tie-breaking. However, because `.cmp()` has no side effects, the existing `.then(...)` usages are already deterministic. Faking a fix for this would violate the output honesty rule.

## 🔎 Evidence
- `crates/tokmd-model/tests/proptest_deep.rs`
- Investigated `sorted1.sort_by(|a, b| b.code.cmp(&a.code).then(a.lang.cmp(&b.lang)));`
- `cargo test -p tokmd-model` passes perfectly as is.

## 🧭 Options considered
### Option A
- Replace `.then(...)` with `.then_with(|| ...)`.
- This would just be a minor performance optimization, not a determinism fix. Claiming it fixes a determinism hazard would be hallucinated work.
- Trade-offs: Minor speed bump for test compilation, but breaks honesty rule.

### Option B (recommended)
- what it is: Do not change the code. Output a learning PR.
- when to choose it instead: When investigation proves the target code is already sound and the hypothesis of drift is incorrect.
- trade-offs: Structure/Honesty / Governance: We stay honest to the pipeline constraints and do not force fake patches.

## ✅ Decision
Option B. We will document a learning PR instead of faking a determinism fix.

## 🧱 Changes made (SRP)
- (None - Learning PR)

## 🧪 Verification receipts
```text
{"cmd": "grep -r -n \"\\.sort_by\" crates/tokmd-types crates/tokmd-scan crates/tokmd-model crates/tokmd-format crates/tokmd/tests", "exit_code": 0}
{"cmd": "grep -n -B 2 -A 2 \"\\.then(\" crates/tokmd-model/tests/proptest_deep.rs", "exit_code": 0}
{"cmd": "cargo test -p tokmd-model", "exit_code": 0}
{"cmd": "cargo clippy -p tokmd-model -- -D warnings", "exit_code": 0}
```

## 🧭 Telemetry
- Change shape: Learning PR (no code change)
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): None
- Risk class + why: None
- Rollback: N/A
- Gates run: `cargo test -p tokmd-model`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism_1/envelope.json`
- `.jules/runs/gatekeeper_determinism_1/decision.md`
- `.jules/runs/gatekeeper_determinism_1/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism_1/result.json`
- `.jules/runs/gatekeeper_determinism_1/pr_body.md`
- Added friction item: `.jules/friction/open/determinism_then_cmp.md`

## 🔜 Follow-ups
- Address the friction item regarding the memory rule that suggests `.then(...)` introduces flakiness for pure comparisons.
