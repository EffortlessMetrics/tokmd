# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Analyze Explain Mode**: Added `tokmd analyze --explain <key>` for quick human-readable metric/finding definitions (`--explain list` for key discovery)
- **Diff Output Controls**: Added `tokmd diff --compact` and `tokmd diff --color <auto|always|never>` for narrow terminals and explicit color policy
- **Cockpit Trend Sparklines**: Added inline unicode sparklines for trend lines in cockpit markdown output
- **Structured CLI Error Hints**: Added actionable `Hints:` section on common failures (missing git, bad paths, missing refs, invalid explain key, TOML parse issues)
- **Technical Debt Ratio**: Added `complexity.technical_debt` to analysis receipts (complexity points per KLOC + severity bucket)
- **Duplication Density**: Added `dup.density` with overall and per-module duplicate waste density metrics
- **Code Age Distribution**: Added `git.age_distribution` with file age buckets and recent-vs-prior refresh trend

### Changed

- **Config Microcrate Extraction**: Moved `TomlConfig` schema/parsing types into `tokmd-settings`; `tokmd-config` now re-exports them for compatibility

## [1.6.2] - 2026-02-16

### Added

- **tokmd-core Analyze Workflow**: Implemented `analyze_workflow(scan, analyze)` to run export + analysis directly from the library API and FFI (`run_json("analyze", ...)`)

### Changed

- **Analyze Settings Validation**: `preset` and `granularity` in FFI analyze args are now strictly validated with `invalid_settings` errors on unknown values

### Fixed

- **Bindings Analyze Path**: Python/Node binding tests now validate successful analyze receipts instead of obsolete `not_implemented` behavior

## [1.6.1] - 2026-02-16

### Added

- **File Classification**: Auto-detect generated, vendored, fixture, lockfile, minified, sourcemap, and dense data blob files during context packing
- **Inclusion Policies**: Per-file budget caps (`--max-file-pct`, `--max-file-tokens`) with Full/HeadTail/Summary/Skip policies
- **Head/Tail Truncation**: Oversized files emit 60% head + 40% tail with omission marker
- **Graceful Metric Fallback**: When git scores unavailable for `--rank-by hotspot/churn`, falls back to code lines with transparent `fallback_reason`
- **Error Suggestions**: Actionable suggestions on git, config, and path errors (`with_suggestions()` builder)

### Changed

- **Handoff Schema**: v3 → v4 — added `rank_by_effective`, `fallback_reason`, `excluded_by_policy`, per-file `policy`/`classifications`
- **Context Bundle Schema**: v1 → v2 — added policy tracking fields
- **Context Receipt Schema**: Split from Core (`SCHEMA_VERSION = 2`) to own `CONTEXT_SCHEMA_VERSION = 3`
- **Diff Markdown**: Added comparison summary table (From / To / Delta / Change %)

### Fixed

- **Error Serialization**: `ResponseEnvelope::to_json()` fallback now emits actual error code and message instead of placeholders

## [1.6.0] - 2026-02-11

### Added

- **Sensor Command**: New `tokmd sensor` for producing conforming `sensor.report.v1` envelopes
  - Wraps cockpit computation and maps results to standardized findings and gates
  - `--base` / `--head` flags for git diff range
  - `--output` for artifact path (default: `artifacts/tokmd/report.json`)
  - `--format json|md` output selection
  - Emits risk findings (hotspots) and contract findings (schema/API/CLI changes)
  - Maps cockpit evidence gates to envelope `GateResults`

- **New Crate `tokmd-sensor`** (Tier 1): Sensor integration layer
  - `EffortlessSensor` trait with `name()`, `version()`, `run(settings, substrate)` contract
  - `build_substrate()` function runs tokei scan once and builds shared `RepoSubstrate`
  - Enables pluggable multi-sensor architecture

- **New Crate `tokmd-settings`** (Tier 0): Clap-free configuration types
  - `ScanOptions`, `ScanSettings`, `LangSettings`, `ModuleSettings`, `ExportSettings`, `AnalyzeSettings`, `DiffSettings`
  - Decouples lower-tier crates from `clap` dependency
  - Enables library usage and FFI/Python/Node bindings without pulling in CLI types

- **New Crate `tokmd-envelope`** (Tier 0): Cross-fleet sensor report contract
  - `SensorReport` envelope with schema `"sensor.report.v1"`
  - `Verdict` enum: `Pass`, `Fail`, `Warn`, `Skip`, `Pending` with aggregation rules
  - `Finding` type with `(check_id, code)` tuple for buildfix routing
  - `GateResults` and `GateItem` for evidence gate status
  - `ToolMeta` and `Artifact` metadata types
  - Finding registry with constants for risk, contract, supply, gate, security, and architecture categories

