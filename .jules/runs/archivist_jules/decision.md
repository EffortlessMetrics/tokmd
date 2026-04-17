# Decision

## Investigation
The current `.jules/bin/build_index.py` scripts aggregates per-run packets from `.jules/runs/`. However, historical run packets and ledgers in `.jules/docs/` and `.jules/quality/` are not captured. Memory constraints indicate these historical packets must be preserved as primary truth and tooling like `build_index.py` should be updated to aggregate them natively instead.

## Options
### Option A
Update `.jules/bin/build_index.py` to parse `.jules/docs/ledger.json` and `.jules/quality/ledger.json` in addition to `.jules/runs/`. Generate a comprehensive `RUNS_ROLLUP.md` with all runs.

### Option B
Migrate all historical runs into `.jules/runs/`.

## Selection
**Option A** is chosen because `.jules/runs/` is untracked and historical runs must be preserved as primary truth without rewriting them, according to strict rules. Option A native aggregation respects this invariant.
