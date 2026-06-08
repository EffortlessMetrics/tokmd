# Decision

## Option A (recommended)
- Modify `redact_path` in `crates/tokmd-format/src/redact/mod.rs` to normalize the safe extensions to lowercase before hashing the path.
- This ensures that trust boundary data leaks related to original file casing are prevented, as paths differing only by case in their extension will yield the same hash/redaction.
- Why it fits this repo and shard: It closes a concrete missed-mutant-style gap regarding path case-normalization for redaction output, strictly within the `tokmd-format` core pipeline, directly preventing information leakage via extension casing.
- Trade-offs:
  - Structure: Requires a slight modification in path string manipulation inside `redact_path`.
  - Velocity: Simple logic addition.
  - Governance: Adds strict, test-backed proof against case leakage.

## Option B
- Modify the `safe_path_extension_suffix` helper in `crates/tokmd-format/src/redact/extensions.rs` to return an explicit list of the lower-cased extensions, and build the hash from that.
- When to choose it instead: If the responsibility of normalizing extension cases should lie entirely on the extension registry rather than the path string logic.
- Trade-offs: Still requires modification of `clean_path` or the hashed string, which operates on the raw path before `safe_path_extension_suffix` is mapped. Doing the logic purely in `redact_path` before hashing is simpler and directly addresses the hash-input problem.

## ✅ Decision
Option A is selected because it tackles the hash-input problem exactly where the string is being prepared for hashing within `redact_path`. We will adjust the path to have a lowercased extension before passing it to `short_hash`.
