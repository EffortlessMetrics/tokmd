# Decision

## Context
The Archivist persona focuses on improving Jules itself by consolidating learnings and sharing scaffolding. Target ranking #2 is "summarize per-run packets into generated indexes/rollups".

Looking at the generated indexes (`.jules/index/generated/RUNS_ROLLUP.md`), there are runs with "Unknown" styles/personas. Some are simply missing an `envelope.json` file. Some of them such as `auditor_bindings_manifests` and `compat_interfaces_matrix_01` are not even in the `.jules/runs/` folder, which suggests they might be leftover lines from a bad rollback or generation failure. But other runs such as `d657338a-caa9-4ccf-93a1-4733ada7154c` have "Unknown" set for style. Same for `run_perf_cockpit_entry` which didn't have an envelope.

## Options considered

### Option A: Clean up missing/malformed envelope files and regenerate the indexes (Recommended)
1. Fix the metadata frontmatter in the `d657338a-caa9-4ccf-93a1-4733ada7154c/envelope.json`.
2. Generate `envelope.json` for `run_perf_cockpit_entry` based on PR body information.
3. Remove non-existent runs from the `RUNS_ROLLUP.md` generated index or just let `xtask jules-index` handle it.
4. Run `cargo xtask jules-index` to update the generated `RUNS_ROLLUP.md` file.

- **Structure**: High. Brings disparate runs into compliance.
- **Velocity**: Low impact on product code velocity.
- **Governance**: High. The generated indexes will correctly track run statuses.

### Option B: Leave as is
1. Do nothing.

- **Structure**: Low.
- **Velocity**: Low.
- **Governance**: Low.

## Decision
**Option A**.
