# Release Process

This document describes how to release a new version of `tokmd`.

## Prerequisites

-   Ensure `Cargo.toml` version is updated.
-   Ensure `CHANGELOG.md` (if present) or release notes are ready.
-   Ensure CI passes on `main`.

## Steps

1.  **Bump Version**:
    Update `version` in `Cargo.toml`.
    ```bash
    # e.g. to 0.2.0
    sed -i 's/^version = ".*"/version = "0.2.0"/' Cargo.toml
    ```

2.  **Commit & Tag**:
    ```bash
    git commit -am "chore: release v0.2.0"
    git tag v0.2.0
    git push origin main --tags
    ```

3.  **Publish to Crates.io**:
    ```bash
    cargo publish
    ```

4.  **GitHub Release**:
    Create a new release on GitHub using the tag. Attach the binary if needed (though Crates.io is the primary distribution channel).
