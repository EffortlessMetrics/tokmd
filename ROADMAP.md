# tokmd Roadmap

This document outlines the evolution of `tokmd` and the path forward.

## Vision

`tokmd` is a **lightweight code intelligence tool** that transforms repository scans into actionable insights for humans, machines, and LLMs.

- **Receipt-Grade**: Outputs are deterministic, versioned, and safe for automated pipelines.
- **Analysis-Ready**: Rich derived metrics, git integration, and semantic analysis.
- **LLM-Native**: Designed for context planning, budget estimation, and AI workflows.

---

## Status Summary

| Version    | Status      | Focus                                                        |
| :--------- | :---------- | :----------------------------------------------------------- |
| **v0.1.0** | ✅ Complete | Basic functionality (scan → model → format).                 |
| **v0.2.0** | ✅ Complete | Receipt schema, filters, redaction, export logic.            |
| **v0.9.0** | ✅ Complete | Integration tests, golden snapshots, edge case verification. |
| **v1.0.0** | ✅ Complete | Schema frozen, release automation, crates.io publish.        |
| **v1.1.0** | ✅ Complete | Analysis engine, presets, badge generation, diff command.    |
| **v1.2.0** | ✅ Complete | Microcrate architecture, context packing, git integration.   |
| **v1.3.0** | ✅ Complete | Advanced enrichers, gate command, interactive wizard.        |
| **v1.4.0** | ✅ Complete | Complexity metrics, cognitive complexity, PR integration.    |
| **v1.5.0** | ✅ Complete | Baseline system, ratchet gates, ecosystem envelope, LLM handoff. |
| **v1.6.0** | ✅ Complete | Halstead metrics, maintainability index, sensor envelope, cockpit overhaul. |
| **v1.6.3** | ✅ Complete | UX polish: colored diff, progress indicators, --explain flag.    |
| **v1.7.0** | ✅ Complete | Near-duplicate detection, commit intent, token estimation renames. |
| **v1.7.1** | ✅ Complete | Focused microcrate extraction, FFI-envelope reuse, and sharper tier boundaries. |
| **v1.7.2** | ✅ Complete | Near-dup enricher extraction, commit intent classification, and CI fixes. |
| **v1.7.x** | ✅ Complete | Deep test expansion across the workspace, sensor determinism, and the first `tokmd-io-port` seam. |
| **v1.8.0** | ✅ Complete | Effort estimation, estimate preset/reporting, `tokmd-io-port` seam work, and release/devex hardening. |
| **v1.9.0** | 🚧 In progress | Browser/WASM productization: parity-covered wasm entrypoints, browser runner MVP, and public repo ingestion via tree+contents |
| **v2.0.0** | 🔭 Planned  | MCP server, streaming analysis, plugin system.               |
| **v3.0.0** | 🔭 Long-term | Tree-sitter AST integration (requires significant R&D).      |
| **v4.0.0** | 🔭 Long-term | Adze AST integration.      |

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

| Tier | Crate                   | Purpose                               |
| :--- | :---------------------- | :------------------------------------ |
| 0    | `tokmd-types`           | Core data structures, no dependencies |
| 0    | `tokmd-analysis-types`  | Analysis receipt types                |
| 0    | `tokmd-settings`        | Clap-free settings types              |
| 0    | `tokmd-envelope`        | Cross-fleet sensor report contract    |
| 0    | `tokmd-substrate`       | Shared repo context (`RepoSubstrate`) |
| 1    | `tokmd-scan`            | tokei wrapper                         |
| 1    | `tokmd-model`           | Aggregation logic                     |
| 1    | `tokmd-tokeignore`      | Template generation                   |
| 1    | `tokmd-redact`          | BLAKE3-based path redaction utilities |
| 1    | `tokmd-sensor`          | `EffortlessSensor` trait + builder    |
| 2    | `tokmd-format`          | Output rendering                      |
| 2    | `tokmd-walk`            | File system traversal                 |
| 2    | `tokmd-content`         | File content scanning                 |
| 2    | `tokmd-git`             | Git history analysis                  |
| 3    | `tokmd-analysis`        | Analysis orchestration                |
| 3    | `tokmd-analysis-format` | Analysis output rendering             |
| 3    | `tokmd-fun`             | Fun/novelty outputs                   |
| 3    | `tokmd-gate`            | Policy evaluation with JSON pointer   |
| 4    | `tokmd-config`          | Configuration loading                 |
| 4    | `tokmd-core`            | Library facade with FFI layer         |
| 5    | `tokmd`                 | CLI binary                            |
| 5    | `tokmd-python`          | Python bindings (PyO3)                |
| 5    | `tokmd-node`            | Node.js bindings (napi-rs)            |

### v1.2.0 Features Delivered

- [x] **Microcrate Architecture**: Focused crates for modularity (16 initial crates; now 58 crate members in the workspace by `1.8.0`)
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

| Preset         | Status | Includes                             |
| :------------- | :----- | :----------------------------------- |
| `receipt`      | ✅     | Core derived metrics                 |
| `health`       | ✅     | TODO density + derived               |
| `risk`         | ✅     | Git hotspots, coupling, freshness    |
| `supply`       | ✅     | Assets + dependency lockfile summary |
| `architecture` | ✅     | Import graph analysis                |
| `topics`       | ✅     | Semantic topic clouds (TF-IDF)       |
| `security`     | ✅     | License radar + entropy profiling    |
| `identity`     | ✅     | Archetype + corporate fingerprint    |
| `git`          | ✅     | Predictive churn + git metrics       |
| `deep`         | ✅     | Everything (except fun)              |
| `fun`          | ✅     | Eco-label, novelty outputs           |

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

