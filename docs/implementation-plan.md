# tokmd Implementation Plan

This document records completed implementation phases through `1.11.0` and the next active buildout aligned with the roadmap.

## Phase 1: Baseline & Ratchet System (v1.5.0) ✅ Complete

**Goal**: Enable quality improvement tracking over time.

### Baseline Storage

1. **Storage format**: `.tokmd/baseline.json`
2. **Types**: `ComplexityBaseline`, `BaselineMetrics`, `FileBaselineEntry`
3. **Command**: `tokmd baseline` to generate from current state

### Ratchet Rules

1. **Configuration**: `[[gate.ratchet]]` in `tokmd.toml`
2. **Evaluation**: `evaluate_ratchet_policy()` in tokmd-gate
3. **Parameters**: `max_increase_pct` and `max_value` for gradual improvement

### Work Items

- [x] Design baseline schema (additive to existing receipts)
- [x] Implement `tokmd baseline` command
- [x] Add `--baseline` flag to `tokmd gate`
- [x] Add `--ratchet-config` flag to `tokmd gate`
- [x] Add ratchet rule types to tokmd-gate
- [x] Implement `evaluate_ratchet_policy()` with `max_increase_pct` and `max_value`
- [x] Integration tests for ratchet evaluation
- [x] Baseline JSON schema (`docs/baseline.schema.json`)
- [x] Ecosystem envelope types for multi-sensor integration

### Tests

- [x] Golden fixtures: Baseline generation and comparison
- [x] Unit tests: Ratchet evaluation edge cases (boundary conditions, missing values)
- [x] Integration tests: CLI baseline + ratchet workflow
- [x] Combined policy + ratchet gate evaluation

---

## Phase 2: Configuration Decoupling ✅ Complete

**Goal**: Clean separation of clap from library API.

### tokmd-settings Crate ✅

1. **Created** `tokmd-settings` crate with pure configuration types (no clap)
2. **Settings types**: `ScanOptions`, `ScanSettings`, `LangSettings`, `ModuleSettings`, `ExportSettings`, `AnalyzeSettings`, `DiffSettings`
3. **Dependency chain**: `tokmd-core` depends on `tokmd-settings` (not `tokmd-config`); `tokmd-scan` accepts `&ScanOptions`

### Sensor Integration Crates ✅

As part of this phase, the sensor/envelope surfaces were created:

- **`tokmd-envelope`**: Cross-fleet `SensorReport` contract (`sensor.report.v1` schema)
- **`tokmd-sensor::substrate`**: Shared `RepoSubstrate` context for multi-sensor pipelines
- **`tokmd-sensor`** (Tier 1): `EffortlessSensor` trait + `build_substrate()` builder

### Work Items

- [x] Create tokmd-settings crate
- [x] Define pure Settings types (no clap derive)
- [x] Update tokmd-scan to accept `&ScanOptions`
- [x] Update tokmd-core to use tokmd-settings
- [x] Retire tokmd-config after moving active settings and CLI ownership
- [x] Implement TOML parsing in tokmd-settings (moved from tokmd-config)
- [x] Update bindings to use new settings directly

---

## Phase 3: tokmd-core Stabilization

**Goal**: Make tokmd-core the stable embedding surface.

### Port Formalization

1. **Define port traits** (optional, for extensibility):
   - `FileSystemPort`: List and read files
   - `GitPort`: History collection
   - `ClockPort`: Timestamps (for testing)
   - `OutputPort`: Writer abstraction

2. **Default adapters**:
   - std FS adapter
   - Shell git adapter (existing)
   - System clock adapter

### Workflow APIs

Stable, pure workflow functions:
```rust
pub fn lang_workflow(settings: &LangSettings) -> Result<LangReceipt>;
pub fn module_workflow(settings: &ModuleSettings) -> Result<ModuleReceipt>;
pub fn export_workflow(settings: &ExportSettings) -> Result<ExportReceipt>;
pub fn analyze_workflow(settings: &AnalyzeSettings) -> Result<AnalysisReceipt>;
pub fn cockpit_workflow(settings: &CockpitSettings) -> Result<CockpitReceipt>;
```

### Work Items

- [ ] Define port traits (if adding extensibility)
- [x] Implement `analyze_workflow`
- [x] Implement `cockpit_workflow`
- [ ] Add comprehensive API documentation
- [ ] Publish tokmd-core to crates.io (when stable)