- **New Crate `tokmd-substrate`** (Tier 0): Shared repository context
  - `RepoSubstrate` with file metrics, language summaries, diff range, and totals
  - `SubstrateFile` per-file metrics including `in_diff` flag
  - `DiffRange` for git context (base, head, changed files, insertions, deletions)
  - Helper methods: `diff_files()`, `files_for_lang()`
  - Single I/O pass feeds multiple sensors, eliminating redundant scans

### Changed

- **Scan API**: `tokmd_scan::scan()` now accepts `&ScanOptions` instead of `&GlobalArgs`, decoupling Tier 1 from CLI types
- **Core Workflows**: `tokmd-core` workflow functions now use settings types (`ScanSettings`, `LangSettings`, etc.) instead of Clap-based args
- **Envelope Schema**: Changed schema identifier from numeric `sensor_report_version: u32` to semantic string `schema: String` (`"sensor.report.v1"`)
- **Finding Identity**: Replaced `Finding.id` with `(check_id, code)` tuple for category-based routing
- **Analysis Types**: Moved envelope and findings types to dedicated `tokmd-envelope` crate
- **Core Settings**: `tokmd-core` re-exports from `tokmd-settings` for backwards compatibility
- **CLI Args**: Renamed `--out` to `--output` across `export`, `badge`, and `context` commands (old name kept as visible alias)
- **Context Command**: Renamed `--output` (mode selector) to `--mode` to avoid collision with `--output` (file path)
- **Cockpit Diff Coverage**: Now intersects LCOV data with git-added lines for accurate diff-scoped coverage instead of whole-file coverage

### Fixed

- **Rust Function Regex**: Fixed pattern to match `(_|XID_Start) XID_Continue*` per Rust language spec; `fn _private_helper()` now correctly detected
- **Cross-Platform Docs**: xtask docs task now normalizes `tokmd.exe` → `tokmd` and CRLF → LF for platform-independent reference output

### Internal

- Hardened tests: replaced sentinel nonexistent paths with `tempdir` in `tokmd-scan` and `tokmd-tokeignore`
- Added `tempfile` dev-dependency to `tokmd-scan`
- Added README files for `tokmd-sensor`, `tokmd-envelope`, `tokmd-substrate`, `tokmd-settings`
- Added `tokmd sensor` documentation to `reference-cli.md`
- Updated `docs/schema.json` and `docs/SCHEMA.md` for new envelope fields
- Added `get_added_lines()` API in `tokmd-git` for per-file added-line extraction from git diff
- Added `xtask docs` command for automated CLI reference regeneration
- Added docs integration test verifying `reference-cli.md` stays in sync with CLI help output
- Added issue templates for cleanup tasks and expanded options for commands

## [1.5.0] - 2026-02-05

### Added

- **Baseline System**: New `tokmd baseline` command for tracking complexity metrics over time
  - Generate complexity baselines to `.tokmd/baseline.json` (or custom path via `--output`)
  - Captures git commit SHA for traceability
  - Support for determinism baselines with build hash tracking (planned for v1.5.1)
  - Baseline types: `ComplexityBaseline`, `BaselineMetrics`, `FileBaselineEntry`
  - Baseline JSON schema in `docs/baseline.schema.json`

- **Ratchet Rules**: Gradual improvement enforcement in `tokmd gate`
  - `--baseline` flag for comparing current state against stored baselines
  - `--ratchet-config` flag for external ratchet rule files
  - `max_increase_pct` constraint for allowing bounded metric regression
  - `max_value` constraint for absolute ceiling enforcement
  - Inline ratchet rules via `[[gate.ratchet]]` in `tokmd.toml`
  - Combined policy + ratchet evaluation with unified pass/fail reporting

- **Ecosystem Envelope Protocol**: Standardized output format for multi-sensor integration
  - `Envelope` type with verdict, findings, gates, and artifacts sections
  - Finding ID registry with `tokmd.<category>.<code>` format (e.g., `tokmd.risk.hotspot`)
  - Verdict aggregation: pass/fail/warn/skip/pending
  - Builder pattern APIs for constructing envelopes programmatically