## Completed: v1.4.0 — Complexity Metrics & PR Integration

**Goal**: Function-level analysis, complexity metrics, and PR template integration.

### Complexity Metrics

| Feature                       | Status      | Description                                                         |
| :---------------------------- | :---------- | :------------------------------------------------------------------ |
| Function count/length metrics | ✅ Complete | Count functions per file, track average/max function length         |
| Cyclomatic complexity         | ✅ Complete | Heuristic-based branching complexity (if/else/switch/loop counting) |
| Cognitive complexity          | ✅ Complete | SonarQube-style cognitive complexity with nesting penalty           |
| Nesting depth analysis        | ✅ Complete | Track max/avg nesting depth per file                                |
| Complexity top offenders      | ✅ Complete | Identify most complex functions/files                               |
| Extended language support     | ✅ Complete | Rust, Python, JS/TS, Go, C, C++, Java, C#                           |

### PR Integration

| Feature                              | Status      | Description                                                        |
| :----------------------------------- | :---------- | :----------------------------------------------------------------- |
| GitHub Actions workflow with caching | ✅ Complete | Reusable workflow with Rust caching for faster builds              |
| Baseline trend comparison            | ✅ Complete | `--baseline` flag for tracking metric trends                       |
| PR template with trend section       | ✅ Complete | Template with TREND section markers                                |
| Automatic PR comment injection       | ✅ Complete | Post cockpit metrics via `thollander/actions-comment-pull-request` |

### Schema Changes

- **Analysis schema version**: 3 → 4
- **New fields in `ComplexityReport`**: `avg_cognitive`, `max_cognitive`, `avg_nesting_depth`, `max_nesting_depth`
- **New fields in `FileComplexity`**: `cognitive_complexity`, `max_nesting`, `functions`
- **New type**: `FunctionComplexityDetail` for function-level metrics
- **New cockpit types**: `TrendComparison`, `TrendMetric`, `TrendIndicator`, `TrendDirection`

---

## Completed: v1.5.0 — Baseline & Ratchet System

**Goal**: Baseline storage and ratchet-based quality gates.

### Baseline System

| Feature                  | Status      | Description                                                  |
| :----------------------- | :---------- | :----------------------------------------------------------- |
| Baseline storage         | ✅ Complete | `.tokmd/baseline.json` for storing complexity baseline       |
| `tokmd baseline` command | ✅ Complete | Generate baseline from current state                         |
| Baseline types           | ✅ Complete | `ComplexityBaseline`, `BaselineMetrics`, `FileBaselineEntry` |
| Baseline JSON Schema     | ✅ Complete | `docs/baseline.schema.json` formal definition                |

### Ratchet Rules

| Feature                       | Status      | Description                                        |
| :---------------------------- | :---------- | :------------------------------------------------- |
| Ratchet rules in `tokmd.toml` | ✅ Complete | `[[gate.ratchet]]` configuration                   |
| Ratchet evaluation            | ✅ Complete | `evaluate_ratchet()` in tokmd-gate                 |
| Max increase percentage       | ✅ Complete | `max_increase_pct` field for gradual improvement   |
| Max value ceiling             | ✅ Complete | `max_value` field for absolute ceiling enforcement |
| Gate integration              | ✅ Complete | `--baseline` and `--ratchet-config` CLI flags      |

### Ecosystem Envelope

| Feature             | Status      | Description                                       |
| :------------------ | :---------- | :------------------------------------------------ |
| Envelope types      | ✅ Complete | `Envelope`, `Finding`, `GatesEnvelope`, `Verdict` |
| Finding ID registry | ✅ Complete | `tokmd.<category>.<code>` format constants        |
| Builder APIs        | ✅ Complete | Fluent API for constructing envelopes             |

---

## Completed: v1.6.0 — Advanced Complexity & Sensor Envelope

**Goal**: Deeper complexity analysis, sensor envelope, and cockpit overhaul.

### Complexity Features

| Feature                | Status      | Description                                          |
| :--------------------- | :---------- | :--------------------------------------------------- |
| Halstead metrics       | ✅ Complete | Feature-gated (`halstead`) Halstead software science metrics |
| Function detail export | ✅ Complete | `--detail-functions` flag for function-level output  |
| Complexity histogram   | ✅ Complete | Wired into analysis pipeline from pre-existing implementation |
| Complexity gates       | ✅ Complete | Shipped in cockpit evidence gate system              |

### Sensor & Envelope

| Feature                  | Status      | Description                                          |
| :----------------------- | :---------- | :--------------------------------------------------- |
| `tokmd sensor` command   | ✅ Complete | Conforming sensor producing `sensor.report.v1` envelope |
| `tokmd-sensor` crate     | ✅ Complete | `EffortlessSensor` trait + substrate builder          |
| `tokmd-envelope` crate   | ✅ Complete | Cross-fleet `SensorReport` contract with verdicts    |
| `tokmd-substrate` crate  | ✅ Complete | Shared `RepoSubstrate` for single-I/O-pass sensors  |
| `tokmd-settings` crate   | ✅ Complete | Clap-free settings types for library/FFI usage       |

