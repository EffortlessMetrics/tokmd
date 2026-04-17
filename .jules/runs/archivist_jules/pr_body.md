## 💡 Summary
Updated `.jules/bin/build_index.py` to aggregate per-run packets natively from historical ledgers in `.jules/docs/` and `.jules/quality/`. This ensures the generated `RUNS_ROLLUP.md` provides a complete view of all historical runs without mutating the original history.

## 🎯 Why
Memory constraints indicate historical run packets and ledgers in version control must be preserved as primary truth and never deleted or rewritten. The aggregation tooling `build_index.py` was missing the historical runs in `.jules/docs/` and `.jules/quality/`. Updating the tool brings the rollup index in sync with all historical ledgers.

## 🔎 Evidence
- File path: `.jules/bin/build_index.py`
- Observed behavior: `RUNS_ROLLUP.md` was missing historical quality and docs runs.
- Execution: Running `python3 .jules/bin/build_index.py` now includes all 4 available runs.

## 🧭 Options considered
### Option A (recommended)
- Parse `.jules/docs/` and `.jules/quality/` ledgers natively inside `build_index.py`.
- This fits the Archivist persona's mission to consolidate run packets while respecting the invariant to preserve historical run data unmodified.
- Trade-offs: Requires custom ledger parsing for each historical directory, but perfectly preserves governance history.

### Option B
- Migrate all historical runs into `.jules/runs/`.
- Choose this if we wanted a single source of truth format, but rewriting historical runs breaks governance rules.
- Trade-offs: Violates the constraint that historical packets must not be modified or moved.

## ✅ Decision
Selected **Option A** to update `build_index.py` to natively aggregate the existing legacy formats. This maintains the untracked `.jules/runs/` invariant while providing an accurate global rollup.

## 🧱 Changes made (SRP)
- `.jules/bin/build_index.py`

## 🧪 Verification receipts
```text
{"cmd": "python3 .jules/bin/build_index.py", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Internal Scaffolding
- Blast radius: Internal documentation index generation. No code impact.
- Risk class: Low
- Rollback: Revert script changes
- Gates run: python3 .jules/bin/build_index.py, cargo xtask docs --check

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