- **Handoff Command**: New `tokmd handoff` for creating LLM-ready code bundles
  - Generates `.handoff/` directory with `manifest.json`, `map.jsonl`, `intelligence.json`, and `code.txt`
  - Token-budgeted file selection with `--budget` and `--strategy` options
  - Risk-ranked ordering via `--rank-by` (hotspot, code, tokens, churn)
  - Intelligence presets: `minimal`, `standard`, `risk`, `deep`
  - Deterministic output with BLAKE3 integrity hashes

- **Finding ID Constants**: New `tokmd_analysis_types::findings` module
  - Risk findings: `hotspot`, `coupling`, `bus_factor`, `complexity_high`, `cognitive_high`, `nesting_deep`
  - Contract findings: `schema_changed`, `api_changed`, `cli_changed`
  - Supply chain findings: `lockfile_changed`, `new_dependency`, `vulnerability`
  - Gate findings: `mutation_failed`, `coverage_failed`, `complexity_failed`
  - Security findings: `entropy_high`, `license_conflict`
  - Architecture findings: `circular_dep`, `layer_violation`

### Changed

- **Gate Config**: Extended `GateConfig` in `tokmd.toml` with ratchet support
  - New fields: `baseline`, `ratchet`, `allow_missing_baseline`, `allow_missing_current`
- **Gate CLI**: `tokmd gate` now supports combined policy and ratchet evaluation
- **Gate Output**: JSON output includes separate `policy` and `ratchet` result sections
- Extended `tokmd-analysis-types` with baseline and envelope structures
- New `BASELINE_VERSION = 1` and `ENVELOPE_VERSION = 1` constants

### Internal

- New `ratchet.rs` module in `tokmd-gate` for ratchet evaluation logic
- Comprehensive integration tests for ratchet workflow
- Property-based tests for ratchet evaluation

## [1.4.0] - 2026-01-31

### Added
- **Node.js Bindings**: New `tokmd-node` crate with napi-rs bindings for npm
  - Full API access: `version()`, `schemaVersion()`, `lang()`, `module()`, `export()`, `analyze()`, `diff()`
  - TypeScript definitions included
  - Async/sync variants for all methods
- **Python Bindings**: New `tokmd-python` crate with PyO3 bindings for PyPI
  - Full API access with Pythonic interface
  - Type stubs for IDE support (`py.typed`)
  - Comprehensive test suite
- **FFI Layer**: Enhanced `tokmd-core` with C-compatible FFI functions
  - JSON-based API for language interop
  - Structured error handling with error codes
  - Settings configuration via JSON
- **Version Bump Command**: `cargo xtask bump <VERSION>` for workspace-wide version management
  - Updates all Cargo.toml files atomically
  - Optional `--schema` flag for schema version constants
  - Dry-run mode for previewing changes
- **Complexity Metrics**: Extended complexity analysis in analysis receipts
  - Trend analysis for complexity over time
  - Enhanced JSON schema properties

### Changed
- **MSRV**: Minimum Supported Rust Version bumped to 1.89 (from 1.85)
- **Schema Version**: Analysis receipts now use `schema_version: 4` (from 3)
- **FFI Error Handling**: Improved error formatting and response envelope handling
- **GitHub Action**: Added checksum verification for downloaded assets
- **Nix Flake**: Replaced `cleanCargoSource` with `mkSrc` for improved source filtering
- **cargo-deny**: Updated to version 0.18.6

### Fixed
- **Gate Comparisons**: Fixed string comparison to handle "inf"/"nan" strings correctly without parsing as floats
- **Cockpit**: Use two-dot diff syntax (`A..B`) for accurate line counts when comparing tags/releases

### Internal
- **Documentation**: Added microcrate extraction analysis documents and git diff syntax guidance
- **Test Refactoring**: Improved test assertions for better readability; simplified configuration setup in property tests
- **Proptest Regressions**: Added regression seeds for property-based tests
- **CI**: Updated cargo-deny action to use `taiki-e/install-action` for improved advisory checks
- **Dependencies**: Bumped PyO3 and pyo3-build-config versions

## [1.3.1] - 2026-01-31

