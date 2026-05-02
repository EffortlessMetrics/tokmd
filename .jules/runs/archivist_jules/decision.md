# Decision

## Option A
Modify `.jules/bin/build_index.py` to also process `.jules/friction/open/` and generate `.jules/index/generated/FRICTION_ROLLUP.md`.
- Why it fits: Matches memory explicitly ("build_index.py completely overwrites both ... RUNS_ROLLUP.md and ... FRICTION_ROLLUP.md") and Archivist's mission to consolidate friction items and generate rollups.
- Trade-offs: Increases the size of `build_index.py`, but it consolidates both indexes into one tool.

## Option B
Create a new tool to process friction items.
- Why it fits: Segregates logic for runs and friction items.
- Trade-offs: More tooling to manage. Memory explicitly says `build_index.py` does this, so option B might go against existing scaffolding.

## ✅ Decision
Option A. It aligns perfectly with the memory constraint that explicitly says `build_index.py` is supposed to completely overwrite the `.jules/index/generated/FRICTION_ROLLUP.md` index using files in `.jules/friction/open/`.
