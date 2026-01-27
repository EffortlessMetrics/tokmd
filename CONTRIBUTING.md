# Contributing to tokmd

Thank you for your interest in contributing to `tokmd`! This project aims to be a robust, pipeline-friendly repository inventory tool.

## Development Setup

### Nix (recommended)
1.  **Enter the dev shell**:
    ```bash
    nix develop
    ```
2.  **Build**:
    ```bash
    cargo build
    ```

### Manual (non-Nix)
1.  **Rust Toolchain**: Ensure you have a recent stable Rust toolchain installed.
    ```bash
    rustup update stable
    ```
2.  **Clone & Build**:
    ```bash
    git clone https://github.com/EffortlessMetrics/tokmd.git
    cd tokmd
    cargo build
    ```

## Testing Strategy

We prioritize deterministic outputs. This is critical because `tokmd` is used to generate receipts that must be stable over time.

### 1. Unit Tests
Run standard unit tests for model logic and utility functions:
```bash
cargo test
```

### 2. Integration / Golden Tests
We use `insta` for snapshot testing. These tests run the full CLI against fixtures in `tests/data`.

**Important: Line Endings & Bytes**
Our receipts include byte counts which are sensitive to line endings. To ensure cross-platform determinism:
*   We enforce `text eol=lf` in `.gitattributes` for `tests/data` and `tests/snapshots`.
*   **Always accept snapshots from an LF-normalized checkout** (Linux/WSL/macOS or a normalized Windows git checkout).
*   If you see byte count diffs (e.g., `183` vs `172`), check your line endings.

**If you change output logic (e.g., new fields, different formatting):**
1.  Run tests: `cargo test` (they will fail with a diff).
2.  Review changes: `cargo insta review` (requires `cargo-insta` installed).
3.  Accept changes if they are intentional.

This guarantees that `tokmd` outputs (receipts) remain deterministic and stable.

## Code Style

-   Run `cargo fmt` before committing.
-   Run `cargo clippy` to catch common issues.

## Pull Requests

1.  Open an issue to discuss major changes first.
2.  Ensure your PR includes relevant tests.
3.  Update documentation if you change CLI behavior or flags.
4.  Reference the Milestone your PR addresses (see `ROADMAP.md`).

## Receipt Schema

`tokmd` treats outputs as "receipts". If you modify the JSON output structure:
1.  Ensure `schema_version` is handled correctly.
2.  Update the struct definitions in `src/model.rs` and `src/format.rs`.
3.  Update the formal schema in `docs/schema.json`.
4.  Run tests to verify backward compatibility if possible.
