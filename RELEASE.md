# Release Process

This repository uses a "Microcrates / Ecosystem" publishing model.
All crates are versioned in lockstep (same version number).

## Publishing Order

Because crates have internal dependencies, they must be published in this specific order:

1.  **`tokmd-types`** (Tier 1: Data Contract)
2.  **`tokmd-config`**
3.  **`tokmd-model`** (Tier 2: Aggregation Logic)
4.  **`tokmd-format`**
5.  **`tokmd-scan`**
6.  **`tokmd-tokeignore`**
7.  **`tokmd`** (CLI Binary)

## Steps to Release

1.  **Bump Versions**:
    Update `[workspace.package].version` in the root `Cargo.toml`.
    (All crates inherit this version).

2.  **Commit & Tag**:
    ```bash
    git commit -am "chore: release v1.0.x"
    git tag v1.0.x
    git push && git push --tags
    ```

3.  **Publish**:
    You can use a tool like `cargo-workspaces` or publish manually:

    ```bash
    # Dry run everything first
    cargo publish -p tokmd-types --dry-run
    # ... etc

    # Publish in order
    cargo publish -p tokmd-types
    cargo publish -p tokmd-config
    cargo publish -p tokmd-model
    cargo publish -p tokmd-format
    cargo publish -p tokmd-scan
    cargo publish -p tokmd-tokeignore
    cargo publish -p tokmd
    ```

## Verification

Before releasing, ensure:
*   `cargo test --workspace` passes.
*   `cargo clippy --workspace` is clean.
*   The release profile in root `Cargo.toml` is active.
