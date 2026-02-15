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

## Project Structure

The codebase uses a tiered microcrate architecture:

```
crates/
├── tokmd-types/           # Tier 0: Core data structures
├── tokmd-analysis-types/  # Tier 0: Analysis receipt types
├── tokmd-settings/        # Tier 0: Clap-free settings types
├── tokmd-envelope/        # Tier 0: Cross-fleet sensor report contract
├── tokmd-substrate/       # Tier 0: Shared repo context
├── tokmd-scan/            # Tier 1: tokei wrapper
├── tokmd-model/           # Tier 1: Aggregation logic
├── tokmd-tokeignore/      # Tier 1: Template generation
├── tokmd-redact/          # Tier 1: BLAKE3-based path redaction
├── tokmd-sensor/          # Tier 1: Sensor trait + substrate builder
├── tokmd-format/          # Tier 2: Output rendering
├── tokmd-walk/            # Tier 2: File system traversal
├── tokmd-content/         # Tier 2: Content scanning
├── tokmd-git/             # Tier 2: Git analysis
├── tokmd-analysis/        # Tier 3: Analysis orchestration
├── tokmd-analysis-format/ # Tier 3: Analysis output
├── tokmd-fun/             # Tier 3: Novelty outputs
├── tokmd-gate/            # Tier 3: Policy evaluation
├── tokmd-config/          # Tier 4: Configuration
├── tokmd-core/            # Tier 4: Library facade + FFI
├── tokmd/                 # Tier 5: CLI binary
├── tokmd-python/          # Tier 5: Python bindings (PyO3)
└── tokmd-node/            # Tier 5: Node.js bindings (napi-rs)
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

### 4. Property-Based Testing (proptest)

Property-based testing verifies that functions behave correctly across a wide range of randomly generated inputs. Instead of testing specific examples, you define properties that should always hold.

**When to use**: Functions with well-defined invariants (e.g., determinism, length constraints, cross-platform consistency).

**Running property tests**:
```bash
cargo test -p tokmd-redact properties  # Run property tests for tokmd-redact
cargo test --workspace                  # Includes all property tests
```

**Example patterns** (from `crates/tokmd-redact/tests/properties.rs`):
```rust
use proptest::prelude::*;

proptest! {
    /// Hash output is always exactly 16 hex characters.
    #[test]
    fn short_hash_length_is_16(input in ".*") {
        let hash = short_hash(&input);
        prop_assert_eq!(hash.len(), 16);
    }

    /// Same input always produces same hash (determinism).
    #[test]
    fn short_hash_is_deterministic(input in ".*") {
        let h1 = short_hash(&input);
        let h2 = short_hash(&input);
        prop_assert_eq!(h1, h2, "Hash must be deterministic");
    }

    /// Unix and Windows paths produce identical hashes.
    #[test]
    fn short_hash_normalizes_separators(path in arb_path()) {
        let unix_path = path.replace('\\', "/");
        let windows_path = path.replace('/', "\\");
        prop_assert_eq!(short_hash(&unix_path), short_hash(&windows_path));
    }
}
```

**Key concepts**:
- Define *strategies* to generate test data (e.g., `arb_path()` for path-like strings)
- Use `prop_assert!` and `prop_assert_eq!` instead of standard assertions
- Use `prop_assume!` to skip inputs that don't meet preconditions
- Proptest will shrink failing cases to minimal reproducible examples

### 5. Fuzz Testing (libfuzzer)

Fuzz testing bombards functions with arbitrary byte sequences to find crashes, panics, and edge cases. This is especially useful for parsing code and security-sensitive operations.

**Prerequisites**:
```bash
rustup install nightly
cargo +nightly install cargo-fuzz
```

**Running fuzz targets**:
```bash
cargo +nightly fuzz run fuzz_entropy --features content
cargo +nightly fuzz run fuzz_json_types --features types
cargo +nightly fuzz run fuzz_redact --features redact
```

**Available targets** (see `fuzz/README.md` for complete list):
| Target | Feature | Description |
|--------|---------|-------------|
| `fuzz_entropy` | `content` | Entropy calculation |
| `fuzz_json_types` | `types` | JSON deserialization of receipt types |
| `fuzz_normalize_path` | `model` | Path normalization |
| `fuzz_toml_config` | `config` | `tokmd.toml` config parsing |
| `fuzz_redact` | `redact` | Path redaction |

**Using dictionaries** for better coverage:
```bash
cargo +nightly fuzz run fuzz_json_types --features types -- -dict=fuzz/dict/json.dict
```

**Limiting input size**:
```bash
cargo +nightly fuzz run fuzz_entropy --features content -- -max_len=4096
```

For full documentation, see [`fuzz/README.md`](fuzz/README.md).

### 6. Mutation Testing (cargo-mutants)

Mutation testing verifies test suite quality by introducing small changes (mutations) to the code and checking if tests catch them. Surviving mutations indicate gaps in test coverage.

**Install**:
```bash
cargo install cargo-mutants
```

**Running locally**:
```bash
# Test mutations in a specific file
cargo mutants --file crates/tokmd-redact/src/lib.rs

