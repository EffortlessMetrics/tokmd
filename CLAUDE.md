# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

**tokmd** is a Rust CLI tool and library that wraps the `tokei` library to generate "inventory receipts" and derived analytics of code repositories. It produces human-readable summaries (Markdown/TSV) and machine-friendly datasets (JSON/JSONL/CSV) for AI-native workflows, LLM context generation, and code analysis pipelines.

## Build and Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build with LTO
cargo test --verbose                 # Run all tests
cargo fmt                            # Format code
cargo clippy -- -D warnings          # Lint with strict warnings
cargo install --path crates/tokmd    # Local install
```

## Architecture

The codebase follows a tiered microcrate architecture: **types → scan → model → format → analysis → CLI**

### Crate Hierarchy

| Tier | Crate | Purpose |
|------|-------|---------|
| 0 | `tokmd-types` | Core data structures, no dependencies |
| 0 | `tokmd-analysis-types` | Analysis receipt types |
| 1 | `tokmd-scan` | tokei wrapper for code scanning |
| 1 | `tokmd-model` | Aggregation logic (lang, module, file rows) |
| 1 | `tokmd-tokeignore` | `.tokeignore` template generation |
| 1 | `tokmd-redact` | BLAKE3-based path redaction utilities |
| 2 | `tokmd-format` | Output rendering (Markdown, TSV, JSON) |
| 2 | `tokmd-walk` | File system traversal for assets |
| 2 | `tokmd-content` | File content scanning (entropy, imports) |
| 2 | `tokmd-git` | Git history analysis |
| 3 | `tokmd-analysis` | Analysis orchestration and enrichers |
| 3 | `tokmd-analysis-format` | Analysis output rendering |
| 3 | `tokmd-fun` | Novelty outputs (eco-label, etc.) |
| 3 | `tokmd-gate` | Policy evaluation with JSON pointer rules |
| 4 | `tokmd-config` | Configuration loading (`tokmd.toml`) |
| 4 | `tokmd-core` | Library facade for external consumers |
| 5 | `tokmd` | CLI binary |
| — | `tokmd-ffi` | C-compatible FFI layer (planned) |
| — | `tokmd-python` | PyO3 bindings for PyPI (planned) |
| — | `tokmd-node` | napi-rs bindings for npm (planned) |

### CLI Commands

- `tokmd` / `tokmd lang` — Language summary
- `tokmd module` — Module breakdown by directory
- `tokmd export` — File-level inventory (JSONL/CSV/CycloneDX)
- `tokmd run` — Full scan with artifact output
- `tokmd analyze` — Derived metrics and enrichments
- `tokmd badge` — SVG badge generation
- `tokmd diff` — Compare two runs or receipts
- `tokmd cockpit` — PR metrics for code review with evidence gates
- `tokmd gate` — Policy-based quality gates with JSON pointer rules
- `tokmd tools` — Generate LLM tool definitions (OpenAI, Anthropic, JSON Schema)
- `tokmd context` — Pack files into LLM context window within token budget
- `tokmd init` — Generate `.tokeignore` template
- `tokmd check-ignore` — Explain why files are being ignored
- `tokmd completions` — Generate shell completions

### Analysis Presets

| Preset | Includes |
|--------|----------|
| `receipt` | Core derived metrics (density, distribution, COCOMO) |
| `health` | + TODO density |
| `risk` | + Git hotspots, coupling, freshness |
| `supply` | + Assets, dependency lockfiles |
| `architecture` | + Import graph |
| `topics` | Semantic topic clouds |
| `security` | License radar, entropy profiling |
| `identity` | Archetype detection, corporate fingerprint |
| `git` | Predictive churn, advanced git metrics |
| `deep` | Everything (except fun) |
| `fun` | Eco-label, novelty outputs |

## Critical Patterns

### Deterministic Output
- Uses `BTreeMap` instead of `HashMap` everywhere for stable key ordering
- Sorting: descending by code lines, then by name
- Essential for golden snapshot tests and reproducible receipts

### Path Normalization
- All paths normalized to forward slashes (`/`) regardless of OS
- Always use `normalize_path()` before output
- Module keys computed from normalized paths

### Children/Embedded Language Handling
- `ChildrenMode::Collapse`: Merge embedded languages into parent totals
- `ChildrenMode::Separate`: Show as "(embedded)" rows
- Applies consistently across all commands

### Receipt Schema
- JSON outputs include envelope metadata with `schema_version`
- Increment schema_version when modifying JSON output structure
- Update `docs/schema.json` (formal JSON Schema) when structures change
- **Schema versions are separate for each receipt family**:
  - Core receipts (`lang`, `module`, `export`, `diff`, `context`, `run`): `SCHEMA_VERSION = 2` (in `tokmd-types`)
  - Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 4` (in `tokmd-analysis-types`)
  - Cockpit receipts: `SCHEMA_VERSION = 3` (local to cockpit.rs)

### Feature Flags
- `git`: Git history analysis (requires git2)
- `content`: File content scanning
- `walk`: Filesystem traversal for assets

## Testing

- **Integration tests**: `crates/tokmd/tests/` using `assert_cmd` + `predicates`
- **Golden snapshots**: Using `insta` crate (timestamps normalized)
- **Crate-level tests**: Each crate has its own `tests/` directory
- **Unit tests**: In-module tests
- **Property-based tests**: Using `proptest` for invariant verification (tokmd-redact, tokmd-tokeignore, tokmd-walk)
- **Fuzz targets**: Using `libfuzzer` for crash/panic detection (see `fuzz/` directory)
- **Mutation testing**: Using `cargo-mutants` for test quality verification

Run a single test:
```bash
cargo test test_name --verbose
```

Update snapshots:
```bash
cargo insta review
```

Run property tests:
```bash
cargo test -p tokmd-redact properties
```

Run mutation testing:
```bash
cargo mutants --file crates/tokmd-redact/src/lib.rs
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tokei` | Core LOC counting |
| `clap` (derive) | CLI parsing |
| `serde`/`serde_json` | JSON serialization |
| `blake3` | Fast hashing for redaction and integrity |
| `anyhow` | Error handling |
| `git2` | Git history analysis (optional) |
| `ignore` | File walking with gitignore support |

## Documentation

- `docs/recipes.md`: Real-world usage examples
- `docs/tutorial.md`: Getting started guide
- `docs/reference-cli.md`: CLI flag reference
- `docs/explanation.md`: Philosophy and design principles
- `docs/SCHEMA.md`: Receipt format documentation
- `docs/schema.json`: Formal JSON Schema Draft 7 definition
- `docs/PRODUCT.md`: Product contract and invariants
- `docs/troubleshooting.md`: Common issues and solutions
- `ROADMAP.md`: Current status and future plans
- `CHANGELOG.md`: Version history
- `CONTRIBUTING.md`: Development setup, testing, and publishing guide
