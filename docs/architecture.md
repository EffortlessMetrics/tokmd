# tokmd Architecture

This document describes the internal architecture of tokmd for contributors and library consumers.

## Design Principles

1. **Receipts are the bus**: Schemaed outputs are the record, not logs
2. **Determinism is UX**: Stable ordering and budgets prevent "comment churn"
3. **Truth-layer discipline**: tokmd stays repo/diff truth; build-truth consumers live elsewhere
4. **One scan, many views**: Single scan produces lang/module/export/analysis views

## Crate Hierarchy

tokmd follows a tiered microcrate architecture with strict dependency rules.

```
Tier 0 (Contracts)     tokmd-types, tokmd-analysis-types, tokmd-settings,
                       tokmd-envelope, tokmd-substrate, tokmd-io-port
         ↓
Tier 1 (Core)          tokmd-scan, tokmd-model, tokmd-module-key, tokmd-path, tokmd-exclude,
                       tokmd-context-policy, tokmd-math, tokmd-redact, tokmd-scan-args,
                       tokmd-tokeignore, tokmd-sensor
         ↓
Tier 2 (Adapters)      tokmd-format, tokmd-walk, tokmd-content, tokmd-git,
                       tokmd-context-git, tokmd-badge, tokmd-progress, tokmd-export-tree
         ↓
Tier 3 (Orchestration) tokmd-analysis, tokmd-analysis-api-surface, tokmd-analysis-archetype,
                       tokmd-analysis-assets, tokmd-analysis-complexity, tokmd-analysis-content,
                       tokmd-analysis-derived, tokmd-analysis-effort, tokmd-analysis-entropy,
                       tokmd-analysis-explain, tokmd-analysis-fingerprint, tokmd-analysis-format,
                       tokmd-analysis-fun, tokmd-analysis-git, tokmd-analysis-grid,
                       tokmd-analysis-halstead, tokmd-analysis-html, tokmd-analysis-imports,
                       tokmd-analysis-license, tokmd-analysis-maintainability,
                       tokmd-analysis-near-dup, tokmd-analysis-topics, tokmd-analysis-util,
                       tokmd-cockpit, tokmd-fun, tokmd-gate
         ↓
Tier 4 (Facade)        tokmd-config, tokmd-core, tokmd-ffi-envelope, tokmd-tool-schema
         ↓
Tier 5 (Products)      tokmd (CLI), tokmd-python, tokmd-node, tokmd-wasm
```

### Tier 0: Contracts (Pure Data)

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `tokmd-types` | Core receipt DTOs (`LangRow`, `ModuleRow`, `FileRow`, `Totals`) | `serde` only |
| `tokmd-analysis-types` | Analysis receipt DTOs | `serde`, `tokmd-types` |
| `tokmd-settings` | Clap-free settings types (`ScanOptions`, `LangSettings`, etc.) | `serde`, `tokmd-types` |
| `tokmd-envelope` | Cross-fleet `SensorReport` contract (`Verdict`, `Finding`, `GateResults`) | `serde`, `serde_json` |
| `tokmd-substrate` | Shared repo context (`RepoSubstrate`, `SubstrateFile`, `DiffRange`) | `serde` only |
| `tokmd-io-port` | Host-abstracted file access contracts (`ReadFs`, `HostFs`, `MemFs`) | `std` only |

