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
7.  **`tokmd-core`** (Façade)
8.  **`tokmd`** (CLI Binary)

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
    Use the included automation script:

    ```powershell
    # Dry run
    ./scripts/publish-all.ps1 -DryRun

    # Real release
    ./scripts/publish-all.ps1
    ```

## Verification

Before releasing, ensure:
*   `cargo test --workspace` passes.
*   `cargo clippy --workspace` is clean.
*   The release profile in root `Cargo.toml` is active.
