# Decision

## Context
The goal is to improve Jules by consolidating run packets, friction, learnings, and shared scaffolding. The target ranking is:
1. consolidate recurring friction themes into better templates/policy/docs
2. summarize per-run packets into generated indexes/rollups
3. clean up prompt/runtime documentation
4. move neutral conventions into shared guidance

I have found an issue with the script `build_index.py` that summarizes per-run packets. It only processes live runs under `.jules/runs/` and completely ignores historical ledger runs from `.jules/docs/` and `.jules/quality/`. As per the prompt and run rules, historical ledgers and their associated run packets are valid sources of information, and the `RUNS_ROLLUP.md` generated index is missing data from them.

## Options Considered

### Option A: Update `build_index.py` to aggregate historical ledgers alongside live runs (Recommended)
- **What it is:** Modify `.jules/bin/build_index.py` to iterate over both the live runs in `.jules/runs/` and the historical json ledgers/envelopes in `.jules/docs/` and `.jules/quality/`. This merges all knowledge into a single generated `RUNS_ROLLUP.md` file.
- **Why it fits:** It directly addresses Target #2 ("summarize per-run packets into generated indexes/rollups") and fixes the fragmentation where historical runs are hidden from the top-level index.
- **Trade-offs:**
  - **Structure:** Improves data consolidation.
  - **Velocity:** No runtime overhead for bots, just a one-time structural fix.
  - **Governance:** Keeps primary truth files (ledgers/envelopes) intact, but generates a better rollup.

### Option B: Migrate historical ledgers into live runs
- **What it is:** Write a script to convert the contents of `.jules/docs/` and `.jules/quality/` to new directories under `.jules/runs/`, deleting the old ledgers.
- **When to choose it instead:** If maintaining backward compatibility with the old ledger structure is deemed unnecessary.
- **Trade-offs:** Modifying/deleting historical primary truth files is explicitly forbidden ("never delete or rewrite historical run packets").

## Decision
I have chosen **Option A**. The memory guidelines state: "Historical run packets/ledgers already tracked in version control (e.g., in `.jules/docs/` or `.jules/quality/`) must be preserved as primary truth and never deleted or rewritten; tooling like `build_index.py` should be updated to aggregate them natively instead." This makes Option A the only fully compliant and correct approach.

I have updated `.jules/bin/build_index.py` to natively aggregate historical and live run data into `RUNS_ROLLUP.md`.
