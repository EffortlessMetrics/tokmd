# Contributing to tokmd

Thank you for your interest in contributing to `tokmd`! This project aims to be a robust, pipeline-friendly repository inventory tool.

## Development Setup

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

## Testing

We prioritize deterministic outputs. This is critical because `tokmd` is used to generate receipts that must be stable over time.

-   **Run Unit Tests**:
    ```bash
    cargo test
    ```
-   **Integration / Golden Tests**:
    We use `insta` for snapshot testing. If you change the output format, the tests will fail with a diff.
    To review and accept changes:
    ```bash
    cargo install cargo-insta
    cargo test
    cargo insta review
    ```
    This guarantees that `tokmd` outputs (receipts) remain deterministic and stable.

-   **Manual Verification**:
    Use the `alias-tok` feature to run the `tok` binary during dev if you prefer short commands:
    ```bash
    cargo run --features alias-tok --bin tok -- help
    ```

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
