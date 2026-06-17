# Decision

## Options considered
### Option A (recommended)
Update the `clean_path` function in `crates/tokmd-format/src/redact/mod.rs` to better resolve parent `..` segments.
This is necessary to guarantee that logically identical paths produce deterministic hashes without leaking directory structure logic when parent directories cancel out child segments. It aligns perfectly with the core-pipeline and mutating proving expectations for robust formatting/path handling.
Trade-offs: Structure is improved, velocity is neutral, governance rules are honored for test determinism.

### Option B
Only address `cargo clippy` warnings and rely on `.to_string().replace("..", ...)` directly without a stack logic in tests.
When to choose it instead: If the impact of parent directory resolution is minor and simple replacement scales.
Trade-offs: Slower execution when traversing trees, and fails edge cases like `crates/foo/../../bar`.

## Decision
Option A. A robust stack-based normalization in `clean_path` is proven to solve directory structure leaks and correctly collapses path sequences without breaking cross-platform path resolution, which passes all unit tests for `tokmd-format` formatting functions and fixes path redaction leakage.
