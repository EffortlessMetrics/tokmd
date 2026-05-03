## Target: `build_integrity_report` in `crates/tokmd-analysis/src/derived/mod.rs`

### Option A: Remove `format!` allocations in `compare_integrity_rows` (Recommended)
**What it is:** The `compare_integrity_rows` function sorts `FileRow` items for a stable checksum. When two files have the same path but different stats, it currently uses `format!("{}:{}", a.bytes, a.lines)` and string sort to break the tie deterministically. This allocates `String`s inside a hot sorting loop. We replace this with a custom zero-allocation number-to-string format-and-compare routine.
**Why it fits:** Reduces repeated string building / allocations on the hot path in `tokmd-analysis`, exactly as requested by the Bolt persona target ranking #2. Determinism and output remain fully intact.
**Trade-offs:**
- Structure: Replaces simple `format!` with an internal `num_str_cmp_with_colon` algorithm using an explicit digit buffer. Slightly more code but highly structured.
- Velocity: Quick to implement and verify since it only applies to `FileRow` sorting.
- Governance: Reduces memory churn in large repositories, enabling larger scans with the same memory budget.

### Option B: Pre-allocate a single sort key per row
**What it is:** Modify `FileRow` or create a wrapper struct to contain a pre-built sort string or tuple representation during report ingestion, so the sorting logic avoids all dynamic string-building completely.
**Why it fits:** Also removes allocations in the sort loop.
**Trade-offs:** Increases baseline memory footprint for every `FileRow` and requires modifying earlier ingestion phases or allocating intermediate arrays just for sorting. Slower overall pipeline.

## Decision
We chose **Option A**. It solves the dynamic string allocation inside the sort loop locally, avoids increasing the memory size of `FileRow` itself, and requires no global refactoring. We replaced `format!` with a custom fast `num_str_cmp_with_colon` which writes digits iteratively into a fixed `[u8; 45]` stack buffer and performs string comparison via slice equality. This avoids all heap allocations during the sort.