**Schema Versions** (separate per family):
- Core receipts: `SCHEMA_VERSION = 2` (lang, module, export, diff, run)
- Context receipts: `CONTEXT_SCHEMA_VERSION = 4`
- Context bundles: `CONTEXT_BUNDLE_SCHEMA_VERSION = 2`
- Handoff manifests: `HANDOFF_SCHEMA_VERSION = 5`
- Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 9`
- Cockpit receipts: `COCKPIT_SCHEMA_VERSION = 3`

### Tier 1: Core Processing

| Crate | Purpose |
|-------|---------|
| `tokmd-scan` | Wraps tokei library for code scanning |
| `tokmd-model` | Aggregation logic: tokei results → tokmd receipts |
| `tokmd-module-key` | Deterministic module-key derivation from normalized paths |
| `tokmd-path` | Cross-platform path normalization helpers (`\\` → `/`, relative path cleanup) |
| `tokmd-exclude` | Deterministic exclude-pattern normalization + dedupe helpers |
| `tokmd-context-policy` | Context/handoff policy helpers (smart excludes, spine matching, classification, inclusion policy) |
| `tokmd-math` | Deterministic numeric/statistical helpers (`round_f64`, `safe_ratio`, percentile, gini) |
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
| `tokmd-export-tree` | Deterministic tree renderers for analysis/handoff exports | — |

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
| `tokmd-analysis-effort` | Effort-estimation engine (COCOMO, delta support, Monte Carlo scaffolding) |
| `tokmd-analysis-entropy` | High-entropy file detection |
| `tokmd-analysis-explain` | Metric/finding explanation catalog and alias lookup |
| `tokmd-analysis-html` | Single-responsibility HTML renderer for analysis receipts |
| `tokmd-analysis-fingerprint` | Corporate fingerprint adapter |
| `tokmd-analysis-format` | Analysis output rendering (Markdown, JSON, SVG, HTML, etc.) |
| `tokmd-analysis-fun` | Analysis-side novelty enrichment wiring |
| `tokmd-analysis-git` | Git history analysis adapters |
| `tokmd-analysis-grid` | Preset/feature matrix metadata |
| `tokmd-analysis-halstead` | Halstead metrics |
| `tokmd-analysis-maintainability` | Maintainability index scoring + Halstead merge |
| `tokmd-analysis-license` | License radar scanning |
| `tokmd-analysis-near-dup` | Near-duplicate detection |
| `tokmd-analysis-topics` | Topic-cloud extraction adapter |
| `tokmd-analysis-util` | Shared analysis utilities |
| `tokmd-cockpit` | PR cockpit metrics computation and rendering |
| `tokmd-fun` | Novelty outputs (eco-label, MIDI, OBJ) |
| `tokmd-gate` | Policy evaluation with JSON pointer rules |

### Tier 4: Facade

| Crate | Purpose |
|-------|---------|
| `tokmd-config` | Clap-backed CLI/config types plus configuration loading |
| `tokmd-ffi-envelope` | Shared FFI envelope parser/extractor for Python/Node bindings |
| `tokmd-tool-schema` | AI tool-schema generation from clap command trees |
| `tokmd-core` | Library facade with FFI layer |

### Tier 5: Products

| Crate | Purpose |
|-------|---------|
| `tokmd` | CLI binary |
| `tokmd-python` | PyO3 bindings for Python |
| `tokmd-node` | napi-rs bindings for Node.js |
| `tokmd-wasm` | wasm-bindgen bindings for browser/worker callers |

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
Receipt / export / paths → tokmd-analysis → Enrichers → tokmd-analysis-format → Output
                                ↓
                 ┌──────────────┴─────────────────────────────┐
                 ↓                                            ↓
        Core enrichers                                  Optional adapters
        - tokmd-analysis-derived                        - tokmd-git / tokmd-analysis-git
        - tokmd-analysis-complexity                     - tokmd-content / tokmd-analysis-content
        - tokmd-analysis-api-surface                    - tokmd-walk / tokmd-analysis-assets
        - tokmd-analysis-effort                         - tokmd-analysis-license / entropy / topics
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
git = ["tokmd-analysis/git", "dep:tokmd-git", "dep:tokmd-cockpit", "tokmd-cockpit/git", "tokmd-context-git/git"]
walk = ["tokmd-analysis/walk"]
content = ["tokmd-analysis/content"]
fun = ["tokmd-analysis/fun", "tokmd-analysis-format/fun"]
topics = ["tokmd-analysis/topics"]
archetype = ["tokmd-analysis/archetype"]
ui = ["dep:dialoguer", "dep:console", "dep:toml", "tokmd-progress/ui"]
```

