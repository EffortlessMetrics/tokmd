## 💡 Summary
Tighten `CockpitReceipt` JSON output integration tests to ensure that `test_ratio` is present in the `composition` field as required by the schema.

## 🎯 Why
The `CockpitReceipt` schema (`docs/schema.json` and `docs/SCHEMA.md`) lists `test_ratio` as a required field in the `composition` object. However, `crates/tokmd/tests/cockpit_integration.rs` was not asserting its presence, allowing potential drift where the schema diverges from output.

## 🔎 Evidence
- `docs/schema.json` requires `test_ratio` in `composition`
- `crates/tokmd/tests/cockpit_integration.rs` `test_cockpit_file_classification` asserted `code_pct`, `test_pct`, `docs_pct`, `config_pct` but not `test_ratio`

## 🧭 Options considered
### Option A (recommended)
Tighten missing CockpitReceipt contract tests inside `tokmd/tests/cockpit_integration.rs` by adding checks for `test_ratio` in the composition sections of existing tests.
- **Structure**: High. Matches missing coverage.
- **Velocity**: High. Quick fix.
- **Governance**: High. Improves contract determinism per shard focus.

### Option B
Do an extensive overhaul of schema validations, digging into all `crates/tokmd-types/src` schema definitions.
- **Structure**: Medium. Adds deep tests but could violate SRP.
- **Velocity**: Low. Could be slow and break existing assumptions.
- **Governance**: Medium. Adds complexity.

## ✅ Decision
Option A was chosen to securely patch the missing assertion cleanly.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cockpit_integration.rs`: Added `test_ratio` presence assertion in `test_cockpit_file_classification` test.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test cockpit_integration
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s

cargo xtask proof-policy
Proof policy OK: ci/proof.toml (schema tokmd.proof_policy.v1, 84 scope(s), 1 allowlist(s), 1 fixture blob rule(s), 1 dependency boundary rule(s) ...

cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: test-addition
- Blast radius: API / IO / docs / schema / concurrency / compatibility / dependencies: tests
- Risk class + why: low
- Rollback: git revert
- Gates run:
  - cargo test -p tokmd --test cockpit_integration
  - cargo xtask proof-policy
  - cargo fmt -- --check
  - cargo clippy -- -D warnings

## 🗂️ .jules artifacts
- `.jules/runs/run_gatekeeper_prover_01/envelope.json`
- `.jules/runs/run_gatekeeper_prover_01/decision.md`
- `.jules/runs/run_gatekeeper_prover_01/receipts.jsonl`
- `.jules/runs/run_gatekeeper_prover_01/result.json`
- `.jules/runs/run_gatekeeper_prover_01/pr_body.md`

## 🔜 Follow-ups
None.
