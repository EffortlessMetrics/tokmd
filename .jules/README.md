# .jules/

State lives on disk. "If it isn't written, it didn't happen."

## Directory Structure
- `policy/`: Configuration for automated tasks.
- `runbooks/`: Templates for PRs and friction items.
- `friction/`: Queue of work items (open/done).
- `bolt/`: Performance engineering state.
  - `runs/`: Logs of execution runs.
  - `envelopes/`: Detailed structured data for runs.
  - `ledger.json`: Append-only index of runs.
  - `notes/`: Reusable patterns.
