## Option A (recommended)
Remove the unused `serde_json` dependency from `tokmd-types/Cargo.toml`'s main dependencies and keep it only in `dev-dependencies`. Also remove it from the workspace's default build config. `serde_json` is only used inside `#[cfg(test)]` blocks in `crates/tokmd-types/src/lib.rs` and `crates/tokmd-types/src/cockpit.rs`, as well as in tests. Removing this dependency from the main crate reduces the compile surface and dependency closure for any crates that depend on `tokmd-types` without needing JSON serialization themselves.

Trade-offs:
- Structure: Improves dependency hygiene.
- Velocity: Faster builds for downstream crates.
- Governance: Aligns with the core mission of the Auditor persona.

## Option B
Keep `serde_json` as a main dependency but feature-gate it.

When to choose it instead:
If downstream crates using `tokmd-types` legitimately need `serde_json` traits/impls exposed through `tokmd-types`, we could add a `json` feature.

Trade-offs:
- Adds complexity to the Cargo.toml.
- `tokmd-types` doesn't currently expose any JSON-specific APIs, so a feature gate is unnecessary and overcomplicates the hygiene fix.

## Decision
Option A. `serde_json` is not used in the production code of `tokmd-types`. Removing it from the main `dependencies` section and keeping it in `dev-dependencies` is the cleanest and most direct way to improve dependency hygiene for this Tier 1 crate.
