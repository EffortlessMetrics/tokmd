# tokmd Roadmap

This document outlines the evolution of `tokmd` and the path forward.

## Vision

`tokmd` is a **code intelligence platform** that transforms repository scans into actionable insights for humans, machines, and LLMs.

- **Receipt-Grade**: Outputs are deterministic, versioned, and safe for automated pipelines.
- **Analysis-Ready**: Rich derived metrics, git integration, and semantic analysis.
- **LLM-Native**: Designed for context planning, budget estimation, and AI workflows.

---

## Status Summary

| Version | Status | Focus |
| :--- | :--- | :--- |
| **v0.1.0** | ✅ Complete | Basic functionality (scan → model → format). |
| **v0.2.0** | ✅ Complete | Receipt schema, filters, redaction, export logic. |
| **v0.9.0** | ✅ Complete | Integration tests, golden snapshots, edge case verification. |
| **v1.0.0** | ✅ Complete | Schema frozen, release automation, crates.io publish. |
| **v1.1.0** | ✅ Complete | Analysis engine, presets, badge generation, diff command. |
| **v1.2.0** | ✅ Complete | Microcrate architecture, context packing, git integration. |
| **v1.3.0** | ✅ Complete | Advanced enrichers, gate command, interactive wizard. |
| **v1.4.0** | 🚧 In Progress | Complexity metrics, function analysis, PR integration. |
| **v1.5.0** | 🔭 Planned | Cognitive complexity, complexity gates, advanced analysis. |
| **v2.0.0** | 🔭 Planned | MCP server, streaming analysis, plugin system, tree-sitter. |

---

## Completed Milestones

### ✅ v1.0.0 — Stability Release
**Goal**: Production-ready CLI with stable schema contract.

- [x] Receipt schema v1 with `schema_version` field
- [x] Integration tests with `assert_cmd` + `predicates`
- [x] Golden snapshot tests with `insta`
- [x] Cross-platform path normalization
- [x] Redaction (paths, all) with BLAKE3 hashing
- [x] `tokmd run` for artifact generation
- [x] `tokmd diff` for receipt comparison
- [x] Configuration profiles (`tokmd.toml`)
- [x] GitHub Actions release automation
- [x] Formal JSON Schema in `docs/schema.json`

### ✅ v1.1.0 — Analysis Engine
**Goal**: Derived metrics and enrichments beyond raw counts.

- [x] `tokmd analyze` command with preset system
- [x] `tokmd badge` for SVG metric badges
- [x] Derived metrics (doc density, test density, verbosity, nesting, distribution)
- [x] COCOMO effort estimation
- [x] Context window fit analysis
- [x] Reading time estimation
- [x] File size histograms and distributions
- [x] Top offenders (largest, least documented, most dense)
- [x] TODO/FIXME density tracking

---

## Completed: v1.2.0 — Microcrate Architecture

**Goal**: Modular crate structure for selective compilation and ecosystem reuse.

### Crate Hierarchy

| Tier | Crate | Purpose |
| :--- | :--- | :--- |
| 0 | `tokmd-types` | Core data structures, no dependencies |
| 0 | `tokmd-analysis-types` | Analysis receipt types |
| 1 | `tokmd-scan` | tokei wrapper |
| 1 | `tokmd-model` | Aggregation logic |
| 1 | `tokmd-tokeignore` | Template generation |
| 1 | `tokmd-redact` | BLAKE3-based path redaction utilities |
| 2 | `tokmd-format` | Output rendering |
| 2 | `tokmd-walk` | File system traversal |
| 2 | `tokmd-content` | File content scanning |
| 2 | `tokmd-git` | Git history analysis |
| 3 | `tokmd-analysis` | Analysis orchestration |
| 3 | `tokmd-analysis-format` | Analysis output rendering |
| 3 | `tokmd-fun` | Fun/novelty outputs |
| 4 | `tokmd-config` | Configuration loading |
| 4 | `tokmd-core` | Library facade |
| 5 | `tokmd` | CLI binary |

### v1.2.0 Features Delivered

- [x] **Microcrate Architecture**: 16 focused crates for modularity
- [x] **Context Packing**: `tokmd context` command for LLM context window optimization
- [x] **Check-Ignore Command**: `tokmd check-ignore` for troubleshooting ignored files
- [x] **Shell Completions**: `tokmd completions` for bash, zsh, fish, powershell
- [x] **Git Integration**: Hotspots, bus factor, freshness, coupling analysis
- [x] **Asset Inventory**: Non-code file categorization and size tracking
- [x] **Dependency Summary**: Lockfile detection and dependency counting
- [x] **Import Graph**: Module dependency analysis with configurable granularity
- [x] **Duplicate Detection**: Content-hash based duplicate file detection
- [x] **CycloneDX Export**: SBOM generation in CycloneDX 1.6 format
- [x] **HTML Reports**: Self-contained, interactive HTML reports with treemap
- [x] **Redaction Utilities**: Centralized BLAKE3-based path hashing
- [x] **CI Hyper-Testing**: Proptest, mutation testing, and fuzz testing workflows

