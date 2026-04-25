## Options

### Option A: Consolidate legacy ledgers into `.jules/runs/`
- **What it is**: Move legacy per-run artifacts located in `.jules/docs/` and `.jules/quality/` to the modern `.jules/runs/` structure, create standardized `envelope.json`, `result.json` and `pr_body.md` based on legacy content, remove the old directories and ledgers (`ledger.json`), and re-run `.jules/bin/build_index.py`.
- **Why it fits this repo and shard**: Memory dictates: "In the `.jules` workspace, per-run packets must be strictly contained within `.jules/runs/<run-id>/`. The script `.jules/bin/build_index.py` relies solely on this directory to generate the central index (`.jules/index/generated/RUNS_ROLLUP.md`). Legacy ledger locations like `.jules/docs/` or `.jules/quality/` are deprecated and should not be used."
- **Trade-offs**:
  - *Structure*: Greatly improves consistency and deprecates duplicate ledger models.
  - *Velocity*: Simplifies indexing for future agents.
  - *Governance*: Conforms precisely to memory instructions.

### Option B: Just update `build_index.py` to scrape legacy ledgers
- **What it is**: Leave `.jules/docs/` and `.jules/quality/` in place but modify `.jules/bin/build_index.py` to parse their `ledger.json` and `.md` files to generate `RUNS_ROLLUP.md`.
- **When to choose it instead**: If the legacy format is still actively used or needed for other tools.
- **Trade-offs**: Keeps the repo cluttered with deprecated formats and violates the memory instruction that explicitly forbids using legacy ledger locations.

## Decision
**Option A**. It strictly follows the memory directive to move everything into `.jules/runs/<run-id>/` and deprecate legacy ledger locations.
