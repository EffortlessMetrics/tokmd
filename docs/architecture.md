# tokmd Architecture

This document describes the internal architecture of tokmd for contributors and library consumers.

See also: [tokmd responsibilities](../tokmd-role.md) - tokmd's position in the sensors -> receipts -> cockpit stack.

## Design Principles

1. **Receipts are the bus**: Schemaed outputs are the record, not logs
2. **Determinism is UX**: Stable ordering and budgets prevent "comment churn"
3. **Truth-layer discipline**: tokmd stays repo/diff truth; build-truth consumers live elsewhere
4. **One scan, many views**: Single scan produces lang/module/export/analysis views

## Crate Hierarchy

tokmd follows a tiered microcrate architecture with strict dependency rules.

```
Tier 0 (Contracts)     tokmd-types, tokmd-analysis-types, tokmd-settings,
                       tokmd-envelope, tokmd-substrate
         ↓
Tier 1 (Core)          tokmd-scan, tokmd-model, tokmd-module-key, tokmd-path, tokmd-redact,
                       tokmd-scan-args,
                       tokmd-tokeignore, tokmd-sensor
         ↓
Tier 2 (Adapters)      tokmd-format, tokmd-walk, tokmd-content, tokmd-git,
                       tokmd-context-git, tokmd-badge, tokmd-progress
         ↓
Tier 3 (Orchestration) tokmd-analysis, tokmd-analysis-format, tokmd-analysis-archetype,
                       tokmd-analysis-topics, tokmd-analysis-fingerprint, tokmd-analysis-explain,
                       tokmd-analysis-html, tokmd-analysis-imports, tokmd-analysis-maintainability,
                       tokmd-fun, tokmd-gate
         ↓
Tier 4 (Facade)        tokmd-config, tokmd-core, tokmd-ffi-envelope, tokmd-tool-schema
         ↓
Tier 5 (Products)      tokmd (CLI), tokmd-python, tokmd-node
```

### Tier 0: Contracts (Pure Data)

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `tokmd-types` | Core receipt DTOs (`LangRow`, `ModuleRow`, `FileRow`, `Totals`) | `serde` only |
| `tokmd-analysis-types` | Analysis receipt DTOs | `serde`, `tokmd-types` |
| `tokmd-settings` | Clap-free settings types (`ScanOptions`, `LangSettings`, etc.) | `serde`, `tokmd-types` |
| `tokmd-envelope` | Cross-fleet `SensorReport` contract (`Verdict`, `Finding`, `GateResults`) | `serde`, `serde_json` |
| `tokmd-substrate` | Shared repo context (`RepoSubstrate`, `SubstrateFile`, `DiffRange`) | `serde` only |

