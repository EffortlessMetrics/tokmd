# Contributing to tokmd

Thank you for your interest in contributing to `tokmd`! This project aims to be a robust code intelligence platform for humans, machines, and LLMs.

Please review our [Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

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

### Optional Local Compiler Cache

For repeated local rebuilds, `sccache` is supported as an opt-in wrapper rather than a repo default.

1.  **Install sccache**:
    ```bash
    winget install Mozilla.sccache
    # or
    cargo install sccache --locked
    ```
2.  **Verify setup**:
    ```bash
    cargo sccache-check
    ```
3.  **Run Cargo through the wrapper**:
    ```bash
    cargo with-sccache test --workspace --all-features
    cargo sccache-stats
    ```

The wrapper sets `RUSTC_WRAPPER=sccache` and defaults `CARGO_INCREMENTAL=0` because incrementally compiled Rust crates do not produce sccache hits. Pass `cargo xtask sccache --keep-incremental -- test ...` if you want to preserve your current incremental setting.
If you want cache hits across multiple worktrees or checkout roots, use `cargo xtask sccache --basedir <PATH> -- test ...` so the wrapper sets `SCCACHE_BASEDIRs` explicitly.

### Local Hooks

Enable the project's git hooks for automated lint-fix and quality gating:

```bash
git config core.hooksPath .githooks
```

This is a one-time setup. Two hooks are provided:

- **pre-commit** → Runs `cargo xtask lint-fix` (fmt + clippy --fix + clippy verify), restages fixed files, and runs `typos --diff` (if installed). Only triggers when `.rs`, `Cargo.toml`, or `Cargo.lock` files are staged.
- **pre-push** → Runs `cargo xtask gate --check` (fmt check + cargo check + clippy + test compile-only) to catch issues before they reach CI.

You can bypass hooks with `git commit --no-verify` or `git push --no-verify` in emergencies.

## Project Structure

The codebase uses a tiered microcrate architecture:

```
crates/
├── tokmd-types/              # Tier 0: Core data structures
├── tokmd-analysis-types/     # Tier 0: Analysis receipt types
├── tokmd-settings/           # Tier 0: Clap-free settings types
├── tokmd-envelope/           # Tier 0: Cross-fleet sensor report contract
├── tokmd-substrate/          # Tier 0: Shared repo context
├── tokmd-scan/               # Tier 1: Tokio wrapper
├── tokmd-model/              # Tier 1: Aggregation logic
├── tokmd-tokeignore/         # Tier 1: Template generation
├── tokmd-path/               # Tier 1: Path utilities
├── tokmd-walk/               # Tier 1: File tree traversal
├── tokmd-analysis-complexity/ # Tier 2: Complexity metrics
├── tokmd-analysis-coverage/  # Tier 2: Coverage mapping
├── tokmd-analysis-entropy/   # Tier 2: Shannon entropy
├── tokmd-analysis-fingerprint/ # Tier 2: Code fingerprinting
├── tokmd-analysis-halstead/  # Tier 2: Halstead metrics
├── tokmd-analysis-maintainability/ # Tier 2: Maintainability index
├── tokmd-analysis-nesting/   # Tier 2: Nesting depth
├── tokmd-analysis-receipt/   # Tier 2: Receipt assembly
├── tokmd-analysis-sources/   # Tier 2: Source counting
├── tokmd-analysis-structure/ # Tier 2: Tree-sitter analysis
├── tokmd-analysis-typos/     # Tier 2: Typos integration
├── tokmd-mergequeue-types/   # Tier 2: Mergequeue protocol types
├── tokmd-gate/               # Tier 2: Quality gate
├── tokmd-policy/             # Tier 2: Policy enforcement
├── tokmd-receipts/           # Tier 2: Receipt generation
├── tokmd-report/             # Tier 2: Report formatting
├── tokmd-critic/             # Tier 2: Critic agent
├── tokmd/                    # Tier 3: CLI binary
└── tokmd-composite/          # Tier 3: Composite analysis
```

## Branch Naming Conventions

This repository uses tool-specific branch naming prefixes. Different tools in our workflow generate different prefix styles, and all of the following are valid:

### Accepted Prefixes

| Prefix | Used By | Example |
|--------|---------|---------|
| `fix/` | Manual fixes, some tools | `fix/path-traversal` |
| `feature/` | Feature branches | `feature/wasm-support` |
| `dev/` | Development branches | `dev/receipt-v2` |
| `deep_` | Deep exploration sessions | `deep_analysis_metrics` |

### Why Multiple Conventions?

Different tools in our SDLC generate branches with different naming styles. Rather than enforce a single convention (which would require tool reconfiguration), we accept all of these as valid.

### Merge Requirements

Regardless of prefix:
- Branch must be descriptive of the change
- PR must link to relevant issues
- All receipts must be present before merge

### For Agent-Generated Branches

Agents should use the prefix appropriate to the change type:
- Bug fixes → `fix/`
- New functionality → `feature/`
- Exploratory work → `deep_`

## Pull Request Process

1.  **Create a branch** using the naming conventions above
2.  **Make your changes** with clear, focused commits
3.  **Run quality gates** locally:
    ```bash
    cargo xtask gate  # Full quality gate
    ```
4.  **Create a PR** with:
    - Descriptive title
    - Link to related issues
    - Summary of changes
5.  **Address feedback** and ensure all CI checks pass

## Coding Standards

- **Formatting**: Run `cargo fmt` before committing
- **Clippy**: Address all clippy warnings
- **Tests**: Add tests for new functionality
- **Documentation**: Update README.md and add doc comments for public APIs

## Release Process

Releases are managed through the mergequeue system. Ensure all receipts are present and quality gates pass before merge.

## Questions?

Open an issue for any questions about contributing to tokmd.