# Decision

### Option A (recommended)
Remove `tokmd-analysis-types` from `tokmd-wasm`'s `Cargo.toml`, re-export `ANALYSIS_SCHEMA_VERSION` in `tokmd-core`, and update `tokmd-wasm/src/lib.rs` to use the re-exported constant. This tightens the dependency graph and simplifies the manifest for the wasm binding.

### Option B
Leave the direct dependency. No churn, but retains an unnecessary direct dependency declaration in a target-specific binding.

### Decision
Option A. It cleanly satisfies the `deps-hygiene` gate profile by removing a redundant dependency declaration.