**Schema Versions** (separate per family):
- Core receipts: `SCHEMA_VERSION = 2` (lang, module, export, diff, context, run)
- Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 8`
- Cockpit receipts: `COCKPIT_SCHEMA_VERSION = 3`

### Tier 1: Core Processing

| Crate | Purpose |
|-------|---------|
| `tokmd-scan` | Wraps tokei library for code scanning |
| `tokmd-model` | Aggregation logic: tokei results → tokmd receipts |
| `tokmd-module-key` | Deterministic module-key derivation from normalized paths |
| `tokmd-path` | Cross-platform path normalization helpers (`\\` → `/`, relative path cleanup) |
| `tokmd-redact` | BLAKE3-based path hashing and redaction |
| `tokmd-scan-args` | Deterministic `ScanArgs` metadata construction + redaction wiring |
| `tokmd-tokeignore` | `.tokeignore` template generation |
| `tokmd-sensor` | `EffortlessSensor` trait + `build_substrate()` builder |

### Tier 2: Adapters

| Crate | Purpose | Feature Flag |
|-------|---------|--------------|
| `tokmd-format` | Output rendering (Markdown, TSV, JSON, CSV, JSONL, CycloneDX) | — |
| `tokmd-walk` | Filesystem traversal with gitignore support | `walk` |
| `tokmd-content` | File content scanning (entropy, tags, hashing) | `content` |
| `tokmd-git` | Git history analysis via shell `git log` | `git` |
| `tokmd-context-git` | Git-derived hotspot/churn scoring for context ranking | `git` |
| `tokmd-badge` | SVG badge rendering helpers | — |
| `tokmd-progress` | Progress spinner and progress-bar abstractions | `ui` |

### Tier 3: Orchestration

| Crate | Purpose |
|-------|---------|
| `tokmd-analysis` | Analysis orchestration with preset system |
| `tokmd-analysis-api-surface` | API surface analysis |
| `tokmd-analysis-archetype` | Archetype inference adapter |
| `tokmd-analysis-assets` | Asset and dependency reports |
| `tokmd-analysis-complexity` | Cyclomatic/cognitive complexity |
| `tokmd-analysis-content` | Content scanning adapters (TODO, dup, imports) |
| `tokmd-analysis-imports` | Language-aware import parsing + normalization |
| `tokmd-analysis-derived` | Core derived metrics (density, COCOMO) |
| `tokmd-analysis-entropy` | High-entropy file detection |
| `tokmd-analysis-explain` | Metric/finding explanation catalog and alias lookup |
| `tokmd-analysis-html` | Single-responsibility HTML renderer for analysis receipts |
| `tokmd-analysis-fingerprint` | Corporate fingerprint adapter |
| `tokmd-analysis-format` | Analysis output rendering (Markdown, JSON, SVG, HTML, etc.) |
| `tokmd-analysis-git` | Git history analysis adapters |
| `tokmd-analysis-grid` | Preset/feature matrix metadata |
| `tokmd-analysis-halstead` | Halstead metrics |
| `tokmd-analysis-maintainability` | Maintainability index scoring + Halstead merge |
| `tokmd-analysis-license` | License radar scanning |
| `tokmd-analysis-near-dup` | Near-duplicate detection |
| `tokmd-analysis-topics` | Topic-cloud extraction adapter |
| `tokmd-analysis-util` | Shared analysis utilities |
| `tokmd-fun` | Novelty outputs (eco-label, MIDI, OBJ) |
| `tokmd-gate` | Policy evaluation with JSON pointer rules |

### Tier 4: Facade

| Crate | Purpose |
|-------|---------|
| `tokmd-config` | CLI parsing (clap) + configuration loading |
| `tokmd-ffi-envelope` | Shared FFI envelope parser/extractor for Python/Node bindings |
| `tokmd-tool-schema` | AI tool-schema generation from clap command trees |
| `tokmd-core` | Library facade with FFI layer |

### Tier 5: Products

| Crate | Purpose |
|-------|---------|
| `tokmd` | CLI binary |
| `tokmd-python` | PyO3 bindings for Python |
| `tokmd-node` | napi-rs bindings for Node.js |

## Dependency Rules

1. **Contracts MUST NOT depend on clap** — Keep `tokmd-types` and `tokmd-analysis-types` pure
2. **Lower tiers MUST NOT depend on higher tiers** — No upward dependencies
3. **Feature flags control optional adapters** — `git`, `walk`, `content` features
4. **IO adapters depend on domain/contracts, never reverse**

## Data Flow

### Flow A: Repository Inventory (lang/module/export)

```
Filesystem → tokmd-walk → tokmd-scan (tokei) → tokmd-model → tokmd-format → Output
                                ↓
                          BTreeMap (deterministic)
                                ↓
                    Receipt DTOs (tokmd-types)
```

### Flow B: Analysis (analyze/cockpit)

```
Receipt → tokmd-analysis → Enrichers → tokmd-analysis-format → Output
              ↓
    ┌───────────────┴────────────────────┐
    ↓                                  ↓
Optional:                          Core:
- tokmd-git            - identity fingerprint, git risk metrics
- tokmd-content        - topics enrichment adapter
- tokmd-walk           - scan-adjacent enrichers (assets/dependency reports, entropy/license)
- tokmd-analysis-fingerprint
- tokmd-analysis-archetype
```

### Flow C: Sensor Integration (tokmd-sensor)

```
ScanOptions → build_substrate() → RepoSubstrate (shared context)
                                       ↓
                            ┌──────────┴──────────┐
                            ↓                     ↓
                     Sensor A.run()         Sensor B.run()
                            ↓                     ↓
                      SensorReport          SensorReport
                            ↓                     ↓
                            └──────────┬──────────┘
                                       ↓
                              Director aggregates
```

"Substrate once, sensors many" — the scan runs once, then each `EffortlessSensor`
receives the same `RepoSubstrate` and produces a standardized `SensorReport` envelope.

### Flow D: Library API (tokmd-core)

```
Settings → Workflow Functions → Receipt → JSON
    ↓
run_json(mode, args_json) ─→ {"ok": true, "data": {...}}
    ↓
