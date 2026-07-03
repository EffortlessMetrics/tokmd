## Option A / Option B
### Option A
Fix the factual doc drift in `docs/SCHEMA.md`. `docs/SCHEMA.md` incorrectly claims `BASELINE_VERSION` is defined in `crates/tokmd-analysis-types/src/lib.rs`. It is actually defined in `crates/tokmd-analysis-types/src/baseline.rs`.

### Option B
Do nothing and emit a learning PR.

### Decision
Option A. It's a clear factual doc drift within the `tooling-governance` shard.
