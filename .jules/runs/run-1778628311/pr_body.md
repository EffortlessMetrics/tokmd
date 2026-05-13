## 💡 Summary
Added BDD scenario coverage for `Health` and `Deep` presets in `analysis_deep_w64.rs` and fixed missing proof mapping for `analysis_deep_w64.rs`.

## 🎯 Why
Currently, only the `Receipt` preset has BDD-style behavioral proofs in `analysis_deep_w64.rs`. `Health` and `Deep` behavior edge cases were not locked in with scenario-style tests.
Also, the newly written test file `analysis_deep_w64.rs` wasn't tracked by the `analysis_orchestration` proof scope in `ci/proof.toml` resulting in CI unmapped files errors.

## 🔎 Evidence
- `crates/tokmd-analysis/tests/analysis_deep_w64.rs`
- BDD tests specifically for `Receipt` existed but omitted `Health` and `Deep`.
- CI reported `crates/tokmd-analysis/tests/analysis_deep_w64.rs` as unmapped during execution artifact verification.

## 🧭 Options considered
### Option A (recommended)
- Expand BDD scenario tests for missing analysis presets (Health, Deep) in `analysis_deep_w64.rs`.
- Include `crates/tokmd-analysis/tests/analysis_*.rs` in the `analysis_orchestration` scope of `ci/proof.toml`.
- Fits the "proof-improvement patch" allowance perfectly without polluting actual app logic.
- Trade-offs: Structure / Velocity / Governance - adds integration test proof overhead but enforces correctness.

### Option B
- Fix feature gates silent skipping in `feature_gates_w71.rs`.
- Might stray into unrelated testing logic instead of BDD scenario logic.

## ✅ Decision
Option A. It aligns perfectly with Specsmith (BDD/integration coverage) and explicitly provides a proof-improvement patch.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/tests/analysis_deep_w64.rs`
- `ci/proof.toml`

## 🧪 Verification receipts
```text
$ CI=true cargo xtask proof --profile affected --base origin/main --head HEAD --executor-mode execute --executor-max-commands 2 --allow-ci-evidence-execution --executor-summary target/proof/executor-summary.json --executor-manifest target/proof/executor-manifest.json
...
Proof execution observation collection OK: 1 observation(s), 1 scope(s), wrote `target/proof/proof-executor-observation-collection.json`, `target/proof/proof-executor-observation-collection.md`
```

## 🧭 Telemetry
- Change shape: proof-improvement patch
- Blast radius: minimal, only tests and ci config
- Risk class: very low, just integration tests
- Rollback: git checkout crates/tokmd-analysis/tests/analysis_deep_w64.rs ci/proof.toml
- Gates run: cargo build, cargo test, clippy, fmt, xtask proof

## 🗂️ .jules artifacts
- `.jules/runs/.../envelope.json`
- `.jules/runs/.../decision.md`
- `.jules/runs/.../receipts.jsonl`
- `.jules/runs/.../result.json`
- `.jules/runs/.../pr_body.md`

## 🔜 Follow-ups
None