### Tests

- Integration tests: Workflow functions with fixture repos
- Mutation testing: Core workflow logic

---

## Phase 4: Advanced Complexity Features (v1.6.0) ✅ Complete

**Goal**: Deeper complexity analysis and gating.

### Halstead Metrics

1. **Feature-gated**: `#[cfg(feature = "halstead")]`
2. **Metrics**: Volume, difficulty, effort
3. **Per-function**: Alongside cyclomatic/cognitive

### Function Detail Export

1. **Flag**: `--detail-functions`
2. **Output**: Per-function complexity in export format
3. **Use case**: Fine-grained analysis and tooling integration

### Complexity Histogram

1. **Distribution**: Complexity score buckets
2. **Visualization**: ASCII histogram in markdown
3. **Trend**: Compare histograms across baselines

### Derived Metrics

1. **Maintainability Index**: Composite of cyclomatic, Halstead, and LOC (SEI formula)
2. **Technical debt ratio**: Complexity-to-size ratio as a heuristic debt signal
3. **Duplication density**: Per-module metric extending duplicate detection
4. **API surface area**: Public export ratio (language-specific heuristics)
5. **Code age distribution**: Age buckets extending git freshness

### Work Items

- [x] Implement Halstead metrics calculation
- [x] Add function detail export format
- [x] Generate complexity histogram
- [x] Implement Maintainability Index (MI) as composite enricher
- [x] Add technical debt ratio enricher
- [x] Extend duplicate detection into duplication density metric
- [x] Add code age distribution to git enrichers
- [x] Documentation and examples

### Tests

- Property tests: Halstead calculation invariants
- Property tests: MI monotonicity (worse inputs → worse score)
- Golden tests: Function detail output format
- Integration tests: Complexity gate evaluation

---

## Phase 4b: UX & Output Quality (v1.6.3) ✅ Complete

**Goal**: Improve developer experience for interactive CLI usage.

### Output Improvements

1. **Colored diff**: Terminal colors for additions/removals in `tokmd diff`
2. **Summary comparison tables**: Side-by-side metric comparisons
3. **Compact table mode**: `--compact` flag for narrow terminals
4. **Sparkline trends**: Unicode sparklines for metric trends

### Interactive Experience

1. **Progress indicators**: Spinner/progress bar for long scans via `indicatif`
2. **Structured errors**: Actionable hints on common failures
3. **`--explain` flag**: Human-readable explanation of any metric or finding
4. **Dynamic completions**: Tab completion for preset/format values

### Scope Constraints

- No changes to JSON/JSONL receipt schemas (machine surfaces are stable)
- Terminal enhancements degrade gracefully on dumb terminals
- Progress output goes to stderr only (preserving pipe-ability)
- Color respects `NO_COLOR` / `CLICOLOR` conventions

### Work Items

- [x] Add `indicatif` progress bars for scan and analysis phases
- [x] Implement colored diff output with `NO_COLOR` support
- [x] Implement summary comparison tables for diff and cockpit output
- [x] Add `--compact` mode for narrow terminal tables
- [x] Implement `--explain` flag for metric definitions
- [x] Improve error messages with actionable hints
- [x] Add sparkline unicode rendering for trend data
- [x] Add dynamic completion values for `--preset` and `--format` flags

### Tests

- Integration tests: Output modes (color, compact, explain)
- Golden tests: Compact table format snapshots
- Unit tests: Sparkline rendering edge cases

---

## Phase 4c: Near-Duplicate Detection & Microcrate Extraction (v1.7.0-v1.7.1) ✅ Complete

**Goal**: Near-duplicate detection, commit intent classification, and focused microcrate extraction.

Historical note: this phase first extracted several implementation boundaries as
independent crates. The current architecture preserves the useful seams as
single-responsibility owner modules in `tokmd-scan`, `tokmd-model`,
`tokmd-format`, `tokmd-analysis`, `tokmd-core`, and `tokmd`.

### Near-Duplicate Detection (v1.7.0)

1. **Enricher**: near-duplicate module for content-similarity detection
2. **CLI flags**: `--near-dup`, `--near-dup-threshold`, `--near-dup-scope`, `--near-dup-max-files`
3. **Types**: `NearDuplicateReport`, `NearDupCluster`, `NearDupPair`

### Commit Intent & Coupling (v1.7.0)