### Derived Metrics

| Feature                   | Status      | Description                                               |
| :------------------------ | :---------- | :-------------------------------------------------------- |
| Maintainability Index     | ✅ Complete | SEI formula (simplified without Halstead, full with)      |
| Technical debt ratio      | ✅ Complete | Complexity-to-size ratio as a heuristic debt signal       |
| Duplication density       | ✅ Complete | Extend duplicate detection into a per-module density metric |
| API surface area          | ✅ Complete | Public export ratio (language-specific heuristics) via `tokmd-analysis-api-surface` |
| Code age distribution     | ✅ Complete | Extend git freshness into age buckets with trend tracking |

### Cockpit & CLI Improvements

| Feature                     | Status      | Description                                             |
| :-------------------------- | :---------- | :------------------------------------------------------ |
| Diff coverage overhaul      | ✅ Complete | LCOV intersected with git-added lines for accurate coverage |
| `get_added_lines()` in git  | ✅ Complete | New API for per-file added-line extraction from git diff |
| CLI arg normalization       | ✅ Complete | `--out` → `--output` (with backward-compatible alias)   |
| Rust fn regex compliance    | ✅ Complete | `(_\|XID_Start) XID_Continue*` per Rust spec            |
| Cross-platform docs         | ✅ Complete | xtask docs normalizes `tokmd.exe` → `tokmd`, CRLF → LF |
| Docs integration test       | ✅ Complete | Automated reference-cli.md freshness verification       |

### Schema Changes

- **Analysis schema version**: 4 → 5
- **New types**: `HalsteadMetrics`, `MaintainabilityIndex`
- **New fields in `ComplexityReport`**: `halstead`, `maintainability_index`, `histogram` (now populated)
- **New CLI flag**: `--detail-functions` on `tokmd analyze`
- **New feature flag**: `halstead` in `tokmd-analysis`
- **Cockpit gates completed**: diff coverage (lcov), semver checks, schema diff
- **Handoff complexity**: Real data from file analysis (replaces heuristic)
- **New crates**: `tokmd-sensor`, `tokmd-settings`, `tokmd-envelope`, `tokmd-substrate`

---

## Completed: v1.6.3 — UX & Output Quality

**Goal**: Improve the developer experience for interactive CLI usage and output readability.

### Output Improvements

| Feature                   | Status      | Description                                               |
| :------------------------ | :---------- | :-------------------------------------------------------- |
| Colored diff output       | ✅ Complete | Terminal colors in `tokmd diff` for additions/removals    |
| Summary comparison tables | ✅ Complete | Side-by-side metric comparisons in diff and cockpit       |
| Compact table mode        | ✅ Complete | `--compact` flag for narrow terminals (elide zero columns) |
| Sparkline trends          | ✅ Complete | Inline unicode sparklines for metric trends in markdown   |

### Interactive Experience

| Feature                   | Status      | Description                                               |
| :------------------------ | :---------- | :-------------------------------------------------------- |
| Progress indicators       | ✅ Complete | Spinner/progress bar for long scans via `indicatif`       |
| Structured error messages | ✅ Complete | Actionable hints on common failures (missing git, bad paths) |
| `--explain` flag          | ✅ Complete | Human-readable explanation of any metric or finding       |
| Tab completion for flags  | ✅ Complete | Dynamic completions for `--preset`, `--format`, etc.      |

### Scope Notes

UX work is explicitly **incremental and non-breaking**:
- No changes to JSON/JSONL receipt schemas (these are machine surfaces)
- Terminal enhancements are opt-in and degrade gracefully on dumb terminals
- Progress output goes to stderr, never stdout (preserving pipe-ability)
- Color respects `NO_COLOR` / `CLICOLOR` environment conventions

### v1.6.3 Features Delivered

- [x] Extracted `tokmd-progress` microcrate for CLI progress rendering primitives
- [x] Extracted `tokmd-badge` microcrate for SVG badge generation
- [x] Added side-by-side summary comparison rows for diff totals (LOC, lines, files, bytes, tokens)
- [x] Added baseline-aware summary comparison tables to cockpit markdown output
- [x] Added integration tests to lock dynamic completion values for `--preset` and `--format`

---

## Completed: v1.7.0 — Near-Duplicate Detection & Commit Intent

**Goal**: Near-duplicate detection, commit intent classification, and token estimation improvements.

### Near-Duplicate Detection

| Feature                    | Status      | Description                                                  |
| :------------------------- | :---------- | :----------------------------------------------------------- |
| Near-dup enricher          | ✅ Complete | Content-similarity detection via `tokmd-analysis-near-dup`   |
| `--near-dup` flag          | ✅ Complete | Enable near-duplicate detection in analysis                  |
| `--near-dup-threshold`     | ✅ Complete | Configurable similarity threshold (default 0.8)              |
| `--near-dup-scope`         | ✅ Complete | Scope filter for near-dup scanning                           |
| `--near-dup-max-files`     | ✅ Complete | Max file guardrail for performance                           |

### Git Enrichments

| Feature                    | Status      | Description                                                  |
| :------------------------- | :---------- | :----------------------------------------------------------- |
| Commit intent classification | ✅ Complete | Automatic classification of commit purpose (feat/fix/refactor/etc.) |
| Coupling metrics           | ✅ Complete | Jaccard similarity and Lift in coupling reports              |
| Commit SHA field           | ✅ Complete | `hash` field on `GitCommit` for identification               |

