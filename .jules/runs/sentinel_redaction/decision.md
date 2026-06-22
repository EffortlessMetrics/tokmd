## Options Considered

### Option A (recommended)
- **What it is:** Update `clean_path` in `tokmd-format/src/redact/mod.rs` to fully parse and resolve `..` and `.` path segments instead of just doing simple string replacements. This completely hardens the trust boundary by ensuring path traversals (`a/../b`) produce the same deterministic hash as the canonical path (`b`).
- **Why it fits:** The redaction logic in the formatting pipeline hashes paths to protect the system's directory structure while outputting metrics. If directory traversals are passed without normalization, the resulting hash can leak details since `a/../b` hashes differently than `b`, and a bad actor might reconstruct directory trees.
- **Trade-offs:**
    - Structure: We add a small path segment resolution algorithm into `clean_path`, which handles segments correctly and ensures stability across inputs.
    - Velocity: We modify `clean_path` and add a new test, taking a few minutes to ensure it resolves things safely without using the slower runtime `std::path::Path::components`.
    - Governance: Complies fully with the Gatekeeper/Sentinel rule of deterministic safety and protecting trust boundaries.

### Option B
- **What it is:** Just use `std::path::PathBuf::canonicalize()`.
- **When to choose it instead:** When the files exist on disk locally.
- **Trade-offs:** Canonicalization does I/O, fails if files do not exist, and depends heavily on the OS/filesystem. The `tokmd-format` crate formats purely logical paths which may not exist or might originate from memory buffers. We cannot use `fs::canonicalize`.

## Decision
Going with Option A. `clean_path` now splits paths by `/` and evaluates segments properly to prevent directory traversal leakage.
