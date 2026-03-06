# Release Process

This repository uses a lockstep microcrate publishing model.
All publishable workspace crates share the same version.

## Publishing Order

Publish order is derived automatically from workspace dependency topology.
Do not maintain a hard-coded list by hand.

Preview the exact order:

```bash
cargo xtask publish --plan
```

## Steps to Release

1. **Bump version**

```bash
cargo xtask bump <MAJOR.MINOR.PATCH>
```

2. **Update changelog**
- Ensure `CHANGELOG.md` has an entry for the release version.

3. **Commit release changes**

```bash
git commit -am "chore: release vX.Y.Z"
git push
```

4. **Run release preflight**

```bash
cargo xtask publish --dry-run
```

This performs:
- git-clean check
- workspace version consistency check
- changelog version check
- full workspace tests (`--all-features`, excluding `tokmd-fuzz`)
- local package validation (`cargo package --list`) for each publishable crate

5. **Publish to crates.io**

```bash
cargo xtask publish --yes
```

Optional tagging via xtask:

```bash
cargo xtask publish --yes --tag
# or custom format
cargo xtask publish --yes --tag --tag-format "release-{version}"
```

If publishing fails mid-stream, resume from a crate:

```bash
cargo xtask publish --from <crate-name>
```

## Publish Paths

There are two ways to publish a release. Both invoke `cargo xtask publish` under the hood.

### Manual local publish

Run the full publish sequence locally:

```bash
cargo xtask publish --dry-run   # preflight
cargo xtask publish --yes       # publish to crates.io
```

### CI-driven publish (canonical)

Push a semver tag to trigger the release workflow:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

This triggers `.github/workflows/release.yml`, which:
1. Builds cross-platform release binaries
2. Creates a GitHub release with artifacts
3. Runs `cargo xtask publish --yes --skip-tests --verbose` to publish to crates.io

The tag-driven path is the canonical production flow.

## Verification

Before releasing, ensure:
- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo xtask publish --dry-run` passes end-to-end.
