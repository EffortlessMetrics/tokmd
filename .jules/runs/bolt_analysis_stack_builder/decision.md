## Options considered
### Option A (recommended)
- **What it is**: Avoid redundant UTF-8 validation and string allocation using `from_utf8` directly.
  The code currently checks if byte arrays read from files are valid text using `is_text_like`, which calls `std::str::from_utf8`. Right after that, the code unconditionally uses `String::from_utf8_lossy(&bytes)`, which allocates a new `String` or `Cow::Owned` for valid utf8 and performs the utf8 checking again. Since we already proved the bytes are valid utf8 (as `is_text_like` returns `true` only if `std::str::from_utf8(bytes).is_ok()` and has no null bytes), we can convert `bytes` to a `&str` directly via `std::str::from_utf8(&bytes).unwrap()`.
  This improves the hot paths in `api_surface`, `halstead`, `content`, and `complexity` analyzers, reducing repeated parsing and unnecessary allocations.
- **Why it fits this repo and shard**: The shard is `analysis-stack` and persona is `Bolt ⚡`. We need to optimize for "unnecessary allocations / cloning / string building" and "repeated parsing/formatting that can be reused". Changing `String::from_utf8_lossy(&bytes)` (which yields `Cow<str>` and validates UTF-8) to `from_utf8(&bytes)` removes redundant UTF-8 validation passes across all scanned files.
- **Trade-offs**:
  - Structure: slightly more boilerplate match blocks.
  - Velocity: minimal changes required.
  - Governance: minimal risk, retains same deterministic behavior.

### Option B
- **What it is**: Cache `String` reading.
- **When to choose it instead**: If files were mostly already parsed as `String` in IO buffers.
- **Trade-offs**: The initial check `is_text_like` uses bytes anyway to safely detect binary files without allocation, so reading as `String` directly would lose this protection.

## Decision
Option A. I have implemented a match block over `std::str::from_utf8(&bytes)` instead of calling `String::from_utf8_lossy` on paths that already checked `is_text_like` or similar. I've also implemented `read_text_capped` to not re-allocate `String` through `from_utf8_lossy` if it's already valid UTF-8.