---

## Completed: v1.3.0 — Polish & Stabilization

**Goal**: Documentation, hardening, gate command, and interactive wizard.

### Analysis Presets

| Preset | Status | Includes |
| :--- | :--- | :--- |
| `receipt` | ✅ | Core derived metrics |
| `health` | ✅ | TODO density + derived |
| `risk` | ✅ | Git hotspots, coupling, freshness |
| `supply` | ✅ | Assets + dependency lockfile summary |
| `architecture` | ✅ | Import graph analysis |
| `topics` | ✅ | Semantic topic clouds (TF-IDF) |
| `security` | ✅ | License radar + entropy profiling |
| `identity` | ✅ | Archetype + corporate fingerprint |
| `git` | ✅ | Predictive churn + git metrics |
| `deep` | ✅ | Everything (except fun) |
| `fun` | ✅ | Eco-label, novelty outputs |

### v1.3.0 Features Delivered

- [x] **Cockpit Command**: `tokmd cockpit` for PR metrics generation with evidence gates
  - Change surface analysis (files added/modified/deleted, lines changed)
  - Code composition breakdown (production vs test vs config)
  - Risk assessment and review plan generation
  - Evidence gates: mutation testing, diff coverage, contracts, supply chain, determinism
- [x] **Gate Command**: `tokmd gate` for policy-based quality gates with JSON pointer rules
- [x] **Interactive Wizard**: `tokmd init --interactive` for guided project setup
- [x] **Git-Ranked Context**: `--rank-by churn/hotspot` in `tokmd context` command
- [x] **Tools Schema**: `tokmd tools` for LLM tool definitions (OpenAI, Anthropic, JSON Schema)
- [x] **Context Output Options**: `--out`, `--force`, `--bundle-dir`, `--log`, `--max-output-bytes` flags
- [x] **Documentation**: README files for all 17 crates
- [x] **Documentation**: Updated troubleshooting guide with new error behaviors
- [x] **Documentation**: Updated CLI reference with exit code changes
- [x] **Documentation**: CONTRIBUTING.md guide with setup, testing, and publishing workflow
- [x] **Performance**: Reduced allocations in export streaming with `Cow` iterators
- [x] **Stability**: Non-existent input paths now error instead of silent success
- [x] **Stability**: Improved error handling in tests (Result instead of unwrap/expect)
- [x] **Architecture**: Decoupled `tokmd-types` from `tokmd-config` (clap now optional)
- [x] **Architecture**: Exposed `git`/`walk`/`content` feature flags in CLI for lightweight builds
- [x] **Architecture**: New `tokmd-gate` crate for policy evaluation
- [x] **Testing**: Comprehensive integration tests across all major crates
- [x] **Testing**: Property-based tests for tokmd-redact, tokmd-tokeignore, tokmd-walk
- [x] **Testing**: Fuzz targets for path redaction and JSON deserialization
- [x] **Testing**: Mutation testing with cargo-mutants and CI gate
- [x] **CI/CD**: Enhanced publish workflow via `cargo xtask publish`

---

## In Progress: v1.4.0 — Complexity Metrics

**Goal**: Function-level analysis and basic complexity metrics for code health assessment.

### Complexity Metrics

| Feature | Status | Description |
| :--- | :--- | :--- |
| Function count/length metrics | 🚧 In Progress | Count functions per file, track average/max function length |
| Simplified cyclomatic complexity | 🚧 In Progress | Heuristic-based branching complexity (if/else/switch/loop counting) |
| Complexity histogram | 📋 Planned | Distribution of complexity scores across codebase |
| Complexity top offenders | 📋 Planned | Identify most complex functions/files |

### PR Integration

| Feature | Status | Description |
| :--- | :--- | :--- |
| GitHub Actions workflow for cockpit | 📋 Planned | Reusable workflow for PR analysis |
| Automatic PR comment injection | 📋 Planned | Post cockpit metrics as PR comments via `gh` CLI |
| PR template generation | 📋 Planned | Generate PR template with metrics placeholders |

---

## Planned: v1.5.0 — Advanced Complexity

