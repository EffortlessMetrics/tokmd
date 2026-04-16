## 💡 Summary
Added missing golden snapshot test coverage for the CLI `analyze` subcommand. Implemented normalization for dynamic properties like `base_signature` to lock in deterministic test outputs for the JSON analysis receipt contract.

## 🎯 Why
The CLI `analyze` subcommand outputs detailed JSON receipts. Without golden snapshot tests, changes to the core `tokmd-analysis` pipeline or output schema could silently drift. By testing `analyze --preset estimate` and normalizing its dynamic output, we strengthen regression coverage for these contract-bearing outputs and prevent future silent drifts.

## 🔎 Evidence
- `crates/tokmd/tests/cli_snapshot_golden.rs` lacked tests for the `analyze` subcommand.
- `analyze` outputs include dynamic values such as `base_signature` which varies per run.
- `cargo test -p tokmd --test cli_snapshot_golden` showed 12 tests passing, missing analysis integration.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add a missing snapshot golden test for `tokmd analyze --preset estimate` and implement regex-based normalization for `base_signature`.
- why it fits this repo and shard: Directly targets the `core-pipeline` shard testing suite to prevent snapshot/schema drift, fitting the Gatekeeper persona.
- trade-offs: Structure: Extends existing testing structure smoothly. Velocity: Quick verification. Governance: Strong contract protection.

### Option B
- what it is: Try to refactor internal data structures to be deterministically hashable across environments without adding the snapshot test directly.
- when to choose it instead: If the problem was purely internal ordering rather than missing observable contracts.
- trade-offs: Fails to provide concrete, PR-reviewable proof of the contract's shape.

## ✅ Decision
Option A was chosen. Adding the snapshot test directly locks in the external contract and fulfills the Gatekeeper's mandate to protect deterministic output.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_snapshot_golden.rs`: Added `snapshot_analyze_estimate_json` test and regex normalization for `base_signature`.
- `crates/tokmd/tests/snapshots/cli_snapshot_golden__analyze_estimate_json.snap`: Initialized the accepted golden snapshot.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test cli_snapshot_golden --features analysis
...
test snapshot_export_jsonl ... ok
test snapshot_export_json ... ok
test snapshot_export_csv ... ok
test snapshot_analyze_estimate_json ... ok
test snapshot_lang_json_structure ... ok
test snapshot_lang_markdown ... ok
...
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
```

## 🧭 Telemetry
- Change shape: New snapshot test + test fixture update
- Blast radius: `crates/tokmd/tests/` (testing only)
- Risk class + why: Low. Modifies only the test suite.
- Rollback: Revert the PR.
- Gates run: `cargo test --all-features`, `cargo insta review`

## 🗂️ .jules artifacts
- `.jules/runs/run-gatekeeper/envelope.json`
- `.jules/runs/run-gatekeeper/decision.md`
- `.jules/runs/run-gatekeeper/receipts.jsonl`
- `.jules/runs/run-gatekeeper/result.json`
- `.jules/runs/run-gatekeeper/pr_body.md`

## 🔜 Follow-ups
None at this time.