Python/Node bindings wrap FFI layer
```

## Determinism Guarantees

tokmd guarantees byte-stable output for identical inputs:

1. **Ordered structures**: `BTreeMap` and `BTreeSet` at all boundaries
2. **Stable sorting**: Descending by code lines, then ascending by name
3. **Path normalization**: Forward slashes (`/`) regardless of OS
4. **Timestamp normalization**: `generated_at_ms` normalized in tests
5. **Redaction determinism**: Same input → same BLAKE3 hash

## Error Handling

| Scenario | Exit Code | Receipt |
|----------|-----------|---------|
| Success | 0 | Full receipt |
| Tool/runtime error | 1 | Partial receipt when possible |
| Policy failure (gate) | 2 | Receipt with failure reason |
| Missing optional input | — | `skip` verdict with `missing_input` reason |

## Feature Flags

```toml
[features]
git = ["tokmd-git", "tokmd-context-git/git"]      # Git history analysis + context git scores
content = ["tokmd-content"]  # File content scanning
walk = ["tokmd-walk"]    # Filesystem traversal
topics = ["tokmd-analysis-topics"] # Topic extraction
archetype = ["tokmd-analysis-archetype"] # Repository archetype detection
fun = ["tokmd-analysis-fun"] # Eco-label and novelty helpers
ui = ["dialoguer", "indicatif"]  # Interactive CLI
```

## Publishing Matrix

### Published to crates.io
- `tokmd` (binary)
- `tokmd-types`, `tokmd-analysis-types` (contracts)
- Core crates as stable

### Workspace-only (publish = false)
- `tokmd-python` (published to PyPI via maturin)
- `tokmd-node` (published to npm via napi-rs)
- Clap-facing argument models
- UI affordances

## Sensor Integration Architecture

The sensor subsystem enables multi-sensor pipelines where tokmd acts as one sensor
among many (cargo-deny, cargo-audit, etc.) in a CI/CD fleet.

### Key Crates

| Crate | Role |
|-------|------|
| `tokmd-substrate` | Shared scan context (files, languages, diff range) — built once |
| `tokmd-envelope` | Standardized report contract (`sensor.report.v1`) |
| `tokmd-settings` | Clap-free settings for library/FFI consumers |
| `tokmd-sensor` | `EffortlessSensor` trait + substrate builder |

### Design Principles

1. **Substrate once, sensors many**: A single I/O pass builds `RepoSubstrate`, eliminating redundant scans
2. **Standardized envelope**: All sensors emit `SensorReport` with findings, verdicts, and gates
3. **Clap-free settings**: Lower-tier crates use `ScanOptions` from `tokmd-settings`, not `GlobalArgs`
4. **Finding identity**: `(check_id, code)` tuples enable category-based routing for buildfix automation

## WASM & Browser Runner (v1.8.0 — v1.9.0)

### v1.8.0 — WASM-Ready Core

Goal: Make the tokmd engine compile for `wasm32-unknown-unknown` and run against an in-memory repo substrate so the same deterministic receipts can be produced from an in-memory file set.

Work items:
- Host abstraction (IO ports): enumerate files, read bytes, clock, and optional logging/progress; native uses FS, WASM uses a host-provided substrate.
- In-memory scan pipeline: accept `Vec<(path, bytes)>` instead of `PathBuf` to enable scans from memory.
- CLI/Clap separation: ensure library crates do not depend on `clap`; keep argument parsing in the CLI crate.
- WASM feature profile: add a `wasm`/`web` feature that disables OS-bound pieces (`git`, `dirs`, `std::process`).
- WASM CI builds and conformance tests: add `cargo build --target wasm32-unknown-unknown` to CI and golden tests to validate parity.

Notes: Git-history enrichers (hotspots/churn) are not available in browser WASM mode and must be reported as unavailable in capability reporting.

### v1.9.0 — WASM Distribution + Browser Runner

Goal: Ship a `tokmd-wasm` bundle and a minimal static web runner that fetches a GitHub zipball, unpacks in the browser, runs tokmd in a Worker, and renders/downloads deterministic receipts locally without server-side computation.

Work items:
- `tokmd-wasm` crate: expose JS-friendly APIs (via `wasm-bindgen`): `run_lang`, `run_module`, `run_export`, `run_analyze` accepting in-memory inputs.
- Browser runner: minimal static app (repo URL + ref + Run) that runs scans in a Web Worker, streams progress, and supports cancel.
- Zipball ingestion: fetch GitHub zipball (`/zipball/{ref}`), unzip in-browser, filter files (skip vendor/binaries by default), and feed `(path, bytes)` to wasm.
- Caching & guardrails: IndexedDB cache keyed by `(repo,ref,options)`, ETag support, and hard limits (max archive size, file count, bytes read).
- Capability reporting: outputs include a capabilities section indicating which enrichers ran and which were unavailable.
- Packaging: publish the WASM bundle as a pinned artifact (GitHub Release / npm) for the web app to consume.

Non-goals for v1.9.0: no in-browser git churn/hotspot metrics or heavy tooling; provide a backend escape hatch for very large repos or git-based analysis.
