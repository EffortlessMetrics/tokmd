## 💡 Summary
Added missing golden snapshot coverage for the CLI `analyze` subcommands (`receipt` and `estimate` presets). Expanded the normalizer to ensure metrics and paths are deterministic across environments.

## 🎯 Why
The CLI output format integration tests lacked coverage for `tokmd analyze`, meaning changes to the effort calculations or analysis report structure could accidentally alter the schema without tripping a deterministic check. This change locks in the expected output formats and verifies they remain stable.

## 🔎 Evidence
- Found that `crates/tokmd/tests/cli_snapshot_golden.rs` was missing `snapshot_analyze_*` tests.
- Modified normalizer in `crates/tokmd/tests/cli_snapshot_golden.rs` to scrub dynamic keys like `timestamp_ms`, `target_dir`, and `schedule_months_*`.
- Ran `cargo test -p tokmd --test cli_snapshot_golden` successfully to prove snapshot determinism.

## 🧭 Options considered
### Option A (recommended)
- Add snapshot tests for `tokmd analyze --preset` commands.
- **Why:** Directly addresses snapshot/golden drift and weak coverage for contract-bearing schemas in `core-pipeline`.
- **Trade-offs:** Fast validation against the current stable output, but requires `insta` to accept snapshots manually on legitimate format updates.

### Option B
- Add exhaustive property testing to `tokmd-model` for analysis metrics.
- **When to choose it instead:** When evaluating mathematical formulas and invariant bounds (like cyclomatic constraints) instead of serialization schemas.
- **Trade-offs:** Validates rules but does not catch schema drift in JSON outputs.

## ✅ Decision
Option A was chosen as the most robust way to ensure that changes to analysis results directly fail golden tests if they unintentionally break CLI output schemas.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_snapshot_golden.rs`: Added `snapshot_analyze_receipt_json` and `snapshot_analyze_estimate_json`. Expanded normalization pattern.
- `crates/tokmd/tests/snapshots/`: Automatically accepted snapshots for the new tests. Some existing snapshot formatting spacing for `generated_at_ms` was normalized.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test cli_snapshot_golden
cargo insta accept
```

## 🧭 Telemetry
- **Change shape:** Test expansion
- **Blast radius:** Zero runtime impact. Increases CLI `analyze` formatting safety against regressions.
- **Risk class:** Low. Test only.
- **Rollback:** `git revert`
- **Gates run:** `cargo test -p tokmd --test cli_snapshot_golden`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper-determinism/envelope.json`
- `.jules/runs/gatekeeper-determinism/decision.md`
- `.jules/runs/gatekeeper-determinism/receipts.jsonl`
- `.jules/runs/gatekeeper-determinism/result.json`
- `.jules/runs/gatekeeper-determinism/pr_body.md`

## 🔜 Follow-ups
None
