# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**tokmd** is a Rust CLI tool that wraps the `tokei` library to generate "inventory receipts" of code repositories. It produces human-readable summaries (Markdown/TSV) and machine-friendly datasets (JSON/JSONL/CSV) for AI-native workflows, LLM context generation, and code analysis pipelines.

## Build and Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build with LTO
cargo test --verbose                 # Run all tests
cargo fmt                            # Format code
cargo clippy -- -D warnings          # Lint with strict warnings
cargo install --path .               # Local install
```

## Architecture

The codebase follows a pipeline architecture: **scan → model → format**

### Core Modules

- **`src/lib.rs`** - Entry point with `run()` function that orchestrates CLI workflow and routes to subcommands (lang, module, export, init)

- **`src/cli.rs`** - Clap-based argument parsing with flattened argument groups:
  - `GlobalArgs`: paths, ignore patterns, config modes, verbosity
  - `LangArgs`: format, top-N filtering, children handling
  - `ModuleArgs`: module roots, depth, children mode
  - `ExportArgs`: format, redaction, row filtering, metadata
  - `InitArgs`: `.tokeignore` template generation

- **`src/scan.rs`** - Wrapper around tokei's `Languages` API for code scanning

- **`src/model.rs`** - Data structures and aggregation:
  - `LangRow/LangReport`: Language-level summaries
  - `ModuleRow/ModuleReport`: Module-level summaries
  - `FileRow/ExportData`: Per-file detailed records
  - Key functions: `normalize_path()`, `module_key()`, `collect_file_rows()`

- **`src/format.rs`** - Output formatting and receipt generation:
  - `LangReceipt`, `ModuleReceipt`: Schema v1 envelopes
  - Redaction using BLAKE3 hashing
  - Metadata includes schema_version, timestamps, tool info, scan args

- **`src/tokeignore.rs`** - `.tokeignore` template generation for various ecosystems

### Binaries

- `src/bin/tokmd.rs`: Main binary
- `src/bin/tok.rs`: Alias binary (feature-gated: `alias-tok`)

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

### Redaction
- `RedactMode::None`: No redaction
- `RedactMode::Paths`: Hash file paths (preserves extension)
- `RedactMode::All`: Hash paths AND module names

## Testing

- **Integration tests**: `tests/integration.rs` using `assert_cmd` + `predicates`
- **Golden snapshots**: `tests/snapshots/` using `insta` crate (timestamps normalized)
- **Fixture data**: `tests/data/` (main.rs, large.rs, ignored.rs)
- **Unit tests**: In-module tests in `src/model.rs`

Run a single test:
```bash
cargo test test_name --verbose
```

Update snapshots:
```bash
cargo insta review
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tokei` | Core LOC counting |
| `clap` (derive) | CLI parsing |
| `serde`/`serde_json` | JSON serialization |
| `blake3` | Fast hashing for redaction |
| `anyhow` | Error handling |

## Documentation

- `docs/recipes.md`: Real-world usage examples
- `docs/SCHEMA.md`: Receipt format documentation
- `docs/schema.json`: Formal JSON Schema Draft 7 definition
- `ROADMAP.md`: v1.0 path and future plans
