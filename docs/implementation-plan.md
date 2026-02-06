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

## Phase 2: Configuration Decoupling

**Goal**: Clean separation of clap from library API.

### tokmd-settings Crate

1. **Create** `tokmd-settings` crate:
   - Pure configuration types (no clap)
   - TOML parsing
   - Defaults and profile application

2. **Migrate** configuration types from `tokmd-config`:
   - Move structs to `tokmd-settings`
   - Keep clap args in `tokmd-config`

3. **Update** dependencies:
   - `tokmd-core` depends on `tokmd-settings` (not `tokmd-config`)
   - `tokmd` binary uses both

### Benefits

- Cleaner library API for embedders
- Smaller dependency tree for bindings
- Better separation of concerns

### Work Items

- [ ] Create tokmd-settings crate
- [ ] Define pure Settings types (no clap derive)
- [ ] Implement TOML parsing in tokmd-settings
- [ ] Update tokmd-core to use tokmd-settings
- [ ] Update tokmd-config to re-export or wrap
- [ ] Update bindings to use new settings

### Tests

- Unit tests: Config parsing + defaults
- Property tests: Profile mapping invariants
- Doc tests: Configuration examples parse correctly

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
- [ ] Implement `analyze_workflow` (currently stub)
- [ ] Implement `cockpit_workflow`
- [ ] Add comprehensive API documentation
- [ ] Publish tokmd-core to crates.io (when stable)

### Tests

- Integration tests: Workflow functions with fixture repos
- Mutation testing: Core workflow logic

---

## Phase 4: Advanced Complexity Features (v1.6.0)

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

### Work Items

- [ ] Implement Halstead metrics calculation
- [ ] Add function detail export format
- [ ] Generate complexity histogram
- [ ] Add complexity gates to tokmd gate
- [ ] Documentation and examples

### Tests

- Property tests: Halstead calculation invariants
- Golden tests: Function detail output format
- Integration tests: Complexity gate evaluation

---

## Phase 5: Tree-sitter Integration (v2.0)

**Goal**: Accurate parsing for precise metrics.

### Language Support

1. **Parser crate**: `tokmd-treesitter` (new)
2. **Languages**: Rust, TypeScript, Python, Go, Java
3. **Feature-gated**: Optional dependency

### Capabilities

- Accurate function boundary detection
- Nested scope analysis for cognitive complexity
- Call graph extraction for coupling analysis

### Work Items

- [ ] Create tokmd-treesitter crate
- [ ] Implement language-specific parsers
- [ ] Integrate with tokmd-content
- [ ] Update complexity calculation
- [ ] Performance benchmarks

### Tests

- Unit tests: Parser correctness per language
- Golden tests: Parse tree snapshots
- Fuzz tests: Parser robustness
- Benchmarks: Performance regression detection

---

## Phase 6: MCP Server Mode (v2.0)

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
