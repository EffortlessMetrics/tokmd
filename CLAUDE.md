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
| 0 | `tokmd-settings` | Clap-free settings types (`ScanOptions`, etc.) |
| 0 | `tokmd-envelope` | Cross-fleet `SensorReport` contract |
| 0 | `tokmd-substrate` | Shared repo context (`RepoSubstrate`) |
| 1 | `tokmd-scan` | tokei wrapper for code scanning |
| 1 | `tokmd-model` | Aggregation logic (lang, module, file rows) |
| 1 | `tokmd-tokeignore` | `.tokeignore` template generation |
| 1 | `tokmd-redact` | BLAKE3-based path redaction utilities |
| 1 | `tokmd-sensor` | `EffortlessSensor` trait + substrate builder |
| 2 | `tokmd-format` | Output rendering (Markdown, TSV, JSON) |
| 2 | `tokmd-walk` | File system traversal for assets |
| 2 | `tokmd-content` | File content scanning (entropy, imports) |
| 2 | `tokmd-git` | Git history analysis |
| 2 | `tokmd-badge` | SVG badge rendering helpers |
| 2 | `tokmd-progress` | Progress spinner and progress-bar abstractions |
| 3 | `tokmd-analysis` | Analysis orchestration and enrichers |
| 3 | `tokmd-analysis-format` | Analysis output rendering |
| 3 | `tokmd-fun` | Novelty outputs (eco-label, etc.) |
| 3 | `tokmd-gate` | Policy evaluation with JSON pointer rules |
| 4 | `tokmd-config` | Configuration loading (`tokmd.toml`) |
| 4 | `tokmd-core` | Library facade with FFI layer |
| 5 | `tokmd` | CLI binary |
| 5 | `tokmd-python` | PyO3 bindings for PyPI |
| 5 | `tokmd-node` | napi-rs bindings for npm |

### CLI Commands

- `tokmd` / `tokmd lang` — Language summary
- `tokmd module` — Module breakdown by directory
- `tokmd export` — File-level inventory (JSONL/CSV/CycloneDX)
- `tokmd run` — Full scan with artifact output
- `tokmd analyze` — Derived metrics and enrichments
- `tokmd badge` — SVG badge generation
- `tokmd diff` — Compare two runs or receipts
- `tokmd cockpit` — PR metrics for code review with evidence gates
- `tokmd sensor` — Conforming sensor producing sensor.report.v1 envelope
- `tokmd gate` — Policy-based quality gates with JSON pointer rules
- `tokmd tools` — Generate LLM tool definitions (OpenAI, Anthropic, JSON Schema)
- `tokmd context` — Pack files into LLM context window within token budget
- `tokmd baseline` — Capture complexity baseline for trend tracking
- `tokmd handoff` — Bundle codebase for LLM handoff with intelligence presets
- `tokmd init` — Generate `.tokeignore` template
- `tokmd check-ignore` — Explain why files are being ignored
- `tokmd completions` — Generate shell completions

### Library API (tokmd-core)

The `tokmd-core` crate provides a clap-free library facade for embedding:

**Workflow Functions** (Rust):
- `lang_workflow(scan, lang) -> LangReceipt`
- `module_workflow(scan, module) -> ModuleReceipt`
- `export_workflow(scan, export) -> ExportReceipt`
- `diff_workflow(settings) -> DiffReceipt`

**FFI Layer** (`ffi::run_json`):
- Single JSON entrypoint: `run_json(mode, args_json) -> String`
- Modes: `lang`, `module`, `export`, `analyze`, `diff`, `version`
- Response envelope: `{"ok": bool, "data": {...}, "error": {...}}`

**Python Bindings** (tokmd-python):
- `tokmd.lang()`, `tokmd.module()`, `tokmd.export()`, `tokmd.analyze()`, `tokmd.diff()`
- Returns native Python dicts
- Releases GIL during long scans

**Node.js Bindings** (tokmd-node):
- All functions return Promises (async)
- Uses `tokio::task::spawn_blocking()` for non-blocking event loop

