# Bolt ⚡ Core Pipeline Decision

## Options Considered

### Option A: Remove String allocations in path normalization
- **What it is**: Optimize `normalize_slashes` and `normalize_rel_path` in `crates/tokmd-scan/src/path/mod.rs` to return `Cow<'_, str>` instead of unconditionally allocating a `String` by calling `.into_owned()` and `.to_string()`. Callers that actually need a `String` can allocate, but many callers just use it to check or compare paths and do not need to take ownership.
- **Why it fits this repo and shard**: Path normalization happens across the pipeline (scan, ignore matching, filtering). Reducing allocations here provides a broad, systemic performance lift in the hot path.
- **Trade-offs**:
  - *Structure*: We change the return type to `std::borrow::Cow<'_, str>`, which requires callers to adjust slightly if they depend on an owned `String`. Since the function is public, this touches the API.

### Option B: Investigate inner allocations in `walk` or model iteration
- **What it is**: Profile or search the `crates/tokmd-scan/src/walk` or model aggregation for hot allocations.
- **When to choose it**: If path normalization is not proven to be hot, or if changing its API causes too much friction.
- **Trade-offs**: Might be harder to isolate into one simple "Refactorer" reviewer story compared to a well-known string-allocation target.

## ✅ Decision
Option A.

Repeated formatting and allocation during path normalization (especially slash normalization) is a classic "unnecessary allocations" target in Rust. In many cases, paths don't contain backslashes and `./` is already stripped, so `normalize_slashes` and `normalize_rel_path` could return `Cow<'a, str>` without allocating. The current code has a `normalize_slashes_cow` helper but `normalize_slashes` wraps it to unconditionally return `String`.

Changing `normalize_slashes` and `normalize_rel_path` to return `Cow<'_, str>` prevents allocations in the fast path where no replacement is needed.