### Added
- **ARM Builds**: Release binaries for macOS ARM (M1/M2) and Linux ARM64
- **SHA256 Checksums**: Release artifacts now include `checksums.txt`
- **Shell Completions**: Release includes `completions.tar.gz` with bash/zsh/fish/powershell/elvish
- **Auto-publish**: Release workflow publishes to crates.io automatically
- **Action Test Workflow**: CI workflow to test the GitHub Action on all platforms and formats
- **README Badges**: Downloads, Docs.rs, and GitHub Marketplace badges
- **SECURITY.md**: Security vulnerability reporting policy
- **FUNDING.yml**: GitHub Sponsors configuration
- **CODEOWNERS**: Default code review assignments
- **.editorconfig**: Consistent editor formatting rules
- **Issue Templates**: YAML form-based bug report and feature request templates
- **cargo-deny**: License compliance and security advisory auditing in CI
- **Typos CI**: Spell checking for code and documentation
- **MSRV**: Minimum Supported Rust Version (1.85) documented and tested in CI
- **Homebrew Formula**: `brew tap EffortlessMetrics/tap && brew install tokmd`
- **CITATION.cff**: Academic citation metadata
- **Docker Image**: Multi-arch image at `ghcr.io/effortlessmetrics/tokmd`
- **SLSA Attestations**: Supply chain provenance for release binaries
- **Scoop Manifest**: Windows package manager support
- **WinGet Manifest**: Windows Package Manager support
- **AUR PKGBUILD**: Arch Linux package support

### Changed
- **GitHub Action**: Fail fast on download failure instead of slow cargo fallback
- **GitHub Action**: Added `format` input for export format (json, jsonl, csv)
- **GitHub Action**: Added `artifact` input to control artifact uploads
- **GitHub Action**: Added Marketplace branding (icon, color)
- **GitHub Action**: Removed unused `token` input
- **GitHub Action**: Renamed output from `receipt-json` to `receipt`
- **Release Workflow**: Automatically updates major version tag (v1) on release
- **.gitattributes**: Enhanced with LF normalization and binary file handling

## [1.3.0] - 2026-01-31

### Added
- **Cockpit Command**: `tokmd cockpit` for PR metrics generation with comprehensive evidence gates
  - Change surface analysis (files added/modified/deleted, lines changed)
  - Code composition breakdown (production vs test vs config)
  - Code health metrics (complexity, doc coverage, test coverage)
  - Risk assessment (hotspots, coupling, freshness)
  - Evidence gates (mutation testing, diff coverage, contracts, supply chain, determinism)
  - Review plan generation with prioritized file list
  - Output formats: JSON, Markdown, Sections (for PR templates)
- **Gate Command**: `tokmd gate` for policy-based quality gates with JSON pointer rules and inline policy support
- **Interactive Wizard**: `tokmd init --interactive` for guided project configuration
- **Git-Ranked Context**: `--rank-by churn/hotspot` options in `tokmd context` for git-aware file prioritization
- **Tools Schema**: `tokmd tools` command for generating LLM tool definitions (OpenAI, Anthropic, JSON Schema formats)
- **New Crate**: `tokmd-gate` for policy evaluation with JSON pointer resolution
- **Archetype Detection**: Identify project types (CLI, library, web app, monorepo)
- **Topic Clouds**: TF-IDF semantic analysis of path segments
- **Entropy Profiling**: Detect high-entropy files (potential secrets)
- **Predictive Churn**: Linear regression on commit history for trend detection
- **Corporate Fingerprint**: Author domain statistics from git history
- **License Radar**: SPDX detection from LICENSE files and metadata
- **Context Output Options**: `--out`, `--force`, `--bundle-dir`, `--log`, `--max-output-bytes` flags for flexible output handling
- **CONTRIBUTING.md**: Comprehensive contributor guide with setup instructions, testing strategy, code style, and publishing workflow
- **Fun Feature Variants**: `render_obj` and `render_midi` functions now have feature-gated variants

### Changed
- **Schema Version**: Analysis receipts now use `schema_version: 2`, cockpit receipts use `schema_version: 3`
- **API**: `tokmd_core::scan_workflow` now accepts `redact: Option<RedactMode>` parameter
- **UX**: Non-existent input paths now return an error instead of silent success
- **Feature Flags**: `git`, `walk`, and `content` features are now exposed in CLI crate for lightweight builds
- **Architecture**: Decoupled `tokmd-types` from `tokmd-config`, making `clap` an optional dependency

### Fixed
- **Git Initialization**: Default branch now correctly set to `main` in git repository initialization
- **Redaction Tests**: Fixed test collection to use `Vec` for proper error handling
- **Scan Tests**: Improved error handling in scan integration tests

### Performance
- **Export Streaming**: Reduced allocations in export streaming by using iterators with `Cow`

### Internal
- **Test Robustness**: Replaced `unwrap`/`expect` with `Result` in tests for better error messages
- **Config Determinism**: Locked deterministic ordering in configuration tests
- **Comprehensive Test Suite**: Added integration tests across all major crates (model, format, walk, git, analysis, fun, config, types)
- **Property-Based Tests**: Added proptest coverage for tokmd-redact, tokmd-tokeignore, and tokmd-walk
- **Fuzz Targets**: Added fuzz targets for path redaction and JSON deserialization with dictionaries
- **Mutation Testing**: Added `cargo-mutants` configuration and CI gate for PR quality assurance
  - Enhanced mutation testing workflow with improved file change detection
  - Mutation testing evidence section in cockpit metrics