## Publishing Matrix

### crates.io publish lane
- Rust crates ship in lockstep from the workspace version.
- `tokmd`, `tokmd-core`, contract crates, and most library crates publish through `cargo xtask publish`.

### Non-crates.io products
- `tokmd-python` ships to PyPI via maturin.
- `tokmd-node` ships to npm via napi-rs.
- `tokmd-wasm` ships as a wasm-bindgen/browser package surface for pinned web artifacts.
- `fuzz/` and `xtask/` stay workspace-only support surfaces.

## Sensor Integration Architecture

The sensor subsystem enables multi-sensor pipelines where tokmd acts as one sensor
among many (cargo-deny, cargo-audit, etc.) in a CI/CD fleet.

### Key Crates

| Crate | Role |
|-------|------|
| `tokmd-io-port` | Host-side file access seam used to keep future in-memory/WASM paths honest |
| `tokmd-substrate` | Shared scan context (files, languages, diff range) — built once |
| `tokmd-envelope` | Standardized report contract (`sensor.report.v1`) |
| `tokmd-settings` | Clap-free settings for library/FFI consumers |
| `tokmd-sensor` | `EffortlessSensor` trait + substrate builder |

### Design Principles

1. **Substrate once, sensors many**: A single I/O pass builds `RepoSubstrate`, eliminating redundant scans
2. **Standardized envelope**: All sensors emit `SensorReport` with findings, verdicts, and gates
3. **Clap-free settings**: Lower-tier crates use `ScanOptions` from `tokmd-settings`, not `GlobalArgs`
4. **Finding identity**: `(check_id, code)` tuples enable category-based routing for buildfix automation

## WASM & Browser Runner (post-1.8.0 horizon)

### Foundation Landed in v1.8.0

`1.8.0` did not finish the full browser/WASM milestone, but it did land the core seam work needed to keep that path real:

- `tokmd-io-port` introduces host-abstracted file access with `HostFs`, `ReadFs`, and `MemFs`.
- Feature-stability tests keep lower tiers honest around clap-free and WASM-friendly boundaries.
- The release/devex hardening work keeps CI and release automation boring enough that a future wasm lane can be added without compounding operator noise.

### Next: v1.9.0 — Finish WASM-Ready Core + Browser Runner

Goal: Complete the in-memory/WASM execution path and expose it through a browser-first runner that produces deterministic receipts locally.

Work items:
- Wire `tokmd-io-port` through scan and walk paths so scans can run against a host-provided in-memory substrate instead of only filesystem `PathBuf`s.
- Add an in-memory scan pipeline that accepts `(path, bytes)` inputs and preserves deterministic ordering and capability reporting.
- Keep CLI/Clap separation strict so the library surface stays free of OS-bound argument types.
- Add a `wasm`/`web` feature profile and CI builds for `wasm32-unknown-unknown`, plus parity tests against the native engine.
- Harden the new `tokmd-wasm` crate so its JS-friendly APIs for `lang`, `module`, `export`, and `analyze` are ready for browser packaging.
- Build a minimal browser runner that fetches a GitHub zipball, unpacks in a Worker, runs tokmd locally, and supports progress/cancel/download flows.
- Add caching and guardrails: IndexedDB cache keyed by `(repo,ref,options)`, ETag support, and hard limits for archive size, file count, and bytes read.
- Publish the WASM bundle as a pinned artifact (GitHub Release / npm) for the web app to consume.

Notes: Git-history enrichers (hotspots/churn) are not available in browser WASM mode and must be reported as unavailable in capability reporting.

Non-goals for v1.9.0: no in-browser git churn/hotspot metrics or heavy tooling; provide a backend escape hatch for very large repos or git-based analysis.
