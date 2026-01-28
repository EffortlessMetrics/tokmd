# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Archetype Detection**: Identify project types (CLI, library, web app, monorepo)
- **Topic Clouds**: TF-IDF semantic analysis of path segments
- **Entropy Profiling**: Detect high-entropy files (potential secrets)
- **Predictive Churn**: Linear regression on commit history for trend detection
- **Corporate Fingerprint**: Author domain statistics from git history
- **License Radar**: SPDX detection from LICENSE files and metadata

### Changed
- **Schema Version**: Analysis receipts now use `schema_version: 2`

## [1.2.0] - 2026-01-27

### Added
- **Microcrate Architecture**: Split into 15 focused crates for modularity and selective compilation
  - `tokmd-types`, `tokmd-analysis-types` (Tier 0: data structures)
  - `tokmd-scan`, `tokmd-model`, `tokmd-tokeignore` (Tier 1: core logic)
  - `tokmd-format`, `tokmd-walk`, `tokmd-content`, `tokmd-git` (Tier 2: I/O)
  - `tokmd-analysis`, `tokmd-analysis-format`, `tokmd-fun` (Tier 3: enrichment)
  - `tokmd-config`, `tokmd-core` (Tier 4: orchestration)
  - `tokmd` (Tier 5: CLI binary)
- **Git Integration**: Hotspots, bus factor, freshness, coupling analysis
- **Asset Inventory**: Non-code file categorization and size tracking
- **Dependency Summary**: Lockfile detection and dependency counting
- **Import Graph**: Module dependency analysis with configurable granularity
- **Duplicate Detection**: Content-hash based duplicate file detection

### Changed
- **Feature Flags**: Git, content, and walk features are now opt-in for faster compilation
- **Analysis Limits**: Added `--max-files`, `--max-bytes`, `--max-commits` for resource control

## [1.1.0] - 2026-01-26

### Added
- **`tokmd analyze`**: New command for derived metrics and enrichments
  - Presets: `receipt`, `health`, `risk`, `supply`, `architecture`, `topics`, `security`, `identity`, `git`, `deep`, `fun`
  - Output formats: `md`, `json`, `jsonld`, `xml`, `svg`, `mermaid`, `obj`, `midi`, `tree`
- **`tokmd badge`**: Generate SVG badges for metrics (lines, tokens, bytes, doc%, hotspots)
- **`tokmd diff`**: Compare two runs or receipts for delta analysis
- **`tokmd run`**: Execute full scans and save artifacts to a run directory
- **Derived Metrics**:
  - Doc density (comments/code ratio by language and module)
  - Test density (test lines vs production lines)
  - Verbosity (bytes per line)
  - Nesting depth (max and average path depth)
  - File size distribution (min, max, mean, median, p90, p99, Gini coefficient)
  - Histogram buckets (tiny, small, medium, large, huge files)
  - Top offenders (largest, least documented, most dense files)
- **COCOMO Estimation**: Effort, duration, and staffing projections
- **Context Window Analysis**: Token utilization against configurable window sizes
- **Reading Time Estimation**: Human reading time based on code volume
- **TODO Density**: TODO/FIXME/HACK tag counting and density per KLOC
- **Integrity Hash**: BLAKE3 hash of receipt contents for verification

### Changed
- **Configuration**: Added `tokmd.toml` support for persistent settings and view profiles
- **Documentation**: Added analysis presets table to README

## [1.0.0] - 2026-01-25

### Added
- **Formal Receipt Schema**: Introduced a stable JSON output format for `lang`, `module`, and `export` modes.
- **Formal Schema Definition**: Added `docs/schema.json` (JSON Schema Draft 07) to validate outputs.
- **Export Mode**: New `tokmd export` command to generate JSONL/CSV inventories of files.
- **Redaction**: `--redact paths` and `--redact all` flags to sanitize output for LLM usage.
- **Filtering**: `--min-code` and `--max-rows` flags to control output size.
- **Initialization**: `tokmd init` command to generate `.tokeignore` templates.
- **Module Analysis**: Enhanced module reporting with configurable roots (`--module-roots`) and depth (`--module-depth`).
- **Test Harness**: Robust integration suite with BDD-style scenarios and golden snapshots using `insta`.

### Changed
- **CLI**: `tokmd` (default) now produces a Markdown table by default (previously text).
- **Semantics**: `--children` flag logic unified across all modes.
- **Docs**: Completely overhauled documentation structure following Diataxis principles (Tutorials, How-to, Reference, Explanation).

### Fixed
- **Ignore Logic**: Corrected behavior where `--no-ignore` did not consistently disable all ignore types.
- **Stability**: Fixed deterministic sorting of output rows.

## [0.1.0] - 2026-01-01
- Initial prototype release.