### Analysis Presets

| Preset | Includes |
|--------|----------|
| `receipt` | Core derived metrics (density, distribution, COCOMO) |
| `health` | + TODO density, complexity, Halstead metrics |
| `risk` | + Git hotspots, coupling, freshness, complexity, Halstead metrics |
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
  - Core receipts (`lang`, `module`, `export`, `diff`, `run`): `SCHEMA_VERSION = 2` (in `tokmd-types`)
  - Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 5` (in `tokmd-analysis-types`)
  - Cockpit receipts: `SCHEMA_VERSION = 3` (local to cockpit.rs)
  - Handoff manifests: `HANDOFF_SCHEMA_VERSION = 4` (in `tokmd-types`)
  - Context receipts: `CONTEXT_SCHEMA_VERSION = 3` (in `tokmd-types`)
  - Context bundles: `CONTEXT_BUNDLE_SCHEMA_VERSION = 2` (in `tokmd-types`)

### Feature Flags
- `git`: Git history analysis (uses shell `git log`)
- `content`: File content scanning (entropy, tags, hashing)
- `walk`: Filesystem traversal for assets
- `halstead`: Halstead metrics computation (requires `content` + `walk`)

### Git Diff Syntax (Two-dot vs Three-dot)
When invoking `git diff` or `git log` with range syntax:

| Syntax | Meaning | Use Case |
|--------|---------|----------|
| `A..B` | Commits reachable from B but not A | Comparing tags/releases (`cockpit`, `diff` commands) |
| `A...B` | Symmetric difference from merge-base | CI workflows comparing PR branches |

**Rule**: Use `..` (two dots) in cockpit/diff commands comparing releases or tags. Use `...` (three dots) only in CI workflows where you want changes since branch divergence.

## Testing

- **Integration tests**: `crates/tokmd/tests/` using `assert_cmd` + `predicates`
- **Golden snapshots**: Using `insta` crate (timestamps normalized)
- **Crate-level tests**: Each crate has its own `tests/` directory
- **Unit tests**: In-module tests
- **Property-based tests**: Using `proptest` across 14 crates for invariant verification
- **Fuzz targets**: 9 targets using `libfuzzer` (see `fuzz/` directory) with seed corpus and dictionaries
- **Mutation testing**: Using `cargo-mutants` for test quality verification (configured in `.cargo/mutants.toml`)

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

Run fuzz targets:
```bash
cargo +nightly fuzz run fuzz_entropy --features content
cargo +nightly fuzz list  # List all targets
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tokei` | Core LOC counting |
| `clap` (derive) | CLI parsing |
| `serde`/`serde_json` | JSON serialization |
| `blake3` | Fast hashing for redaction and integrity |
| `anyhow` | Error handling |
| `ignore` | File walking with gitignore support |
| `pyo3` | Python bindings (tokmd-python) |
| `napi-rs` | Node.js bindings (tokmd-node) |

## Documentation

### Architecture & Design
- `docs/architecture.md`: Crate hierarchy, data flow, dependency rules
- `docs/design.md`: Design principles, system context, data model
- `docs/requirements.md`: Requirements, interfaces, quality bar
- `docs/implementation-plan.md`: Phased roadmap for future work

### User Guides
- `docs/tutorial.md`: Getting started guide
- `docs/recipes.md`: Real-world usage examples
- `docs/reference-cli.md`: CLI flag reference
- `docs/troubleshooting.md`: Common issues and solutions

### Technical Reference
- `docs/SCHEMA.md`: Receipt format documentation
- `docs/schema.json`: Formal JSON Schema Draft 7 definition
- `docs/testing.md`: Testing strategy and frameworks

### Product & Philosophy
- `docs/PRODUCT.md`: Product contract and invariants
- `docs/explanation.md`: Philosophy and design principles

### Project
- `ROADMAP.md`: Current status and future plans
- `CHANGELOG.md`: Version history
- `CONTRIBUTING.md`: Development setup, testing, and publishing guide
