# Steward Release Drift

## Inspected
- `cargo xtask version-consistency`
- `cargo xtask publish --plan --verbose`
- `git grep "version =" Cargo.toml crates/*/Cargo.toml`
- `.jules/policy/agent_profiles.json`
- `CHANGELOG.md`

## Observation
Running `cargo xtask version-consistency` passes, but memory states:
> In the `tokmd` project, `cargo xtask version-consistency` only checks dependencies listed in the `[workspace.dependencies]` section of the root `Cargo.toml`. It does not detect version drift for hardcoded inline path dependencies inside individual crate `Cargo.toml` files.

Checking inline dependencies via `git grep "version =" crates/*/Cargo.toml | grep path` reveals several crates (such as `tokmd-analysis-types`, `tokmd-cockpit`, `tokmd-envelope`, `tokmd-scan`, `tokmd-types`, and `tokmd-wasm`) still rely on inline version constraints (like `>=1.9, <2` and `1.11.0`) instead of workspace inheritance. This is a release publish-plan metadata mismatch risk because they can silently drift.

## Options considered
### Option A (recommended)
Fix the inline path dependencies for internal workspace crates to use workspace inheritance (`.workspace = true`).
- What it is: Replace instances of `version = "..."` alongside `path = "..."` for internal dependencies with `workspace = true`.
- Why it fits: It aligns with the rest of the workspace, properly fixes the version consistency drift vulnerability, and ensures all internal crates are published with synchronized versions.
- Trade-offs: Structure / Velocity / Governance - Better governance and structure, no downside since it uses a standard Cargo workspace feature.

### Option B
Manually update the hardcoded versions to `1.13.1`.
- What it is: Replace `version = ">=1.9, <2"` and `version = "1.11.0"` with `version = "1.13.1"`.
- When to choose: If workspace inheritance was deliberately avoided.
- Trade-offs: It requires manual updates for every version bump which goes against automated release tools.

## Decision
Option A. I will use workspace inheritance for these internal dependencies to fix the consistency check blind spot and align with standard internal conventions.
