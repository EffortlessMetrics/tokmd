# Decision

## Option A (recommended)
Update `normalize_path` in `tokmd-model`, `normalize_slashes` and `normalize_rel_path` in `tokmd-scan` to use `std::borrow::Cow` instead of `String`.
- **Why it fits**: This change directly targets "unnecessary allocations / string building" on the hot path. These functions are heavily called during directory traversal and data rollups, often returning the exact same string (without backslashes). Using `Cow` allows returning a borrowed view of the input string, avoiding unnecessary string allocations.
- **Trade-offs**: Slightly more complex function signature / body, but entirely hidden behind the API. Velocity and Governance are unaffected. High performance win (~65% faster on `normalize_slashes`, ~22-38% on `normalize_path`, ~47% on `normalize_rel_path`).

## Option B
Do not optimize the path normalization functions.
- **When to choose it**: If the complexity cost of `Cow` was high and the functions weren't on the hot path.
- **Trade-offs**: Keeps code slightly simpler but leaves performance on the table in a core data pipeline.

## Decision
**Option A**. The benchmarks clearly show a significant performance win by avoiding allocations for paths that don't need backslash conversion. Given the high volume of path normalizations during a full repo scan, this is a clear "hot-path work reduction" and "unnecessary allocations" win.
