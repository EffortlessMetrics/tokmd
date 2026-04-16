## 💡 Summary
Updated `.jules/bin/build_index.py` to aggregate historical ledgers natively. This consolidates metrics from both standard run packets and older shared ledgers (`.jules/docs/` and `.jules/quality/`) into the generated rollup.

## 🎯 Why
Historical run packets and ledgers tracked in version control must be preserved as primary truth and never deleted or rewritten. To consolidate run metrics safely without rewriting history, the tooling must be updated to aggregate them natively.

## 🔎 Evidence
- Path: `.jules/bin/build_index.py`
- Behavior: Previously only read from `.jules/runs/`. Now it aggregates arrays from `docs/ledger.json` and `quality/ledger.json`.
- Proof: The generated `RUNS_ROLLUP.md` file correctly displays both old historical runs and new standard packets.

## 🧭 Options considered
### Option A (recommended)
- Update `.jules/bin/build_index.py` to aggregate run history natively from existing ledgers (`.jules/docs/ledger.json` and `.jules/quality/ledger.json`) in addition to `.jules/runs/`.
- Why it fits: Preserves historical ledgers as primary truth instead of rewriting history, matching the `Archivist` proof expectations and instructions to summarize/supersede ledgers natively.
- Trade-offs: Centralizes index generation. Provides immediate visibility of old and new run formats. Stops rewriting history while maintaining complete auditability.

### Option B
- Write a script to migrate all historical ledger entries into `.jules/runs/<run-id>/` packets.
- When to choose it instead: If historical formats were explicitly deprecated and required to be destroyed.
- Trade-offs: Violates the directive to never delete or rewrite historical ledger data already tracked in version control.

## ✅ Decision
Selected Option A. Updating `build_index.py` consolidates run metrics securely without altering or erasing source-of-truth ledgers.

## 🧱 Changes made (SRP)
- `.jules/bin/build_index.py`: Added logic to parse and aggregate `.jules/docs/ledger.json` and `.jules/quality/ledger.json`.
- `.jules/index/generated/RUNS_ROLLUP.md`: Regenerated the index to reflect the historical ledgers.

## 🧪 Verification receipts
```text
{"cmd": "python3 .jules/bin/build_index.py", "status": "success", "summary": "Aggregates standard packets, docs ledgers, and quality ledgers into RUNS_ROLLUP.md"}
{"cmd": "cargo xtask docs --check", "status": "success", "summary": "Documentation is up to date"}
{"cmd": "cargo test", "status": "success", "summary": "Tests passed successfully"}
{"cmd": "cargo clippy -- -D warnings", "status": "success", "summary": "No clippy warnings"}
{"cmd": "git restore Cargo.lock", "status": "success", "summary": "Restored lockfile drift caused by cargo tasks"}
```

## 🧭 Telemetry
- Change shape: Tooling update.
- Blast radius: Only affects the generated Jules rollup markdown index. Safe for API / IO / docs / schema / concurrency / compatibility / dependencies.
- Risk class: Low. Internal Jules tooling modification.
- Rollback: `git restore .jules/bin/build_index.py`
- Gates run: `cargo xtask docs --check`, `cargo test`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/run-archivist-001/envelope.json`
- `.jules/runs/run-archivist-001/decision.md`
- `.jules/runs/run-archivist-001/receipts.jsonl`
- `.jules/runs/run-archivist-001/result.json`
- `.jules/runs/run-archivist-001/pr_body.md`

## 🔜 Follow-ups
None.
