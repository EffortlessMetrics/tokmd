## 💡 Summary
Added missing golden snapshot tests for the `tokmd analyze` subcommand in `cli_snapshot_golden.rs`. This closes a regression gap and prevents silent drift in the `analyze` output format.

## 🎯 Why
The `tokmd analyze` command produces deeply enriched receipts and is a major contract-bearing surface. While `lang`, `module`, and `export` already had snapshot coverage, `analyze` was notably missing from `cli_snapshot_golden.rs`, leaving its Markdown and JSON console outputs vulnerable to silent drift. This PR locks in its contract using deterministic normalizations.

## 🔎 Evidence
- `crates/tokmd/tests/cli_snapshot_golden.rs`
- Missing `snapshot_analyze_json` and `snapshot_analyze_markdown` tests.
- Extracted and normalized dynamic fields (`generated_at_ms`, `target_path`, `base_signature`).

## 🧭 Options considered
### Option A (recommended)
- Add snapshot tests for `analyze` with proper regex normalizations.
- Closes the regression gap on output contract directly within the snapshot golden testing framework.
- Structure / Velocity / Governance: High alignment with deterministic output expectations and gate profiles.

### Option B
- Add new test gate checks to `xtask version-consistency` for overall constant schemas.
- Does not fix the missing output snapshot tests for a major subcommand.

## ✅ Decision
Option A. I implemented `snapshot_analyze_markdown` and `snapshot_analyze_json` in `crates/tokmd/tests/cli_snapshot_golden.rs`, properly feature-gating them and configuring regex for dynamic values to ensure reproducible test runs.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/cli_snapshot_golden.rs`: Added `snapshot_analyze_markdown` and `snapshot_analyze_json`. Fixed whitespace parsing in `generated_at_ms` normalization and added normalizations for `target_path` and `base_signature`.
- Generated `crates/tokmd/tests/snapshots/cli_snapshot_golden__analyze_json.snap` and `crates/tokmd/tests/snapshots/cli_snapshot_golden__analyze_markdown.snap`.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test cli_snapshot_golden --all-features
...
test snapshot_analyze_json ... ok
test snapshot_analyze_markdown ... ok
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🧭 Telemetry
- Change shape: Proof-improvement patch
- Blast radius: Tests only.
- Risk class: Low - Test-only addition.
- Rollback: Revert the PR.
- Gates run: `cargo test -p tokmd --test cli_snapshot_golden --all-features`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