1. **Commit intent**: Automatic classification of commit purpose (`CommitIntentKind`)
2. **Coupling metrics**: Jaccard similarity and Lift in coupling reports
3. **Token estimation**: Renamed `tokens_low`/`tokens_high` → `tokens_min`/`tokens_max` with backward-compatible serde aliases

### Boundary Extraction (v1.7.1)

1. **Tier 1 owner modules**: context policy, scan args, math, exclude, path, and module-key seams
2. **Tier 2 owner modules**: context git and export-tree seams
3. **Tier 3 seams**: CLI-local analysis explanations, analysis implementation modules, and analysis renderers under `tokmd-format`
4. **Tier 4 seams**: tool-schema wiring and `tokmd-envelope/src/ffi.rs`
5. **Architectural**: Moved `AnalysisFormat` to `tokmd-types` (Tier 0)

### Schema Changes

- **Analysis schema version**: 6 → 7 (v1.7.0) → 8 (v1.7.1)

### Work Items

- [x] Implement near-duplicate detection enricher with configurable threshold
- [x] Add commit intent classification to git reports
- [x] Add Jaccard similarity and Lift to coupling metrics
- [x] Rename token estimation fields with backward-compatible aliases
- [x] Extract focused boundaries from monolithic modules, later consolidated as owner modules
- [x] Move `AnalysisFormat` to `tokmd-types` (Tier 0)
- [x] Update CI/tooling for the expanded workspace
- [x] Fix clippy/lint across all new crates

### Tests

- [x] Near-dup detection integration tests
- [x] Serde alias backward compatibility tests for token field renames
- [x] E2E `ContextReceipt` backward compatibility test
- [x] Boundary checks for the extracted crate/module seams

---

## Phase 4d: Deep Test Expansion & I/O Seam (v1.7.x) ✅ Complete

**Goal**: Harden every tier with deeper coverage while landing the first host-abstraction seam for future in-memory and WASM execution.

### Coverage Expansion

1. **Workspace-wide test deepening**: property, BDD, snapshot, determinism, and cross-crate integration tests across all tiers
2. **Regression resistance**: more contract and serde compatibility tests for schema-bearing crates
3. **CLI confidence**: broader E2E coverage for `context`, `handoff`, `sensor`, `baseline`, `cockpit`, and help/docs sync

### Host-Abstraction Foundation

1. **`tokmd-io-port` crate**: `ReadFs`, `HostFs`, and `MemFs` abstractions
2. **WASM seam groundwork**: lower-tier code paths prepared for future host-provided file access
3. **Sensor determinism**: stable set ordering and envelope consistency across platforms

### Work Items

- [x] Expand tests across the full 50+ crate workspace
- [x] Add deeper docs/help/schema sync coverage
- [x] Land `tokmd-io-port` as the first host I/O seam
- [x] Tighten deterministic sensor output and contract checks
- [x] Keep CI green while the workspace size and test volume scaled up

---

## Phase 4e: Effort Estimation & Operator Hardening (v1.8.0) ✅ Complete

**Goal**: Add first-class effort estimation while making the repo-native release and operator path more boring on Windows, CI, and release day.

### Effort Estimation

1. **New module**: analysis effort estimation
2. **Preset**: `estimate`
3. **CLI surface**: `--effort-model`, `--effort-layer`, `--effort-base-ref`, `--effort-head-ref`, `--monte-carlo`, `--mc-iterations`, `--mc-seed`

### Operator Hardening

1. **Workspace-native quality commands**: smoother Windows formatting and xtask invocation paths
2. **Build footprint reduction**: leaner Windows debug info, `cargo trim-target`, and opt-in `sccache`
3. **CI boringness**: smarter caching, concurrency cancellation, and release workflow cleanup

### Schema Changes

- **Analysis schema version**: 8 → 9
- **New receipt section**: `effort`

### Work Items

- [x] Implement the effort estimation engine and receipt/report integration
- [x] Add the `estimate` preset and effort-specific CLI flags
- [x] Update analysis schema and docs for effort output
- [x] Keep receipt and estimate presets aligned with newer enrichers
- [x] Harden repo-native commands and release preflight on Windows
- [x] Reduce local build churn with better debug-info defaults and helper commands

---

## Phase 5: WASM-Ready Core + Browser Runner (v1.9.0) ✅ Complete

