## 💡 Summary
Added missing BDD scenario coverage for `tokmd module` using the `--children parents-only` argument. This locks in the behavior to ensure it properly propagates and is serialized to the JSON `args` output.

## 🎯 Why
The `module` command correctly accepts `--children separate` and `--children parents-only`, while the `lang` command only supports `separate` and `collapse`. The `separate` behavior had BDD test coverage in `bdd_module_scenarios_w50.rs`, but `parents-only` did not, leaving a gap where we could break argument parsing or mode propagation without failing the test suite.

## 🔎 Evidence
- `crates/tokmd/tests/bdd_module_scenarios_w50.rs` lacked a test for `parents-only`.
- `crates/tokmd-config/src/cli_enums.rs` explicitly defines `parents-only` as a valid CLI child inclusion mode.
- Running `cargo run --bin tokmd -- module --format json --children parents-only` succeeds and outputs `{"args": { "children": "parents-only" }}`.

## 🧭 Options considered
### Option A (recommended)
- Add the missing BDD test in `crates/tokmd/tests/bdd_module_scenarios_w50.rs`.
- **Trade-offs:**
  - Structure: Improves coverage for CLI argument routing.
  - Velocity: Fast, locks in existing behavior without large refactoring.
  - Governance: Ensures CLI outputs align with their declared enum modes.

### Option B
- Add a learning PR without touching the code.
- **Trade-offs:** Misses a clear opportunity to increase confidence in the interfaces shard coverage.

## ✅ Decision
Option A. It's a clean, safe scenario proof improvement perfectly aligned with the Specsmith persona.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/bdd_module_scenarios_w50.rs`: Added `given_project_when_module_children_parents_only_then_mode_recorded` test.

## 🧪 Verification receipts
```text
cargo test --test bdd_module_scenarios_w50
test given_project_when_module_children_parents_only_then_mode_recorded ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🧭 Telemetry
- Change shape: Add test
- Blast radius: Tests only
- Risk class: Low - pure verification.
- Rollback: Revert the test addition.
- Gates run: cargo test, cargo fmt

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces_001/envelope.json`
- `.jules/runs/specsmith_interfaces_001/decision.md`
- `.jules/runs/specsmith_interfaces_001/receipts.jsonl`
- `.jules/runs/specsmith_interfaces_001/result.json`
- `.jules/runs/specsmith_interfaces_001/pr_body.md`

## 🔜 Follow-ups
None.
