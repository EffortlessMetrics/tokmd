# Decision

## Option A (recommended)
Fix dependency direction and workspace structure issues by moving internal crate references to use `workspace = true` rather than explicit path + version strings where possible. This tightens workspace bounds and prevents versions from drifting within the `tokmd` project. Note that Cargo workspace inheritance does not yet fully support overriding `default-features = false` if the workspace specifies `default-features = true`, so crates like `tokmd-cockpit` requiring this pattern will retain their explicit path dependency.

- Why it fits: We are acting as Surveyor, and dependency direction/workspace structure is exactly within our target ranking. Using explicit versions/paths for internal workspace dependencies instead of relying on workspace resolution is a structural issue.
- Trade-offs:
    - Structure: Better cohesion; everything centralizes to `workspace.dependencies`.
    - Velocity: Faster version bumps.
    - Governance: Less chance of accidentally unlinking a local crate during publish.

## Option B
Do nothing, since explicit paths still technically link crates.
- Trade-offs: Will leave messy dependency duplication and version mismatches.

We will proceed with Option A.
