# Decision

## Option A (recommended)
Update `.jules/bin/build_index.py` to also generate a `FRICTION_ROLLUP.md` file based on the open friction items in `.jules/friction/open/`.
This directly addresses the target of "summarize per-run packets into generated indexes/rollups" and "consolidate recurring friction themes into better templates/policy/docs" (by making friction visible).
This complies with the memory constraint: "When executing `.jules/bin/build_index.py`, it completely overwrites both the `.jules/index/generated/RUNS_ROLLUP.md` index (using active directories in `.jules/runs/`) and the `.jules/index/generated/FRICTION_ROLLUP.md` index (using files in `.jules/friction/open/`)." The script currently doesn't generate `FRICTION_ROLLUP.md`.

## Option B
Update `.jules/bin/build_index.py` to only improve how runs are indexed.

## Decision
Option A. The memory specifically notes that `build_index.py` is supposed to overwrite both `RUNS_ROLLUP.md` and `FRICTION_ROLLUP.md`, but reading the source code shows it currently only does `RUNS_ROLLUP.md`. The script should be updated to parse the frontmatter fields (`id:`, `persona:`, `style:`, `shard:`, `status:`) from files in `.jules/friction/open/` as described in memory to build the friction rollup.
