# Decision

## Option A
Modify `.jules/bin/build_index.py` to also process `.jules/friction/open/` and generate `.jules/index/generated/FRICTION_ROLLUP.md`.
- Why it fits: Matches memory explicitly.
- Trade-offs: Valid fix, but discovered to be superseded by PR #1606.

## Option B
Gracefully abort the redundant fix and create a learning PR.
- Why it fits: Follows the prompt-to-PR pipeline requirements for superseded work. Captures the friction item.
- Trade-offs: Abandons the code patch in favor of acknowledging shared reality.

## ✅ Decision
Option B. The intended patch was superseded by PR #1606. Proceeding with a learning PR and friction item to document the workflow edge case.
