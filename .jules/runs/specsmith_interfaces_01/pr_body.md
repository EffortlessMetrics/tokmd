## 💡 Summary
Added missing BDD scenarios and determinism tests for the `--children` flag within the `export` and `module` CLI commands.

## 🎯 Why
While reviewing the test coverage within the `interfaces` shard (Specsmith persona), I found that the `tokmd export` and `tokmd module` CLI commands supported the `--children separate` and `--children parents-only` flags, but these paths lacked explicit BDD regression tests for `export` and determinism hardening checks for both `export` and `module` commands. Closing this gap improves our proof of correct behavior around this edge case.

## 🔎 Evidence
The determinism suite in `crates/tokmd/tests/cli_determinism_e2e_w54.rs` checked `--children collapse` and `separate` for `tokmd lang`, but omitted them for `export` and `module`. Similarly, BDD tests for `--children separate` and `parents-only` were missing from `crates/tokmd/tests/bdd_export_scenarios_w50.rs`.
I ran `cargo run --bin tokmd -- export --format json --children separate > a.json` twice and observed differing `generated_at_ms` keys, verifying that determinism isn't guaranteed trivially without the test harness checks running properly.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add BDD test cases for `export --children` combinations and missing determinism tests for `export` and `module` with `--children`.
- why it fits this repo and shard: Directly aligns with the "Specsmith" mission of improving regression and scenario coverage within the CLI interface shard.
- trade-offs: Increases the test suite size slightly (Structure), provides long-term safety (Velocity), aligns with strict testing requirements (Governance).

### Option B
- what it is: Look for edge-cases in other CLI commands (like `init`).
- when to choose it instead: If the `--children` coverage was already complete.
- trade-offs: Leaves a known gap in coverage for the config/CLI boundary on an existing parameter.

## ✅ Decision
Option A. I added the targeted tests to prove correct output behavior and enforce determinism invariants.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/bdd_export_scenarios_w50.rs`
- `crates/tokmd/tests/cli_determinism_e2e_w54.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd --test bdd_export_scenarios_w50
cargo test -p tokmd --test cli_determinism_e2e_w54
```

## 🧭 Telemetry
- Change shape: proof-improvement patch
- Blast radius: tests
- Risk class + why: Low, test-only addition.
- Rollback: Revert the added test files.
- Gates run: `cargo build --verbose`, `CI=true cargo test -p tokmd --verbose`, `cargo fmt -- --check`, `cargo clippy -p tokmd -- -D warnings`.

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces_01/envelope.json`
- `.jules/runs/specsmith_interfaces_01/decision.md`
- `.jules/runs/specsmith_interfaces_01/receipts.jsonl`
- `.jules/runs/specsmith_interfaces_01/result.json`
- `.jules/runs/specsmith_interfaces_01/pr_body.md`

## 🔜 Follow-ups
None.
