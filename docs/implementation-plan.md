# tokmd Implementation Plan

This document outlines planned improvements aligned with the roadmap.

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

As part of this phase, three additional Tier 0 crates were created:

- **`tokmd-envelope`**: Cross-fleet `SensorReport` contract (`sensor.report.v1` schema)
- **`tokmd-substrate`**: Shared `RepoSubstrate` context for multi-sensor pipelines
- **`tokmd-sensor`** (Tier 1): `EffortlessSensor` trait + `build_substrate()` builder

### Work Items

- [x] Create tokmd-settings crate
- [x] Define pure Settings types (no clap derive)
- [x] Update tokmd-scan to accept `&ScanOptions`
- [x] Update tokmd-core to use tokmd-settings
- [x] Update tokmd-config to re-export or wrap
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
- [ ] Implement `cockpit_workflow`
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

### Near-Duplicate Detection (v1.7.0)

1. **Enricher**: `tokmd-analysis-near-dup` crate for content-similarity detection
2. **CLI flags**: `--near-dup`, `--near-dup-threshold`, `--near-dup-scope`, `--near-dup-max-files`
3. **Types**: `NearDuplicateReport`, `NearDupCluster`, `NearDupPair`

### Commit Intent & Coupling (v1.7.0)

1. **Commit intent**: Automatic classification of commit purpose (`CommitIntentKind`)
2. **Coupling metrics**: Jaccard similarity and Lift in coupling reports
3. **Token estimation**: Renamed `tokens_low`/`tokens_high` → `tokens_min`/`tokens_max` with backward-compatible serde aliases

### Microcrate Extraction (v1.7.1)

1. **Tier 1 microcrates**: `tokmd-context-policy`, `tokmd-scan-args`, `tokmd-math`, `tokmd-exclude`, `tokmd-path`, `tokmd-module-key`
2. **Tier 2 microcrates**: `tokmd-context-git`, `tokmd-export-tree`
3. **Tier 3 microcrates**: `tokmd-analysis-explain`, `tokmd-analysis-imports`, `tokmd-analysis-maintainability`, `tokmd-analysis-html`
4. **Tier 4 microcrates**: `tokmd-tool-schema`, `tokmd-ffi-envelope`
5. **Architectural**: Moved `AnalysisFormat` to `tokmd-types` (Tier 0)

### Schema Changes

- **Analysis schema version**: 6 → 7 (v1.7.0) → 8 (v1.7.1)

### Work Items

- [x] Implement near-duplicate detection enricher with configurable threshold
- [x] Add commit intent classification to git reports
- [x] Add Jaccard similarity and Lift to coupling metrics
- [x] Rename token estimation fields with backward-compatible aliases
- [x] Extract 15 focused microcrates from monolithic modules
- [x] Move `AnalysisFormat` to `tokmd-types` (Tier 0)
- [x] Update CI/tooling for 40+ crate workspace
- [x] Fix clippy/lint across all new crates

### Tests

- [x] Near-dup detection integration tests
- [x] Serde alias backward compatibility tests for token field renames
- [x] E2E `ContextReceipt` backward compatibility test
- [x] Boundary checks for microcrate extraction

---

## Phase 5: MCP Server Mode (v2.0)

**Goal**: Native integration with Claude and MCP clients.

### Server Implementation

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

## Phase 6: Tree-sitter Integration (v3.0 — Long-term)

**Goal**: Accurate parsing for precise metrics. This is a significant R&D effort requiring multi-language grammar integration, cross-platform build toolchains, and extensive correctness validation. Intentionally deferred well beyond v2.x.

### Language Support

1. **Parser crate**: `tokmd-treesitter` (new)
2. **Languages**: Rust, TypeScript, Python, Go, Java
3. **Feature-gated**: Optional dependency

### Capabilities

- Accurate function boundary detection
- Nested scope analysis for cognitive complexity
- Call graph extraction for coupling analysis

### Prerequisites

- Stable tokmd-core API (Phase 3)
- Halstead and function-level metrics (Phase 4) as integration surface
- MCP server mode (Phase 5) to validate use cases that require AST precision

### Work Items

- [ ] Evaluate tree-sitter grammar availability and build complexity per language
- [ ] Create tokmd-treesitter crate with feature-gated dependency
- [ ] Implement language-specific parsers
- [ ] Integrate with tokmd-content for precise boundary detection
- [ ] Update complexity calculation to use AST when available
- [ ] Performance benchmarks (must not regress heuristic-based path)

### Tests

- Unit tests: Parser correctness per language
- Golden tests: Parse tree snapshots
- Fuzz tests: Parser robustness
- Benchmarks: Performance regression detection

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