**Goal**: Turn the new host-abstraction seam into a real in-memory/WASM execution path and ship a browser-first runner.

### In-Memory Core

1. **Wire `tokmd-io-port` through scan/walk paths**
2. **Accept ordered `(path, bytes)` inputs for in-memory scans**
3. **Preserve deterministic ordering and capability reporting without filesystem-only assumptions**

### WASM + Browser Delivery

1. **Feature profile**: `wasm` / `web`
2. **Crate**: `tokmd-wasm`
3. **Runner**: static browser app using a Worker, repo tree + contents ingestion, and artifact download

### Work Items

- [x] Route scan and walk through host-provided I/O traits
- [x] Add wasm CI builds and parity checks against native output
- [x] Expose JS-friendly wasm bindings for `lang`, `module`, `export`, and `analyze`
- [x] Build a browser runner for current `lang`, `module`, `export`, and browser-safe `analyze` flows with artifact download
- [x] Add capability and guardrail policy for browser-safe modes, archive size, file count, and bytes read

### Tests

- Parity tests: native vs wasm receipt equivalence on fixture repos
- Integration tests: in-memory scan path using `MemFs`
- Browser smoke tests: worker execution and tree + contents ingestion

Runtime hardening beyond the v1.9.0 browser baseline is tracked below as
Phase 5c: cache behavior, progress events, retry/rate-limit UX, and optional
authenticated fetch.

---

## Phase 5b: Release Train Hardening (v1.10.0) ✅ Complete

**Goal**: Stabilize the shipped control-plane and browser/WASM surfaces without widening product scope.

### Release Fence

1. **No crate-boundary work**: crate surface and facade boundaries stay frozen for the release.
2. **No browser mode widening**: browser support remains governed by `docs/capabilities/wasm.json`.
3. **No browser runtime feature expansion**: cache/progress/retry/auth polish moves to `v1.11.0`.

### Work Items

- [x] Ship GitHub Action explicit modes for `module`, `export`, `gate`, `cockpit`, `sensor`, and `baseline`
- [x] Harden bounded path/root handling across native, Git, and in-memory surfaces
- [x] Make WASM timestamp and capability reporting truthful
- [x] Enforce publish-surface classification and verify package-list proof for the 16 published crates
- [x] Replace manual CLI reference tables with checked HELP markers
- [x] Add determinism, snapshot, and property-test proof coverage for release-critical paths
- [x] Clarify Jules provenance policy without blanket-blocking intentional `.jules/**` history

## Phase 5c: Browser Runtime Polish (v1.11.0) ✅ Complete

**Goal**: Deliver browser runtime polish for cache semantics, long-running analysis visibility, fetch resilience, and authenticated-fetch boundaries.

### Work Items

- [x] Define cache key and invalidation semantics
- [x] Emit explicit progress events (CLI grammar documented in
      `docs/progress-events.md`; browser worker progress is shipped)
- [x] Improve retry and rate-limit UX
- [x] Partition authenticated fetch/cache behavior safely

### Tests

- Unit tests: cache key and invalidation behavior
- Worker tests: progress event emission during long scans
- Runner tests: retry/rate-limit and authenticated-cache partition behavior

---

## Phase 5d: Cockpit Hardening & Architecture Consolidation (v1.11.0) ✅ Complete

**Goal**: Improve cockpit as the PR-review evidence surface before adding a
separate review orchestrator, consolidate implementation microcrates into SRP
modules, and keep proof observations advisory while artifacts mature.

### Work Items

- [x] Finish cockpit review-packet and Action-hosting gaps, including packet
      manifests, review maps, verifier receipts, hosted artifact comments, and
      optional proof/doc-artifact imports
- [x] Preserve `tokmd cockpit` as the review evidence implementation surface
      without adding a separate public `tokmd review` command
- [x] Consolidate architecture in batches, preserving `ci/proof.toml` scope
      granularity and owner-module boundaries
- [x] Ensure proof-run, scoped coverage, mutation, and Codecov observations
      remain advisory until a separate promotion decision is made
- [x] Add source-of-truth, user-path, handoff, release-readiness, and AST-shadow
      planning/evidence surfaces without changing default product behavior

---

## Phase 5e: Bun UB Evidence Readiness (v1.12.0) ✅ Complete

