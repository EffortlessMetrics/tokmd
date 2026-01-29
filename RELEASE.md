# Release Process

This repository uses a "Microcrates / Ecosystem" publishing model.
All crates are versioned in lockstep (same version number).

## Publishing Order

Because crates have internal dependencies, they must be published in this specific order:

**Tier 0 — Data Structures** (no internal deps)
1.  **`tokmd-types`**
2.  **`tokmd-analysis-types`**

**Tier 1 — Core Logic**
3.  **`tokmd-scan`**
4.  **`tokmd-model`**
5.  **`tokmd-tokeignore`**
6.  **`tokmd-redact`**

**Tier 2 — I/O & Analysis**
7.  **`tokmd-format`**
8.  **`tokmd-walk`**
9.  **`tokmd-content`**
10. **`tokmd-git`**

**Tier 3 — Enrichment**
11. **`tokmd-analysis`**
12. **`tokmd-analysis-format`**
13. **`tokmd-fun`**

**Tier 4 — Orchestration**
14. **`tokmd-config`**
15. **`tokmd-core`**

**Tier 5 — CLI**
16. **`tokmd`**

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
