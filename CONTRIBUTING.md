# Contributing to tokmd

Thank you for your interest in contributing to `tokmd`! This project aims to be a robust code intelligence platform for humans, machines, and LLMs.

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

## Project Structure

The codebase uses a tiered microcrate architecture:

```
crates/
├── tokmd-types/           # Tier 0: Core data structures
├── tokmd-analysis-types/  # Tier 0: Analysis receipt types
├── tokmd-scan/            # Tier 1: tokei wrapper
├── tokmd-model/           # Tier 1: Aggregation logic
├── tokmd-tokeignore/      # Tier 1: Template generation
├── tokmd-redact/          # Tier 1: BLAKE3-based path redaction
├── tokmd-format/          # Tier 2: Output rendering
├── tokmd-walk/            # Tier 2: File system traversal
├── tokmd-content/         # Tier 2: Content scanning
├── tokmd-git/             # Tier 2: Git analysis
├── tokmd-analysis/        # Tier 3: Analysis orchestration
├── tokmd-analysis-format/ # Tier 3: Analysis output
├── tokmd-fun/             # Tier 3: Novelty outputs
├── tokmd-config/          # Tier 4: Configuration
├── tokmd-core/            # Tier 4: Library facade
└── tokmd/                 # Tier 5: CLI binary
```

## Testing Strategy

We prioritize deterministic outputs. This is critical because `tokmd` is used to generate receipts that must be stable over time.

### 1. Unit Tests
Run standard unit tests for model logic and utility functions:
```bash
cargo test
```

### 2. Integration / Golden Tests
We use `insta` for snapshot testing. These tests run the full CLI against fixtures.

**Important: Line Endings & Bytes**
Our receipts include byte counts which are sensitive to line endings. To ensure cross-platform determinism:
*   We enforce `text eol=lf` in `.gitattributes` for test fixtures and snapshots.
*   **Always accept snapshots from an LF-normalized checkout** (Linux/WSL/macOS or a normalized Windows git checkout).
*   If you see byte count diffs (e.g., `183` vs `172`), check your line endings.

**If you change output logic (e.g., new fields, different formatting):**
1.  Run tests: `cargo test` (they will fail with a diff).
2.  Review changes: `cargo insta review` (requires `cargo-insta` installed).
3.  Accept changes if they are intentional.

This guarantees that `tokmd` outputs (receipts) remain deterministic and stable.

### 3. Crate-Level Tests
Each crate may have its own tests in a `tests/` directory. Run all tests with:
```bash
cargo test --workspace
```

## Code Style

-   Run `cargo fmt` before committing.
-   Run `cargo clippy -- -D warnings` to catch common issues.

## Contribution Areas

### Priority Areas

1. **Enricher implementations** — Add new analysis enrichers in `crates/tokmd-analysis/src/`:
   - Look at existing enrichers like `derived.rs` or `git.rs` for patterns
   - Add new modules and wire them into `analysis.rs`

2. **Output format templates** — Improve Markdown/SVG rendering in `crates/tokmd-analysis-format/`

3. **Language support** — Extend import graph parsing for more languages

4. **Documentation** — Recipe examples, use cases, and tutorials

### Adding a New Enricher

1. Create a new module in `crates/tokmd-analysis/src/` (e.g., `my_enricher.rs`)
2. Add the data structures to `crates/tokmd-analysis-types/src/lib.rs`
3. Wire it into `AnalysisReceipt` and the preset system in `analysis.rs`
4. Add formatting support in `crates/tokmd-analysis-format/`
5. Add tests and update documentation

## Pull Requests

1.  Open an issue to discuss major changes first.
2.  Ensure your PR includes relevant tests.
3.  Update documentation if you change CLI behavior or flags.
4.  Reference the relevant section in `ROADMAP.md`.

## Receipt Schema

`tokmd` treats outputs as "receipts". If you modify the JSON output structure:

### For core receipts (lang, module, export):
1.  Update struct definitions in `tokmd-types` or `tokmd-model`.
2.  Update formatting in `tokmd-format`.
3.  Update the formal schema in `docs/schema.json`.
4.  Increment `schema_version` for breaking changes.

### For analysis receipts:
1.  Update struct definitions in `tokmd-analysis-types`.
2.  Update formatting in `tokmd-analysis-format`.
3.  Update `docs/SCHEMA.md` documentation.
4.  Increment `ANALYSIS_SCHEMA_VERSION` for breaking changes.

## Feature Flags

Some features are gated to allow selective compilation:
- `git`: Git history analysis (shells out to `git` command)
- `content`: File content scanning (entropy, TODOs, duplicates)
- `walk`: Filesystem traversal for assets

When adding new features with heavy dependencies, consider making them optional.

## Publishing to crates.io

Publishing is handled via `cargo xtask publish`, which ensures correct dependency order, validates packaging, and handles propagation delays.

### Workflow

```bash
# 1. Review the publish plan
cargo xtask publish --plan --verbose

# 2. Validate packaging (runs cargo publish --dry-run for each crate)
cargo xtask publish --dry-run

# 3. Publish for real (requires confirmation)
cargo xtask publish --yes

# 4. Publish and create git tag
cargo xtask publish --yes --tag
```

### Pre-publish checks

The xtask runs these checks before publishing:
- Clean git working directory
- Version consistency across all crates
- CHANGELOG.md contains the version
- All tests pass

Skip individual checks with `--skip-git-check`, `--skip-version-check`, `--skip-changelog-check`, `--skip-tests`, or all with `--skip-checks`.

### Resuming after failure

If publishing fails partway through:
```bash
cargo xtask publish --from tokmd-format --yes
```

### Justfile shortcuts

```bash
just publish-plan   # cargo xtask publish --plan --verbose
just publish-dry    # cargo xtask publish --dry-run
just publish        # cargo xtask publish --yes
just publish-tag    # cargo xtask publish --yes --tag
```

## Language Bindings (Planned)

We're building native FFI bindings for Python and Node.js:

```
crates/
├── tokmd-ffi/      # C-compatible FFI layer (shared)
├── tokmd-python/   # PyO3 bindings → PyPI
└── tokmd-node/     # napi-rs bindings → npm
```

**Design principles:**
- JSON serialization at FFI boundary for simplicity
- Mirror the CLI's mental model (`scan`, `analyze`, `diff`)
- Return native language types (Python dicts, JS objects)
- Cross-platform wheels/prebuilds via CI matrix

If you're interested in helping with bindings, see the `tokmd-ffi` crate (once created) for the shared interface.