**Goal**: Make tokmd useful as a bounded evidence sensor for high-risk review workflows by producing scoped analysis and context artifacts for explicit refs and changed paths.

### Work Items

- [x] Fixed visible evidence defects before adding the preset: doc density math, context token reconciliation, path scope, symlink omissions, and bad ref handling.
- [x] Added `tokmd analyze --preset bun-ub` as a thin, scoped review evidence preset.
- [x] Added Bun-shaped fixture coverage for scoped native/API paths, valid refs, and unrelated dangling symlinks.
- [x] Documented `bun-ub`, context handoff, and `ub-review` artifact recipes.
- [x] Taught review/handoff surfaces to reference tokmd Bun UB artifacts and regeneration commands.

---

## Phase 6: MCP Server Mode (v2.0)

**Goal**: Native integration with Claude and MCP clients.

### Server Implementation

`tokmd tools` already ships tool schema generation for agent consumers. This phase is the future server/resource layer on top of that shipped schema surface.

1. **Command**: `tokmd serve`
2. **Protocol**: MCP (Model Context Protocol)
3. **Transport**: stdio or HTTP

### Resources

- Expose receipts as MCP resources
- Resource URIs: `tokmd://lang`, `tokmd://module`, etc.

### Tools

- `scan`: Run inventory scan
- `analyze`: Run analysis with preset
- `diff`: Compare receipts
- `suggest`: Context-aware recommendations

### Work Items

- [ ] Implement MCP server framework
- [ ] Define resource schemas
- [ ] Implement tool handlers
- [ ] Add streaming support
- [ ] Documentation and examples

### Tests

- Integration tests: MCP protocol compliance
- E2E tests: Claude integration scenarios

---

## Phase 7: Tree-sitter Integration (v3.0 — Shadow Mode Active)

**Goal**: Accurate parsing for precise metrics. This is a significant R&D effort requiring multi-language grammar integration, cross-platform build toolchains, and extensive correctness validation. Intentionally deferred well beyond v2.x for full default integration, but foundation shadow work has begun.

ADR-0008 defines the rollout boundary: AST work starts as a feature-gated,
Rust-first owner module with deterministic shadow comparison artifacts. It must
not change default receipt semantics until maintainers accept the evidence,
schema impact, and runtime capability story.

### Language Support

1. **First module**: `crates/tokmd-analysis/src/ast/`
2. **First language**: Rust
3. **Feature-gated**: explicit `ast` capability
4. **Later languages**: TypeScript, Python, Go, and Java only after Rust shadow evidence
5. **Optional crate boundary**: only if ADR-0002 justifies dependency isolation or a public contract

### Capabilities

- Accurate function boundary detection
- Nested scope analysis for cognitive complexity
- Call graph extraction for coupling analysis
- Deterministic heuristic-vs-AST shadow comparisons

### Prerequisites

- Stable tokmd-core API (Phase 3)
- Halstead and function-level metrics (Phase 4) as integration surface
- MCP server mode (Phase 6) to validate use cases that require AST precision
- Proof-policy scopes and publish-surface review for any new parser dependency

### Work Items

- [x] Evaluate Tree-sitter Rust grammar availability and dependency footprint for a first shadow slice
- [x] Add feature-gated `tokmd-analysis::ast` owner module
- [x] Parse initial Rust function landmarks
- [x] Add Rust import and simple control-flow landmarks
- [x] Add feature-gated library helpers that emit deterministic shadow
      artifacts under `target/tokmd-ast-shadow/`
- [x] Compare caller-supplied heuristic landmarks and AST evidence without
      changing default receipts
- [x] Add proof scope coverage for AST shadow parsing
- [x] Add performance benchmarks
- [ ] Decide later whether AST-derived public fields need schema changes

### Tests

- Unit tests: Rust parser fixture correctness
- Golden tests: Deterministic shadow comparison artifacts
- Fuzz tests: Parser robustness
- Benchmarks: Performance regression detection
- WASM/browser tests only before exposing AST capabilities in browser bundles

---

## Governance

### Schema Evolution

- Additive changes within vN
- Breaking changes bump schema version
- Document migration in CHANGELOG

### Compatibility Policy

- Maintain backwards compatibility for 2 minor versions
- Deprecation warnings before removal
- Clear upgrade guides

### Quality Gates

- No regressions in golden tests
- Property tests must pass
- Mutation testing threshold maintained
- Schema validation tests pass
