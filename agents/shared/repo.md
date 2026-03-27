# Shared Agent Repo Guide

This file is the canonical shared repo context for checked-in agent adapters in this repository.

## Project Overview

**tokmd** is a Rust CLI tool and library that generates deterministic inventory receipts and derived analytics for code repositories. It produces human-readable summaries and machine-friendly artifacts for AI-native workflows, LLM context generation, code analysis, and review pipelines.

## Developer Workflow

Common commands:

```bash
cargo build
cargo build --release
cargo test --workspace
cargo fmt-fix
cargo fmt-check
cargo clippy --all-features -- -D warnings
cargo xtask lint-fix
cargo xtask gate --check
just lint
just fmt
just publish-plan
```

Optional git hooks:

```bash
git config core.hooksPath .githooks
```

## Architecture

The codebase follows a tiered microcrate architecture:

`types -> scan -> model -> format -> analysis -> CLI`

Tier summary:

| Tier | Purpose | Example crates |
|------|---------|----------------|
| 0 | Contracts and settings | `tokmd-types`, `tokmd-analysis-types`, `tokmd-settings`, `tokmd-envelope`, `tokmd-substrate` |
| 1 | Core scan and aggregation | `tokmd-scan`, `tokmd-model`, `tokmd-redact`, `tokmd-context-policy`, `tokmd-scan-args`, `tokmd-sensor` |
| 2 | Adapters and rendering | `tokmd-format`, `tokmd-walk`, `tokmd-content`, `tokmd-git`, `tokmd-badge`, `tokmd-progress` |
| 3 | Analysis orchestration | `tokmd-analysis`, `tokmd-analysis-format`, `tokmd-analysis-*`, `tokmd-fun`, `tokmd-gate` |
| 4 | Facade and config | `tokmd-config`, `tokmd-tool-schema`, `tokmd-core` |
| 5 | End-user products | `tokmd`, `tokmd-python`, `tokmd-node` |

Dependency rule:

- Lower tiers must never depend on higher tiers.

## CLI Surface

- `tokmd` / `tokmd lang` - language summary
- `tokmd module` - module breakdown
- `tokmd export` - file-level inventory
- `tokmd run` - full scan with artifacts
- `tokmd analyze` - derived metrics and enrichments
- `tokmd badge` - SVG badge generation
- `tokmd diff` - compare runs or receipts
- `tokmd cockpit` - PR metrics and evidence gates
- `tokmd sensor` - sensor envelope output
- `tokmd gate` - policy evaluation
- `tokmd tools` - LLM tool definitions
- `tokmd context` - context packing under token budget
- `tokmd baseline` - baseline capture
- `tokmd handoff` - LLM handoff bundle generation
- `tokmd init` - generate `.tokeignore`
- `tokmd check-ignore` - explain ignore decisions
- `tokmd completions` - shell completions

## Critical Invariants

### Deterministic output

- Use `BTreeMap` instead of `HashMap` for stable ordering.
- Sort descending by code lines, then by name.
- Keep output byte-stable for snapshot and receipt diffs.

### Path normalization

- Normalize output paths to forward slashes (`/`) on every platform.
- Normalize before emitting output or computing module keys.

### Children and embedded languages

- `ChildrenMode::Collapse` merges embedded languages into parent totals.
- `ChildrenMode::Separate` emits explicit embedded rows.
- Apply this consistently across commands and receipt surfaces.

### Schema versioning

- Bump the relevant schema version when JSON structure changes.
- Update formal schema docs when structure changes.
- Receipt families currently version independently:
  - core receipts: `SCHEMA_VERSION = 2`
  - analysis receipts: `ANALYSIS_SCHEMA_VERSION = 9`
  - cockpit receipts: `COCKPIT_SCHEMA_VERSION = 3`
  - handoff manifests: `HANDOFF_SCHEMA_VERSION = 5`
  - context receipts: `CONTEXT_SCHEMA_VERSION = 4`
  - context bundles: `CONTEXT_BUNDLE_SCHEMA_VERSION = 2`

### Feature flags

- `git` enables git-history analysis.
- `content` enables file-content scanning.
- `walk` enables filesystem traversal helpers.
- `halstead` requires `content` plus `walk`.

### Git range syntax

| Syntax | Meaning | Use case |
|--------|---------|----------|
| `A..B` | commits reachable from `B` but not `A` | comparing tags or releases |
| `A...B` | symmetric difference from merge-base | CI workflows comparing branch divergence |

Rule:

- Use `..` in cockpit and diff flows comparing releases or tags.
- Use `...` only in CI workflows that want branch-divergence changes.

## Testing Notes

- Integration tests live under `crates/tokmd/tests/`.
- Golden snapshots use `insta`.
- Property testing uses `proptest`.
- Fuzz targets live in `fuzz/`.
- Mutation testing is configured in `.cargo/mutants.toml`.

Common targeted commands:

```bash
cargo test test_name --verbose
cargo test -p tokmd-redact properties
cargo mutants --file crates/tokmd-redact/src/lib.rs
cargo +nightly fuzz list
```

## Reference Docs

- `docs/architecture.md`
- `docs/design.md`
- `docs/requirements.md`
- `docs/implementation-plan.md`
- `docs/reference-cli.md`
- `docs/SCHEMA.md`
- `docs/schema.json`
- `docs/testing.md`
- `docs/PRODUCT.md`
- `docs/explanation.md`
- `ROADMAP.md`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
