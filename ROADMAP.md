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
| **v1.9.0** | ✅ Complete | Browser/WASM productization: parity-covered wasm entrypoints, browser runner MVP, and public repo ingestion via tree+contents |
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

## v1.9.0 — Browser/WASM Productization

**Goal:** Finish the browser/WASM product surface around the already-landed in-memory execution path and make the supported browser workflow explicit, repeatable, and capability-honest.

### What shipped in v1.9.0

- [x] `tokmd-io-port`, in-memory scan/model/core workflow seams, and lower-tier clap-free boundaries now keep browser/WASM execution honest.
- [x] `tokmd-wasm` exposes browser-friendly entrypoints for `lang`, `module`, `export`, and browser-safe `analyze`.
- [x] Native-vs-wasm parity coverage exists for `lang`, `module`, `export`, `analyze receipt`, and `analyze estimate`.
- [x] `web/runner` boots the real `tokmd-wasm` bundle in a dedicated worker, reports capabilities, renders the latest successful result, and supports JSON download.
- [x] Public GitHub repo acquisition uses the browser-safe GitHub tree and contents APIs to materialize deterministic ordered inputs locally in the page.
- [x] `tokmd-wasm` browser bundle is deployed as a versioned release artifact consumed directly from `web/runner/vendor/tokmd-wasm`.
- [x] Browser runner guardrails and UX hardening landed, including caching, progress, authenticated fetch options, and rate-limit handling.

### Supported browser-safe surface today

- Browser/WASM modes: `lang`, `module`, `export`
- Browser/WASM analyze presets: `receipt`, `estimate`
- Public repo acquisition strategy: GitHub tree + contents API, not zipball fetch
- Capability reporting is explicit about unavailable host-backed enrichers and reserved protocol features

### Current browser constraints

- Browser-safe analysis should expand only where the preset can stay rootless and capability-honest.

### Non-goals for v1.9.0

- No browser-side git-history churn/hotspot metrics; keep those as explicit capability misses or backend follow-ups.
- No browser zipball ingestion as the primary supported path for `v1.9.0`; tree+contents is the supported browser acquisition strategy.
- No mutation testing or other heavy tooling in-browser.

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

#### K. Tree-sitter AST Parsing

- `tokmd-treesitter` crate with multi-language AST parsing
- Language-specific complexity rules (Rust, TypeScript, Python, Go, etc.)
- Accurate function boundary detection
- Nested scope analysis for cognitive complexity
- Call graph extraction for coupling analysis

---

## Non-Goals

These are explicitly out of scope for tokmd:

- **Code formatting/linting** — Use dedicated tools (rustfmt, eslint)
- **Dependency vulnerability scanning** — tokmd delegates to external tools (cargo-audit, npm audit) when available; it does not maintain its own advisory database
- **Test execution** — Use cargo test, pytest, jest
- **Build orchestration** — Use cargo, make, just
- **Full AST analysis** — tokmd uses heuristics, not parsers (tree-sitter is a long-term v3.0 aspiration)

---

## Contributing

Contributions welcome! Priority areas:

1. **Enricher implementations** — See `crates/tokmd-analysis/src/` for patterns
2. **Output format templates** — Markdown templates in `tokmd-analysis-format`
3. **Language support** — Extend import graph parsing
4. **Documentation** — Recipe examples and use cases

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
