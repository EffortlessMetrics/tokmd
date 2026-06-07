# Decision

## Options considered

### Option A (recommended)
- what it is: Move `source_complexity` out of `tokmd-analysis` into `tokmd-cockpit` and remove the dependency.
- why it fits this repo and shard: It resolves a crate boundary layering violation where `tokmd-cockpit` directly reached into `tokmd-analysis` internals for a cockpit-specific heuristic, and completely drops the `tokmd-analysis` crate dependency from `tokmd-cockpit`.
- trade-offs: Improves structural layering and build times without sacrificing functionality.

### Option B
- what it is: Export `source_complexity` from `tokmd_analysis_types`.
- when to choose it instead: If it was truly a shared type definition.
- trade-offs: It violates the Tier 0 definition of `tokmd_analysis_types` which should have no business logic.

## Decision
Option A, as it perfectly fits the Surveyor mission of fixing structural coherence and dependency direction problems.