**Goal**: Cognitive complexity and complexity-based quality gates.

### Complexity Metrics

| Feature | Status | Description |
| :--- | :--- | :--- |
| Cognitive complexity | 📋 Planned | SonarQube-style cognitive complexity scoring |
| Complexity gate in cockpit | 📋 Planned | Fail PR review if complexity exceeds threshold |
| Complexity delta tracking | 📋 Planned | Track complexity changes between commits |
| Complexity trend analysis | 📋 Planned | Historical complexity evolution |

---

## Future Horizons

### v2.0 — Platform Evolution

#### A. Tree-sitter Integration
*Goal: Accurate parsing for precise complexity metrics.*

- tree-sitter integration for multi-language AST parsing
- Language-specific complexity rules (Rust, TypeScript, Python, Go, etc.)
- Accurate function boundary detection
- Nested scope analysis for cognitive complexity
- Call graph extraction for coupling analysis

#### B. Language Bindings (FFI)
*Goal: Native integration in CI pipelines and tooling ecosystems.*

**Python (PyPI: `tokmd`)**
- Native bindings via PyO3 + maturin
- Crate: `tokmd-python/`
- API: `tokmd.scan()`, `tokmd.analyze()`, `tokmd.diff()`
- Returns native Python dicts/dataclasses
- Wheels for Linux, macOS, Windows (x64 + arm64)

**Node.js (npm: `@tokmd/core`)**
- Native bindings via napi-rs
- Crate: `tokmd-node/`
- API: `scan()`, `analyze()`, `diff()` returning JS objects
- Prebuilds for major platforms

**Shared Infrastructure**
- `tokmd-ffi/` crate with C-compatible interface
- JSON serialization at FFI boundary for simplicity
- CI matrix for cross-platform wheel/prebuild generation

#### C. MCP Server Mode
*Goal: Native integration with Claude and other MCP-compatible clients.*

- `tokmd serve` — Start MCP server for tool-based interaction
- Resources: Expose receipts as MCP resources
- Tools: `scan`, `analyze`, `diff`, `suggest` as MCP tools
- Streaming: Incremental analysis results

#### D. Streaming Analysis
*Goal: Handle massive repositories without memory pressure.*

- JSONL streaming for all outputs
- Incremental file processing
- Memory-bounded analysis limits
- Progress reporting via stderr

#### E. Plugin System
*Goal: Extensible enrichers without core changes.*

- WASM plugin interface for custom analyzers
- Plugin discovery from `~/.tokmd/plugins/`
- Schema for plugin metadata and capabilities

### v2.1 — Intelligence Features

#### F. Smart Suggestions
*Goal: Actionable recommendations, not just metrics.*

- `tokmd suggest --budget 128k` — Files to include for context
- `tokmd suggest --review` — Files likely to need attention
- `tokmd suggest --test` — Untested code paths

#### G. Diff Intelligence
*Goal: Semantic diff beyond structural changes.*

- Complexity delta detection
- Breaking change indicators
- Migration path suggestions

#### H. Watch Mode
*Goal: Continuous analysis during development.*

- `tokmd watch` — Re-analyze on file changes
- Integration with LSP for editor feedback
- Real-time metric updates

### v2.2 — Ecosystem Integration

#### I. CI/CD Native
*Goal: First-class CI pipeline support.*

- GitHub Action with PR comments
- GitLab CI template
- Trend tracking across commits
- Threshold-based failures (e.g., fail if complexity increases)

#### J. Editor Extensions
*Goal: Analysis at your fingertips.*

- VS Code extension with inline metrics
- Neovim plugin for buffer analysis
- JetBrains plugin

#### K. Cloud Dashboard
*Goal: Historical tracking and team insights.*

- Receipt aggregation service
- Trend visualization
- Team comparison views
- Alert on anomalies

---

## Non-Goals

These are explicitly out of scope for tokmd:

- **Code formatting/linting** — Use dedicated tools (rustfmt, eslint)
- **Dependency vulnerability scanning** — Use cargo-audit, npm audit
- **Test execution** — Use cargo test, pytest, jest
- **Build orchestration** — Use cargo, make, just
- **Full AST analysis** — tokmd uses heuristics, not parsers (until v2.0 tree-sitter integration)

---

## Contributing

Contributions welcome! Priority areas:

1. **Enricher implementations** — See `crates/tokmd-analysis/src/` for patterns
2. **Output format templates** — Markdown templates in `tokmd-analysis-format`
3. **Language support** — Extend import graph parsing
4. **Documentation** — Recipe examples and use cases

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
