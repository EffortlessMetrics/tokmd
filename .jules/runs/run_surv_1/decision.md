## Option A (recommended)
Use workspace dependencies for internal crates like `tokmd-scan`, `tokmd-model`, and `tokmd-format` in the `dev-dependencies` sections of other internal crates. Currently, crates like `tokmd-types`, `tokmd-analysis-types`, and `tokmd-scan` specify their dev-dependencies on other internal crates using path and version references like `{ path = "../tokmd-scan", version = ">=1.9, <2" }`. Since these are workspace members, they should be configured to use `workspace = true` to maintain a consistent structure and single source of truth for versioning, preventing dependency drift and aligning with how other workspace members (e.g. `tokmd-settings`, `tokmd-io-port`) are referenced.

- Why it fits this repo and shard: This directly improves workspace dependency hygiene and consistency, ensuring we have a coherent, well-structured workspace that doesn't define versions inline when a workspace property already does it. It targets dependency hygiene across crate boundaries.
- Trade-offs: Structure is improved significantly. Velocity is barely impacted but future bumps are easier. Governance is respected by maintaining version-consistency.

## Option B
Do not touch the dev-dependencies and instead look for other feature flag inconsistencies or crate layerings.

- When to choose it instead: If the path dependencies are strictly required by some external publishing requirement that workspace dependencies cannot fulfill, though Cargo publish usually rewrites workspace dependencies to actual versions.
- Trade-offs: Leaves the current inconsistency in place, which goes against workspace-wide hygiene goals.

## Decision
Choosing **Option A**. The workspace already uses `workspace.dependencies` for all these internal crates. They should simply reference `{ workspace = true }` rather than hardcoding `path` and `version`.
