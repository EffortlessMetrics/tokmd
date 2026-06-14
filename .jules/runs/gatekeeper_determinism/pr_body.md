## 💡 Summary
Added golden snapshot tests for the `tokmd diff` command in JSON and Markdown formats. This locks in the diff output contract and protects against format regressions.

## 🎯 Why
While `lang`, `module`, `export`, and `analyze` outputs were protected by `insta` snapshot tests, the `diff` command lacked snapshot coverage. Adding it ensures that refactoring the diff algorithm or serialization structures won't unintentionally break the deterministic contract for downstream consumers.

## 🔎 Evidence
- Added `snapshot_diff_json` and `snapshot_diff_markdown` to `crates/tokmd/tests/cli_snapshot_golden.rs` using `tempfile::tempdir()` to avoid fixture pollution.
- Run `UPDATE_EXPECT=1 cargo test -p tokmd --test cli_snapshot_golden` successfully accepted and stored the new `.snap` files.

## 🧭 Options considered
### Option A (recommended)
Add snapshot tests in `cli_snapshot_golden.rs` that generate receipt inputs natively and diff them, applying specific regex normalizations.
- **Structure**: High-signal integration test that ensures CLI fidelity.
- **Velocity**: Easy to maintain with `cargo insta review`.
- **Governance**: Complies with the Gatekeeper mandate for determinism.

### Option B
Unit test the format serialization deeper in `tokmd-format` without CLI flags.
- Less coverage of the actual user/system interface, missing CLI format wrappers.

## ✅ Decision
Option A. It closes a concrete snapshot gap on an important CLI command while aligning with the determinism mandate of this run.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_snapshot_golden.rs`: Added diff generation tests.
- `crates/tokmd/tests/snapshots/cli_snapshot_golden__diff_json.snap`: Golden JSON snapshot.
- `crates/tokmd/tests/snapshots/cli_snapshot_golden__diff_markdown.snap`: Golden Markdown snapshot.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd --test cli_snapshot_golden
running 16 tests
test snapshot_analyze_json ... ok
test snapshot_diff_json ... ok
test snapshot_diff_markdown ... ok
...
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
```

## 🧭 Telemetry
- Change shape: New snapshot tests
- Blast radius: None (Test-only change)
- Risk class: Low - Improves coverage
- Rollback: Revert the PR
- Gates run: contracts-determinism

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`

## 🔜 Follow-ups
None.
