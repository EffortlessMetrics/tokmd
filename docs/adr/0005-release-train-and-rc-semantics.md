# ADR-0005: Release train and RC semantics

- Status: accepted
- Date: 2026-04-29

## Context

`v1.10.0-rc.1` surfaced risk around prerelease tags, moving major aliases, and stable-channel publication behavior. Release train semantics need explicit durable policy.

## Decision

- Package metadata uses semver prerelease format for release candidates (example: `1.10.0-rc.1`).
- Git tags use the matching hyphenated prerelease form (example: `v1.10.0-rc.1`).
- Non-hyphenated RC tags such as `v1.10.0rc1` are not release-train tags unless automation is explicitly updated to classify them as prereleases.

RC releases:

- are prereleases
- are not latest
- do not move `v1`
- do not publish stable Docker aliases
- skip crates.io publication unless explicitly approved

Stable releases:

- may move `v1`
- may publish crates.io artifacts
- may publish stable Docker aliases (including semver/latest rules)

## Consequences

- Prevents RC channel leakage into stable consumption paths.
- Makes alias/tag behavior predictable across GitHub and Docker ecosystems.
- Reduces accidental prerelease promotion risk.

## Alternatives

- Treat RC and stable release trains identically.
- Allow RC tags to update stable aliases by default.

Both alternatives were rejected due to high distribution risk.

## Enforcement

- Release automation must enforce RC/stable behavior differences.
- Version and release docs must match tag and package semantics.

## Related specs

- `docs/publish-surface.md`
