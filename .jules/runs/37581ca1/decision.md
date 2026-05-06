## 🧭 Options considered

### Option A: Fix binary search by sorting `SAFE_PATH_EXTENSIONS` (recommended)
- **What it is:** Sort the `SAFE_PATH_EXTENSIONS` array alphabetically in `crates/tokmd-format/src/redact/mod.rs` so that `binary_search` succeeds for all items.
- **Why it fits this repo and shard:** The array was already hardcoded as a `const &[&str]`, making it highly efficient for binary searches if sorted. Sorting the array fixes the logical bug with zero runtime overhead or dependency additions, fitting perfectly within the `core-pipeline` shard's boundaries for zero-cost abstraction and redaction correctness.
- **Trade-offs:**
  - **Structure:** Keeps existing architecture completely intact.
  - **Velocity:** Extremely fast and minimal patch.
  - **Governance:** Easy to review, minimal risk.

### Option B: Switch from `binary_search` to a `HashSet` or `phf::Set`
- **What it is:** Replace the sorted slice with a `HashSet` (using `lazy_static`/`OnceLock`) or a perfect hash function set (`phf` crate).
- **When to choose it instead:** When the allowlist is incredibly large, or when the cost of maintaining a sorted array manually is deemed too high by the team.
- **Trade-offs:** Requires introducing additional macro dependencies or initialization overhead. Slower than a simple `binary_search` over ~70 items. Overcomplicates a simple allowlist.

## ✅ Decision
I chose **Option A**. The issue was simply that `md` and `svelte` (and possibly others) were missed because the array was out of alphabetical order. Sorting it properly and adding strict unit tests to ensure it stays sorted resolves the bug while keeping the implementation highly performant and allocation-free.