### Token Estimation

| Feature                    | Status      | Description                                                  |
| :------------------------- | :---------- | :----------------------------------------------------------- |
| Field renames              | ✅ Complete | `tokens_low`/`tokens_high` → `tokens_min`/`tokens_max`      |
| Backward compatibility     | ✅ Complete | Serde aliases preserve deserialization of old field names     |
| Divisor fields             | ✅ Complete | Explicit `bytes_per_token_low`/`bytes_per_token_high` fields |

### Schema Changes

- **Analysis schema version**: 6 → 7
- **New types**: `NearDuplicateReport`, `NearDupCluster`, `NearDupPair`, `CommitIntentKind`
- **New fields**: `coupling.jaccard`, `coupling.lift`, `git_commit.hash`
- **Renamed fields**: `tokens_low` → `tokens_min`, `tokens_high` → `tokens_max` (with serde aliases)

---

## Completed: v1.7.1 — Focused Microcrate Extraction

**Goal**: Extract focused microcrates from monolithic modules for better separation of concerns.

### New Microcrates

| Crate                          | Tier | Purpose                                                |
| :----------------------------- | :--- | :----------------------------------------------------- |
| `tokmd-context-policy`         | 1    | Context/handoff policy helpers (smart excludes, classification) |
| `tokmd-scan-args`              | 1    | Deterministic `ScanArgs` metadata construction         |
| `tokmd-math`                   | 1    | Deterministic numeric/statistical helpers              |
| `tokmd-exclude`                | 1    | Exclude-pattern normalization + dedup                  |
| `tokmd-path`                   | 1    | Cross-platform path normalization                      |
| `tokmd-module-key`             | 1    | Deterministic module-key derivation                    |
| `tokmd-context-git`            | 2    | Git-derived hotspot/churn scoring for context ranking  |
| `tokmd-export-tree`            | 2    | Deterministic tree renderers for analysis/handoff exports |
| `tokmd-analysis-explain`       | 3    | Metric/finding explanation catalog and alias lookup    |
| `tokmd-analysis-imports`       | 3    | Language-aware import parsing + normalization          |
| `tokmd-analysis-maintainability` | 3  | Maintainability index scoring + Halstead merge         |
| `tokmd-analysis-html`          | 3    | Single-responsibility HTML renderer for analysis       |
| `tokmd-tool-schema`            | 4    | AI tool-schema generation from clap command trees      |
| `tokmd-ffi-envelope`           | 4    | Shared FFI envelope parser for Python/Node bindings    |

### Architectural Changes

- [x] Moved `AnalysisFormat` to `tokmd-types` (Tier 0) for broader reuse
- [x] Extracted 15 focused microcrates from monolithic modules
- [x] Analysis schema version: 7 → 8
- [x] Workspace graph continued to expand beyond the original 16-crate v1.2.0 layout and now sits at 58 crate members in `1.8.0`
- [x] Fixed clippy/lint across all new crates for strict `--all-targets` check coverage
- [x] Updated CI/tooling for release and publish readiness

---

## Completed: v1.7.x — Deep Test Coverage Expansion

**Goal**: Achieve broad, multi-strategy test coverage across the workspace without breaking deterministic or release-facing surfaces.

### Test Numbers

| Metric | Current framing |
| :----- | :-------------- |
| Test depth | Expanded across unit, integration, snapshot, deep, property, fuzz, and mutation layers |
| Workspace reach | Coverage spread across essentially the full crate graph, including CLI and binding-facing seams |
| Determinism focus | Receipt stability, schema contracts, and cross-crate invariants locked in by dedicated suites |

### Coverage by Tier

| Tier | Crates Covered | Test Types Added |
| :--- | :------------- | :--------------- |
| 0 | `tokmd-types`, `tokmd-analysis-types`, `tokmd-settings`, `tokmd-envelope`, `tokmd-substrate` | Determinism regression, contract expansion, boundary props |
| 1 | `tokmd-scan`, `tokmd-model`, `tokmd-redact`, `tokmd-context-policy`, `tokmd-scan-args`, `tokmd-math`, `tokmd-path`, `tokmd-module-key`, `tokmd-exclude` | Property tests, deep proptests, snapshot suites |
| 2 | `tokmd-format`, `tokmd-walk`, `tokmd-content`, `tokmd-git`, `tokmd-badge`, `tokmd-export-tree`, `tokmd-context-git` | Snapshot tests for all renderers, traversal properties |
| 3 | All `tokmd-analysis-*` microcrates, `tokmd-gate`, `tokmd-fun` | BDD scenarios, enricher contract verification, deep proptests |
| 4 | `tokmd-core`, `tokmd-config`, `tokmd-tool-schema`, `tokmd-ffi-envelope` | FFI workflow integration, JSON API round-trip tests |
| 5 | `tokmd` CLI | E2E tests for `lang`, `module`, `export`, `run`, `analyze`, `diff`, `badge`, `gate`, `cockpit`, `context`, `handoff`, `sensor`, and `baseline` |

### What Landed (36+ PRs first wave, 16 PRs second wave)

