## 🧭 Options considered

### Option A (recommended)
- Run `cargo xtask jules-index` to regenerate the `.jules/index/generated/RUNS_ROLLUP.md` and related index files, which are currently drifting because new run metadata exists in `.jules/runs/`.
- Commit the result as a standard patch update.
- Fit for the Archivist persona's mission to "summarize per-run packets into generated indexes/rollups".

### Option B
- Ignore the drift and create a learning PR documenting that it exists.
- This is not recommended because the drift is easily fixable with an existing tool and aligns perfectly with the target ranking: "summarize per-run packets into generated indexes/rollups".

## ✅ Decision
Option A. This is the exact task expected of the Archivist persona, the drift is easily verifiable with `cargo xtask jules-index --check`, and fixable with `cargo xtask jules-index`. This provides a clean, coherent story that improves Jules's indexing.
