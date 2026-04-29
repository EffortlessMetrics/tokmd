# Decision

## Option A: Add Corpus Seeds for Existing Targets (Recommended)
Add corpus directories for `fuzz_scan_args` and `fuzz_toml_config`. This is a low-risk proof-improvement that supports the fuzz targets. However, `cargo-fuzz` is not available, so we will not be able to run these targets.

## Option B: Document friction regarding `cargo-fuzz`
Since `cargo-fuzz` is unavailable and fails to run (as expected based on memory/knowledge), we document this as a friction item.

**Decision:** We will combine these approaches by initializing corpus directories to make future fuzzing easier, and documenting the missing `cargo-fuzz` tool as a friction item, leading to a learning PR.
