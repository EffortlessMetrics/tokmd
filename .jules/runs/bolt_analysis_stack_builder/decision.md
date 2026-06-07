# Decision

## Option A (recommended)
**What it is:** Reduce unnecessary String allocations and clones inside analysis hot paths. Specifically:
- `git/mod.rs` and `git/freshness.rs`: Use `&str` instead of `String` for map keys (avoiding string allocations per commit file).
- `topics/mod.rs`: Maintain `overall_tf` inside the main pass rather than merging dynamically, avoiding extra string allocations/clones.

**Why it fits:** The analysis stack operates on a high volume of file rows and git commits. Unnecessary string allocations within loops (e.g. `String::clone()`) contribute heavily to GC / allocator pressure. Using reference keys where possible yields deterministic and safe memory footprint reductions without behavior changes.

**Trade-offs:**
- Structure: Mildly tighter lifetimes are required (e.g. `BTreeMap<&str, i64>`).
- Velocity: Trivial to verify via existing test suite.
- Governance: Follows standard Rust performance patterns.

## Option B
**What it is:** Restructure derived metrics parallelization with Rayon or crossbeam.

**Why not:** It violates the goal of starting with explicit structural reduction (allocations) vs risking determinism drift with concurrency, as parallel analysis outputs can become non-deterministic if not carefully sequenced.

## ✅ Decision
Option A. It's safe, fully structural, relies heavily on eliminating `.clone()` for `String`s in high loop iterations, and passes all existing determinism test suites while delivering explicit alloc reductions.
