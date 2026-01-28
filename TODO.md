# TODO

## Completed

### v1.0.0 — Production Readiness
- [x] Formal JSON schema for `lang`, `module`, and `export`
- [x] `schema_version`, `tool`, `inputs` metadata
- [x] `export` subcommand with JSONL and CSV formats
- [x] Filters: `--min-code`, `--max-rows`
- [x] Redaction: `--redact paths`, `--redact all`
- [x] Unified `--children` flag behavior
- [x] Integration tests with `insta` snapshots
- [x] Diataxis documentation structure
- [x] Recipe book and schema reference

### v1.1.0 — Analysis Engine
- [x] `tokmd analyze` command with preset system
- [x] `tokmd badge` for SVG metric badges
- [x] `tokmd diff` for receipt comparison
- [x] `tokmd run` for artifact generation
- [x] Derived metrics (doc density, test density, distribution)
- [x] COCOMO effort estimation
- [x] Context window fit analysis
- [x] TODO/FIXME density tracking
- [x] Configuration profiles (`tokmd.toml`)
- [x] GitHub Action for CI integration
- [x] Binary releases via GitHub Actions

### v1.2.0 — Microcrate Architecture
- [x] Split into 15 focused crates
- [x] Feature flags for optional dependencies
- [x] Git integration (hotspots, freshness, coupling, bus factor)
- [x] Asset inventory
- [x] Dependency lockfile summary
- [x] Import graph analysis
- [x] Duplicate detection

## In Progress

### Enrichers (v1.2.x)
- [ ] **Archetype Detection**: Identify project types (CLI, library, web app, monorepo)
- [ ] **Topic Clouds**: TF-IDF semantic analysis of path segments
- [ ] **Entropy Profiling**: Detect high-entropy files (potential secrets)
- [ ] **Predictive Churn**: Linear regression on commit history
- [ ] **Corporate Fingerprint**: Author domain statistics
- [ ] **License Radar**: SPDX detection from LICENSE files and metadata

## Future

### v2.0 — Platform Evolution
- [ ] **MCP Server Mode**: `tokmd serve` for Claude/MCP integration
- [ ] **Streaming Analysis**: JSONL streaming for large repos
- [ ] **Plugin System**: WASM-based extensible enrichers

### v2.1 — Intelligence Features
- [ ] **Smart Suggestions**: `tokmd suggest --budget 128k`
- [ ] **Diff Intelligence**: Complexity delta, breaking change detection
- [ ] **Watch Mode**: `tokmd watch` for continuous analysis

### v2.2 — Ecosystem Integration
- [ ] **CI/CD Native**: PR comments, trend tracking, threshold gates
- [ ] **Editor Extensions**: VS Code, Neovim, JetBrains plugins
- [ ] **Cloud Dashboard**: Historical tracking and team insights