- **Publish Workflow**: Enhanced `cargo xtask publish` with `--plan`, `--dry-run`, `--from`, `--skip-*` options and Justfile shortcuts
- **CI Improvements**: Added publish plan verification and mutation testing jobs to CI workflow
- **Deprecated API Migration**: Replaced deprecated `cargo_bin` usage with `cargo_bin_cmd` in integration tests

### Documentation
- **Crate READMEs**: Added README.md files for all 17 crates with installation, usage, and API documentation
- **New Troubleshooting Guide**: Comprehensive guide covering common issues, exit codes, performance optimization, and debugging tips
- **CI/CD Integration Recipes**: Added GitHub Actions, GitLab CI, pre-commit hooks, and baseline tracking workflow examples
- **Configuration Reference**: Expanded `tokmd.toml` documentation with full schema, file location precedence, environment variables, and named profiles
- **Tutorial Improvements**: Added Step 11 for troubleshooting missing files with `check-ignore` command
- **Exit Codes Reference**: Documented standard and command-specific exit codes
- **Sorting Clarification**: Clarified that output is automatically sorted (descending by code lines, then path) with no `--sort` flag
- **Bug Fix**: Removed reference to non-existent `--sort code` flag in tutorial
- **Path Error Documentation**: Added troubleshooting section for non-existent path errors
- **CLI Reference**: Documented new context command output flags (`--out`, `--bundle-dir`, `--log`, etc.)

## [1.2.0] - 2026-01-27

### Added
- **Microcrate Architecture**: Split into 16 focused crates for modularity and selective compilation
  - `tokmd-types`, `tokmd-analysis-types` (Tier 0: data structures)
  - `tokmd-scan`, `tokmd-model`, `tokmd-tokeignore`, `tokmd-redact` (Tier 1: core logic)
  - `tokmd-format`, `tokmd-walk`, `tokmd-content`, `tokmd-git` (Tier 2: I/O)
  - `tokmd-analysis`, `tokmd-analysis-format`, `tokmd-fun` (Tier 3: enrichment)
  - `tokmd-config`, `tokmd-core` (Tier 4: orchestration)
  - `tokmd` (Tier 5: CLI binary)
- **Git Integration**: Hotspots, bus factor, freshness, coupling analysis
- **Asset Inventory**: Non-code file categorization and size tracking
- **Dependency Summary**: Lockfile detection and dependency counting
- **Import Graph**: Module dependency analysis with configurable granularity
- **Duplicate Detection**: Content-hash based duplicate file detection
- **CycloneDX Export**: `export --format cyclonedx` generates CycloneDX 1.6 SBOM with file-level components
- **HTML Reports**: `analyze --format html` produces self-contained, offline-capable HTML reports with interactive treemap and sortable tables
- **Context Packing**: New `context` command for LLM context window optimization
  - Budget-aware file selection with `--budget` (e.g., `128k`, `1M`)
  - Multiple strategies: `greedy`, `spread`
  - Output modes: `list`, `bundle`, `json`
- **Redaction Utilities**: New `tokmd-redact` crate centralizes BLAKE3-based path hashing
- **CI Hyper-Testing**: Added proptest smoke tests, mutation testing, and fuzz testing workflows
- **Integration Tests**: Comprehensive `analyze` command smoke tests
- **Check-Ignore Command**: New `check-ignore` command explains why files are being ignored
  - Delegates to `git check-ignore -v` for git-related ignores
  - Shows `.tokeignore` and `--exclude` pattern matches
  - Exit codes: 0=ignored, 1=not ignored
- **Shell Completions**: New `completions` command generates shell completions for bash, zsh, fish, powershell, and elvish

### Changed
- **Feature Flags**: Git, content, and walk features are now opt-in for faster compilation
- **Analysis Limits**: Added `--max-files`, `--max-bytes`, `--max-commits` for resource control

### Fixed
- **RFC3339 Timestamps**: CycloneDX and HTML reports now use proper RFC3339 format via `time` crate
- **Export Bundle Input**: Fixed input path handling in export bundle operations
- **Module Key Computation**: Corrected module key derivation for edge cases

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

## [0.1.0] - 2026-01-25
- Initial prototype release.
