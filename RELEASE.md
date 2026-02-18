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

## Verification

Before releasing, ensure:
- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo xtask publish --dry-run` passes end-to-end.