# Test a specific crate (run from crate directory for proper test scoping)
cd crates/tokmd-redact && cargo mutants

# Test with all features enabled
cargo mutants --all-features
```

**Interpreting results**:
- **Killed**: Test suite caught the mutation (good)
- **Timeout**: Mutation caused tests to hang (usually acceptable)
- **Unviable**: Mutation caused compilation failure (neutral)
- **Missed/Survived**: Mutation was not detected (indicates a test gap)

**Configuration** (`.cargo/mutants.toml`):
```toml
all_features = true
gitignore = true
timeout_multiplier = 2.0

# Exclude test code from mutation
exclude_globs = ["**/tests/**", "fuzz/**"]

# Exclude boilerplate
exclude_re = ["impl.*Display", "fn main\\("]
```

**CI integration**: Mutation testing runs automatically on PRs via `.github/workflows/mutants.yml`:
- Only tests changed `.rs` files (up to 20 files)
- Generates a summary JSON with survivors
- PRs with surviving mutations will fail CI

**When mutations survive**:
1. Review the mutation to understand what behavior wasn't tested
2. Add a test that specifically exercises that code path
3. Re-run mutation testing to verify the mutation is now caught

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
- `halstead`: Halstead software science metrics (requires `content` + `walk`)

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

## Language Bindings

Native bindings are available for Python and Node.js:

```
crates/
├── tokmd-core/     # FFI layer via ffi::run_json()
├── tokmd-python/   # PyO3 bindings → PyPI
└── tokmd-node/     # napi-rs bindings → npm
```

### Python (tokmd-python)

```python
import tokmd

# All functions return native Python dicts
result = tokmd.lang(paths=["src"])
result = tokmd.analyze(paths=["."], preset="risk")
```

- Install: `pip install tokmd`
- Functions: `lang()`, `module()`, `export()`, `analyze()`, `diff()`
- Releases GIL during long scans

### Node.js (tokmd-node)

```javascript
const tokmd = require('tokmd');

// All functions return Promises
const result = await tokmd.lang({ paths: ['src'] });
const analysis = await tokmd.analyze({ paths: ['.'], preset: 'risk' });
```

- Install: `npm install tokmd`
- Functions: `lang()`, `module()`, `export()`, `analyze()`, `diff()`
- Non-blocking via `spawn_blocking()`

### FFI Interface

Both bindings use the unified FFI layer in `tokmd-core`:

```rust
pub fn run_json(mode: &str, args_json: &str) -> String
// Response: {"ok": bool, "data": {...}, "error": {...}}
```

**Design principles:**
- JSON serialization at FFI boundary for simplicity
- Mirror the CLI's mental model (`lang`, `analyze`, `diff`)
- Return native language types (Python dicts, JS objects)
- Cross-platform wheels/prebuilds via CI matrix

## AI-Assisted Development

If you use AI tools like Claude Code to contribute to this project, please refer to [`CLAUDE.md`](CLAUDE.md) for project-specific guidance. This file contains:

- Build and test commands
- Architecture overview and crate hierarchy
- Critical patterns (deterministic output, path normalization, etc.)
- Testing strategies and snapshot management
- Key dependencies and documentation references

The CLAUDE.md file helps AI assistants understand the codebase conventions and produce contributions that align with project standards.