- [x] Boundary verification tests across core types
- [x] Determinism regression tests for all receipt-producing paths
- [x] Byte-stable output regression suite with ordering locks
- [x] Error handling coverage for edge cases and malformed inputs
- [x] Snapshot tests (`insta`) for all format renderers (Markdown, TSV, JSON, HTML)
- [x] Deep analysis crate tests: complexity, halstead, near-dup, topics, entropy, license, archetype, fingerprint, API surface
- [x] CLI E2E tests for the core scan, analysis, review, sensor, and LLM-bundle commands
- [x] FFI and workflow integration tests in `tokmd-core`
- [x] Property tests expanded across 14+ crates with `proptest`
- [x] 3 new fuzz targets (import parser, export tree, policy TOML)
- [x] BDD-style scenario tests (`tests/bdd.rs`) in every `tokmd-analysis-*` crate
- [x] Doctest coverage expanded across crates

### CI & Performance

- [x] CI green on main with full mutation testing gate passing
- [x] macOS jobs gated to main-only pushes for CI cost control (#409)
- [x] Nix CI fixes: resolved `cloned_ref_to_slice_refs` clippy lint for cargo 1.93 (#407)
- [x] Fix-forward for typo, rustfmt, and content test failures (#390)
- [x] Reduced allocations in token stream formatting (perf improvement)

---

## Completed: v1.8.0 — Effort Estimation & Release Hardening

**Goal:** Expand `tokmd analyze` with first-class effort estimation while hardening the repo-native operator surface for CI, Windows, and release prep.

### What landed

- [x] **Effort estimation engine**: new `tokmd-analysis-effort` crate with COCOMO 81, COCOMO II, and Monte Carlo scaffolding.
- [x] **Estimate preset and receipt/report support**: effort outputs now flow through analysis receipts and Markdown renderers.
- [x] **Preset grid expansion**: the analysis surface now exposes 12 presets, with `estimate` joining a stronger `receipt` baseline.
- [x] **Schema evolution**: analysis schema advanced to v9 to carry effort estimation data.
- [x] **WASM seam foundation**: `tokmd-io-port` landed with `ReadFs`, `HostFs`, and `MemFs` as the host-abstracted file access boundary.
- [x] **Windows-safe repo-native quality path**: repo-native fmt and publish flows avoid Windows `xtask.exe` self-lock and `cargo fmt --all` pain.
- [x] **Build-footprint reduction**: `cargo trim-target`, leaner Windows debug info defaults, and opt-in `sccache` support reduce local rebuild churn.
- [x] **CI/release boringness**: workflow concurrency, smarter Rust caching, Node 24 Nix canary, and a clean tag-driven `1.8.0` release through GitHub Actions.

### Notes

- The full in-memory scan path and wasm CI parity work did not fully land in `1.8.0`; that continuation is now the next milestone instead of implicit spillover.

## In Progress: v1.9.x — Browser/WASM Productization

**Goal:** Finish the browser/WASM product surface around the already-landed in-memory execution path and make the supported browser workflow explicit, repeatable, and capability-honest.

### v1.9.0 — WASM Foundation & Parity

**Theme:** Core WASM build-out and parity coverage.

#### Completed
- [x] `tokmd-io-port` landed with `ReadFs`, `HostFs`, and `MemFs` host-abstracted file access boundary
- [x] In-memory scan/model/core workflow seams and lower-tier clap-free boundaries for browser/WASM execution
- [x] `tokmd-wasm` exposes browser-friendly entrypoints for `lang`, `module`, `export`, and browser-safe `analyze`
- [x] Native-vs-wasm parity coverage for `lang`, `module`, `export`, `analyze receipt`, and `analyze estimate`
- [x] `web/runner` boots real `tokmd-wasm` bundle in dedicated worker with capability reporting
- [x] Public GitHub repo acquisition via browser-safe tree + contents APIs (not zipball)

#### v1.9.0 Remaining
- [ ] **Docs truth pass** — README and architecture docs match shipped browser/WASM surface
- [ ] **Document WASM limitations explicitly** — Browser/WASM capability matrix (supported commands/presets, unavailable enrichers, rootless constraints)

### v1.9.1 — Browser UX Hardening

**Theme:** Production-ready browser experience with guardrails and performance.

- [ ] **In-browser caching layer** — Cache parsed repo trees and receipt outputs in IndexedDB
- [ ] **Progress indicators** — Visual progress for large repo ingestion and analysis
- [ ] **Rate-limit handling** — Exponential backoff for GitHub API limits with user-facing messages
- [ ] **Authenticated fetch options** — Support for private repos via GitHub token input
- [ ] **Error boundary hardening** — Graceful degradation when WASM panics or fetch fails
- [ ] **Mobile viewport optimization** — Responsive layout for phone/tablet usage

### v1.9.2 — Analysis Expansion

**Theme:** Expand browser-safe analysis where presets can stay rootless.

- [ ] **Additional analyze presets** — `health`, `supply` presets if they can operate without host-backed enrichers
- [ ] **Content scanning in-browser** — Entropy detection, TODO/FIXME scanning on in-memory content
- [ ] **Import graph analysis** — Parse imports from in-memory source (no filesystem needed)
- [ ] **Badge generation in-browser** — SVG badge rendering client-side
- [ ] **Export formats** — JSONL, CSV export from browser runner

### v1.9.3 — Integration & Tooling

**Theme:** Developer experience and ecosystem integration.

- [ ] **Embed API** — Documented JS API for embedding tokmd-wasm in other web apps
- [ ] **npm package publish** — `@tokmd/wasm` or `@tokmd/browser` package
- [ ] **TypeScript definitions** — Full type coverage for browser API
- [ ] **vite/webpack integration guide** — Bundler configuration examples
- [ ] **Playground/sandbox** — Interactive demo site with preset code samples

### v1.9.4 — Advanced Browser Features

**Theme:** Power-user features and enterprise readiness.

- [ ] **Local file drop** — Drag-and-drop local repo analysis (with caveats about .git availability)
- [ ] **Diff mode** — Compare two GitHub refs in-browser
- [ ] **Context packing** — `tokmd context` equivalent for LLM prompt assembly
- [ ] **Offline support** — Service worker for offline capability
- [ ] **Analytics/telemetry** — Optional usage telemetry (opt-in) to guide future development

### Supported Browser-Safe Surface (v1.9.x)

| Mode | Status | Notes |
|------|--------|-------|
| `lang` | ✅ | Full parity with native |
| `module` | ✅ | Full parity with native |
| `export` | ✅ | Full parity with native |
| `analyze receipt` | ✅ | Core derived metrics only |
| `analyze estimate` | ✅ | COCOMO effort estimation |
| `analyze health` | 🚧 v1.9.2 | If content scanning lands |
| `analyze supply` | 🚧 v1.9.2 | If asset detection lands |
| `badge` | 🚧 v1.9.2 | Client-side SVG generation |
| `diff` | 🚧 v1.9.4 | Cross-ref comparison |
| `context` | 🚧 v1.9.4 | LLM context packing |

### Capability Constraints (Documented)

**Unavailable in browser (by design):**
- Git history analysis (hotspot, churn, coupling) — requires `git log` subprocess
- Filesystem walking — requires host filesystem access
- Content scanning that touches disk — requires file reads outside memory
- Mutation testing, heavy analysis — performance constraints

**Rootless constraints:**
- All inputs must be provided in-memory or via HTTP fetch
- No shelling out to external tools
- Deterministic timestamps (0) instead of system time on bare WASM

### Non-Goals for v1.9.x

- No browser-side git-history metrics (keep as explicit capability miss)
- No zipball ingestion as primary path (tree+contents is supported)
- No mutation testing or heavy tooling in-browser
- No full AST analysis (waits for v3.x Tree-sitter integration)

[190 more lines in file. Use offset=510 to continue.]

## v1.10.x — Code Quality Initiative: Unwrap Burn-down

### v1.10.0 — Unwrap Burn-down Sprint 1

_Goal: Eliminate all `.unwrap()` calls from core libraries (Tier 0-1), achieving panic-free foundation._

**Current state:** 19,462 unwrap() calls (concentrated in tests, CLI, and some library code)  
**Target state:** Zero unwrap() calls in Tier 0-1 — every fallible operation returns `Result`

**Scope (Tier 0-1 crates):**
- [ ] `tokmd-types` — Contract types, zero panics
- [ ] `tokmd-analysis-types` — Analysis contract types
- [ ] `tokmd-settings` — Settings types
- [ ] `tokmd-scan` — Core scanning logic
- [ ] `tokmd-model` — Aggregation logic
- [ ] `tokmd-math` — Deterministic math helpers
- [ ] `tokmd-path` — Path normalization
- [ ] `tokmd-module-key` — Module key derivation
- [ ] `tokmd-exclude` — Exclude pattern handling
- [ ] `tokmd-redact` — BLAKE3 redaction
- [ ] `tokmd-scan-args` — Scan arguments
- [ ] `tokmd-tokeignore` — Ignore template generation
- [ ] `tokmd-sensor` — Sensor trait and builder

**Mechanics:**
- Enforce via CI lint: `#![deny(clippy::unwrap_used)]` per crate (opt-in, tier-by-tier)
- Use `anyhow::Context` for error enrichment: `.context("failed to parse config")?`
- Prefer `expect()` over `unwrap()` during transition, with descriptive "why this shouldn't fail" messages
- Track progress with `cargo grep unwrap | wc -l` in CI metrics

**Rationale:**
- Deterministic error handling aligns with tokmd's "receipt-grade" philosophy
- Panic-free libraries enable panic-free downstream consumers (WASM, FFI, Python/Node)
- Foundation must be solid before higher-tier cleanup

### v1.10.1 — Unwrap Burn-down Sprint 2

_Goal: Clean up Tier 2-3 crates (adapters and orchestration)._

**Scope:**
- [ ] `tokmd-analysis-*` crates — All 20+ analysis enrichers
- [ ] `tokmd-format` — Output formatting
- [ ] `tokmd-walk` — Filesystem traversal
- [ ] `tokmd-content` — Content scanning
- [ ] `tokmd-git` — Git history analysis
- [ ] `tokmd-cockpit` — PR review metrics
- [ ] `tokmd-gate` — Policy evaluation

### v1.10.2 — Unwrap Burn-down Sprint 3

_Goal: Clean up Tier 4-5 (facades and products) and test code._

**Scope:**
- [ ] `tokmd-core` — Library facade
- [ ] `tokmd-config` — Configuration loading
- [ ] `tokmd` — CLI binary (can use `.expect()` for truly unrecoverable)
- [ ] `tokmd-python` — Python bindings
- [ ] `tokmd-node` — Node.js bindings
- [ ] `tokmd-wasm` — WASM bindings
- [ ] **Test code** — Replace all `unwrap()` with `?` propagation; tests return `Result<(), Box<dyn std::error::Error>>`
- [ ] **Benchmarks and fuzz targets** — Audit for panic paths

---

## v1.11.x — Code Quality Sprints (Ongoing)

_Following the unwrap burn-down, additional code quality initiatives will be staged in 1.11.x and beyond._

Each sprint targets a specific quality dimension with measurable outcomes.

### v1.11.0 — Dead Code Elimination Sprint

**Goal:** Remove unused code, dependencies, and exports across the workspace.

**Scope:**
- [ ] **cargo-udeps sweep** — Identify unused crate dependencies in all 61 crates
- [ ] **pub-visibility audit** — Mark truly internal items as `pub(crate)` instead of `pub`
- [ ] **unused-feature flags** — Remove features with zero consumers
- [ ] **dead code in tests** — Remove test helpers copied between crates, consolidate to `tokmd-test-helpers`
- [ ] **script cleanup** — Audit and archive one-off Python/Shell scripts in repo root

**Mechanics:**
- Run `cargo udeps` in CI (nightly) with fail-on-warning for new deps
- Use `cargo-public-api` to track API surface changes
- Before/after binary size comparison for CLI and WASM

**Rationale:**
- Smaller binaries (WASM bundle size matters for browser)
- Faster compile times (less code to check)
- Clearer API boundaries (only exported what is used)

### v1.11.1 — Dependency Audit and Pruning

**Goal:** Audit dependency tree for risk, duplication, and freshness.

**Scope:**
- [ ] **cargo-deny audit** — License compliance, security advisories, banned crates
- [ ] **duplicate dependency cleanup** — Use `cargo tree -d` to find and consolidate duplicates
- [ ] **version bump sweep** — Update deps that are >1 year old
- [ ] **feature unification** — Ensure same crate version used across workspace (via workspace deps)
- [ ] **vendor policy review** — Document why each vendored crate exists (e.g., `home` fork)

**Mechanics:**
- `cargo deny check` in CI (already present, ensure strict mode)
- Weekly `cargo update` PRs with automated testing
- Dependency decision log in `docs/dependencies.md`

**Rationale:**
- Security posture (fast response to RUSTSEC advisories)
- Build reproducibility (lockfile hygiene)
- Supply chain risk (understand every dependency)

### v1.11.2 — Documentation Completeness Pass

**Goal:** Every public API has rustdoc, every module has module-level docs.

**Scope:**
- [ ] **rustdoc coverage** — `#[deny(missing_docs)]` on all Tier 0-3 crates
- [ ] **module-level documentation** — Every `lib.rs` explains the crate's purpose and boundaries
- [ ] **README freshness** — All 61 crates have current README with usage example
- [ ] **doc-link checking** — `cargo doc` with no broken intra-doc links
- [ ] **tutorial refresh** — `docs/tutorial.md` tested against latest CLI
- [ ] **recipes expansion** — Add 5+ new practical recipes to `docs/recipes.md`

**Mechanics:**
- CI job: `cargo doc --no-deps` with warnings-as-errors
- doctest execution: `cargo test --doc` passes for all crates
- "Documentation debt" metric: count of `// TODO: document this`

**Rationale:**
- Onboarding friction (new contributors need good docs)
- API discoverability (rustdoc is the contract)
- Long-term maintenance (docs explain *why*, not just *what*)

### v1.11.3 — Test Coverage Gap Closure

**Goal:** Identify and fill critical test gaps.

**Scope:**
- [ ] **coverage analysis** — Run `cargo llvm-cov` across workspace, identify <80% files
- [ ] **error path testing** — Every `?` and `match` branch has test case
- [ ] **property test expansion** — Add proptest to 10 more crates (currently 14)
- [ ] **fuzz target addition** — 3 new fuzz targets (currently 3)
- [ ] **mutation testing gate** — Ensure cargo-mutants survival rate >95%

**Mechanics:**
- Coverage reporting in CI (codecov or similar)
- Mutation testing as merge gate (not just informational)
- "Coverage delta" check on PRs (cannot drop coverage)

**Rationale:**
- Confidence in refactors (tests catch behavior changes)
- Bug prevention (edge cases documented as test cases)
- Release readiness (high coverage = lower risk)

### v1.11.4 — Performance Hot-Path Optimization

**Goal:** Profile and optimize critical paths identified in v1.9.x and v1.10.x work.

**Scope:**
- [ ] **benchmark baselines** — Establish `cargo bench` suite for key operations
- [ ] **memory profiling** — Heap analysis for large repo scans (>10k files)
- [ ] **allocation reduction** — Target allocations in `tokmd-format` and `tokmd-analysis`
- [ ] **parallelization review** — Where can rayon or async improve throughput?
- [ ] **WASM bundle optimization** — Size and runtime performance in browser

**Mechanics:**
- `criterion` benchmarks in CI with performance regression detection
- Flamegraph generation for analysis workflows
- Size budgets: WASM bundle must stay under 5MB (gzipped)

**Rationale:**
- User experience (fast feedback in CLI and browser)
- Cost efficiency (less CPU time in CI)
- Scalability (can handle larger repos without degradation)

---

## Future Horizons

### v2.0 — Platform Evolution

#### A. Language Bindings (FFI) ✅ Complete

_Goal: Native integration in CI pipelines and tooling ecosystems._

**Python (PyPI: `tokmd`)** ✅

- Native bindings via PyO3 + maturin
- Crate: `tokmd-python/`
- API: `tokmd.lang()`, `tokmd.module()`, `tokmd.export()`, `tokmd.analyze()`, `tokmd.diff()`
- Returns native Python dicts
- Wheels for Linux, macOS, Windows (x64 + arm64)
- JSON API: `tokmd.run_json(mode, args_json)` for low-level access

**Node.js (npm: `@tokmd/core`)** ✅

- Native bindings via napi-rs
- Crate: `tokmd-node/`
- API: `lang()`, `module()`, `export()`, `analyze()`, `diff()` returning JS objects
- Prebuilds for major platforms
- All functions return Promises (async/non-blocking)

**Shared Infrastructure** ✅

- `tokmd-core` crate expanded with binding-friendly API
- Pure settings types (no Clap dependencies)
- JSON-in/JSON-out FFI boundary via `run_json()`
- Structured error types for FFI

#### B. MCP Server Mode

_Goal: Native integration with Claude and other MCP-compatible clients._

- `tokmd serve` — Start MCP server for tool-based interaction
- Resources: Expose receipts as MCP resources
- Tools: `scan`, `analyze`, `diff`, `suggest` as MCP tools
- Streaming: Incremental analysis results

#### C. Streaming Analysis

_Goal: Handle massive repositories without memory pressure._

- JSONL streaming for all outputs
- Incremental file processing
- Memory-bounded analysis limits
- Progress reporting via stderr

#### D. Plugin System

_Goal: Extensible enrichers without core changes._

- WASM plugin interface for custom analyzers
- Plugin discovery from `~/.tokmd/plugins/`
- Schema for plugin metadata and capabilities

#### E. Analysis Engine Performance

_Goal: Reduce analysis latency and I/O overhead for large repositories._

- **Enricher parallelization** — Execute independent enrichers concurrently (complexity, imports, content scanning can run in parallel)
- **Inter-enricher file content caching** — Cache file contents in memory during analysis pass to eliminate redundant reads across enrichers
- Streaming JSONL output for memory-bounded processing
- Progress reporting via stderr for long-running analysis

### v2.1 — Intelligence Features

#### E. Smart Suggestions

_Goal: Actionable recommendations, not just metrics._

- `tokmd suggest --budget 128k` — Files to include for context
- `tokmd suggest --review` — Files likely to need attention
- `tokmd suggest --test` — Untested code paths

#### F. Diff Intelligence

_Goal: Semantic diff beyond structural changes._

- Complexity delta detection
- Breaking change indicators
- Migration path suggestions

#### G. Watch Mode

_Goal: Continuous analysis during development._

- `tokmd watch` — Re-analyze on file changes
- Integration with LSP for editor feedback
- Real-time metric updates

### v2.2 — Ecosystem Integration

#### H. CI/CD Native

_Goal: First-class CI pipeline support._

- GitHub Action with PR comments
- GitLab CI template
- Trend tracking across commits
- Threshold-based failures (e.g., fail if complexity increases)

#### I. Editor Extensions

_Goal: Analysis at your fingertips._

- VS Code extension with inline metrics
- Neovim plugin for buffer analysis
- JetBrains plugin

#### J. Cloud Dashboard

_Goal: Historical tracking and team insights._

- Receipt aggregation service
- Trend visualization
- Team comparison views
- Alert on anomalies

### v3.0 — Tree-sitter Integration (Long-term)

_Goal: Accurate parsing for precise metrics. This is a significant undertaking requiring substantial R&D investment and is intentionally deferred well beyond the v2.x roadmap._

#### K. AST Foundation

- `tokmd-treesitter` crate with multi-language AST parsing
- Language support: Rust, TypeScript, Python, Go, C, C++, Java, C#
- Basic AST traversal and node extraction
- Accurate function boundary detection

### v3.1 — AST-Aware Metrics

_Goal: Leverage AST for precise metric calculation._

- **Cyclomatic complexity** — Control-flow analysis instead of keyword counting
- **Cognitive complexity** — Nested scope analysis using actual scopes
- **Import resolution** — Precise parsing vs. regex-based
- **Function-level detail** — Accurate boundaries for all supported languages

### v3.2 — Advanced AST Features

_Goal: Rich code intelligence from AST._

- **Call graph extraction** — Cross-function dependency analysis
- **Data flow analysis** — Basic taint tracking for security analysis
- **Refactoring detection** — Identify extracted methods, renamed variables across commits

---

## Non-Goals

These are explicitly out of scope for tokmd:

- **Code formatting/linting** — Use dedicated tools (rustfmt, eslint)
- **Dependency vulnerability scanning** — tokmd delegates to external tools (cargo-audit, npm audit) when available; it does not maintain its own advisory database
- **Test execution** — Use cargo test, pytest, jest
- **Build orchestration** — Use cargo, make, just
- **Full AST analysis** — tokmd uses heuristics, not parsers (tree-sitter is a long-term v3.x aspiration)

---

## Contributing

Contributions welcome! Priority areas:

1. **Enricher implementations** — See `crates/tokmd-analysis/src/` for patterns
2. **Output format templates** — Markdown templates in `tokmd-analysis-format`
3. **Language support** — Extend import graph parsing
4. **Documentation** — Recipe examples and use cases

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
