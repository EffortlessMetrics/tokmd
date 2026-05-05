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
                       tokmd-envelope, tokmd-io-port
         ↓
Tier 1 (Core)          tokmd-scan, tokmd-model, tokmd-sensor
         ↓
Tier 2 (Adapters)      tokmd-format, tokmd-git
         ↓
Tier 3 (Orchestration) tokmd-analysis, tokmd-cockpit,
                       tokmd-gate
         ↓
Tier 4 (Facade)        tokmd-core
         ↓
Tier 5 (Products)      tokmd (CLI), tokmd-python, tokmd-node, tokmd-wasm
```

Helper boundaries that do not need an independent crates.io package live as
single-responsibility owner modules: module-key logic in `tokmd-model`,
path/exclude/math/tokeignore/walk helpers in `tokmd-scan`, shared analysis
limits and path helpers in `tokmd-analysis-types`, redaction/scan-args/badge
and export-tree, fun renderers, and analysis rendering in `tokmd-format`, assets/fun and
metric/security analysis enrichers plus content/import/Git adapters in
`tokmd-analysis`, context policy/git helpers in
`tokmd-core`, sensor substrate context in `tokmd-sensor`, and
CLI/config/progress/tool-schema/explain wiring in `tokmd`.

### Tier 0: Contracts (Pure Data)

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `tokmd-types` | Core receipt DTOs (`LangRow`, `ModuleRow`, `FileRow`, `Totals`) | `serde` only |
| `tokmd-analysis-types` | Analysis receipt DTOs | `serde`, `tokmd-types` |
| `tokmd-settings` | Clap-free settings types (`ScanOptions`, `LangSettings`, etc.) | `serde`, `tokmd-types` |
| `tokmd-envelope` | Cross-fleet `SensorReport` contract plus FFI envelope parser/extractor helpers | `serde`, `serde_json` |
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
| `tokmd-scan` | Wraps tokei library for code scanning and owns file walking helpers |
| `tokmd-model` | Aggregation logic: tokei results → tokmd receipts |
| `tokmd-sensor` | `EffortlessSensor` trait + `build_substrate()` builder |

### Tier 2: Adapters

| Crate | Purpose | Feature Flag |
|-------|---------|--------------|
| `tokmd-format` | Output rendering (Markdown, TSV, JSON, CSV, JSONL, CycloneDX) | — |
| `tokmd-git` | Git history analysis via shell `git log` | `git` |

### Tier 3: Orchestration

| Crate | Purpose |
|-------|---------|
| `tokmd-analysis` | Analysis orchestration with preset system; owner modules for derived metrics, archetype, fingerprint, preset grid, topics, assets, fun, complexity, entropy, Halstead, license, maintainability, API surface, effort, near-duplicate, content, and import enrichers |
| `tokmd-format::analysis` | Analysis output rendering (Markdown, JSON, SVG, HTML, etc.) |
| `tokmd-cockpit` | PR cockpit metrics computation and rendering |
| `tokmd-gate` | Policy evaluation with JSON pointer rules |

### Tier 4: Facade

| Crate | Purpose |
|-------|---------|
| `tokmd-core` | Library facade with FFI layer; exposes analysis formatting via `analysis_facade` module (see ADR-001) |

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
3. **Tier boundary compliance via facade** — Tier 5 products access Tier 3 orchestration only through Tier 4 facades (e.g., `tokmd-core::analysis_facade`). See ADR-001.
4. **Feature flags control optional adapters** — `git`, `walk`, `content` features
5. **IO adapters depend on domain/contracts, never reverse**

## Data Flow

### Flow A: Repository Inventory (lang/module/export)

```
Filesystem → tokmd-scan::walk → tokmd-scan (tokei) → tokmd-model → tokmd-format → Output
                                ↓
                          BTreeMap (deterministic)
                                ↓
                    Receipt DTOs (tokmd-types)
```

### Flow B: Analysis (analyze/cockpit)

```
Receipt / export / paths → tokmd-analysis → Enrichers → tokmd-format::analysis → Output
                                ↓
                 ┌──────────────┴─────────────────────────────┐
                 ↓                                            ↓
       Core enrichers                                  Optional adapters
        - tokmd-analysis derived/grid modules           - tokmd-git / analysis git module
        - tokmd-analysis assets/fun modules             - tokmd-scan::walk / license / entropy / topics
        - tokmd-analysis complexity/halstead modules    - tokmd-analysis content modules
        - tokmd-analysis API surface/effort/import modules
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
git = ["tokmd-analysis/git", "dep:tokmd-git", "dep:tokmd-cockpit", "tokmd-cockpit/git", "tokmd-core/git"]
walk = ["tokmd-analysis/walk"]
content = ["tokmd-analysis/content"]
fun = ["tokmd-analysis/fun", "tokmd-format/fun"]
topics = ["tokmd-analysis/topics"]
archetype = ["tokmd-analysis/archetype"]
ui = ["dep:dialoguer", "dep:console", "dep:toml", "dep:indicatif"]
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
| `tokmd-sensor::substrate` | Shared scan context (files, languages, diff range) — built once |
| `tokmd-envelope` | Standardized report contract (`sensor.report.v1`) |
| `tokmd-settings` | Clap-free settings for library/FFI consumers |
| `tokmd-sensor` | `EffortlessSensor` trait + substrate builder |

### Design Principles

1. **Substrate once, sensors many**: A single I/O pass builds `RepoSubstrate`, eliminating redundant scans
2. **Standardized envelope**: All sensors emit `SensorReport` with findings, verdicts, and gates
3. **Clap-free settings**: Lower-tier crates use `ScanOptions` from `tokmd-settings`, not `GlobalArgs`
4. **Finding identity**: `(check_id, code)` tuples enable category-based routing for buildfix automation

## WASM & Browser Runner

### Shipped foundation and product surface

The browser/WASM lane is now a shipped product surface:

- `tokmd-io-port` plus the in-memory scan/model/core workflow paths keep lower tiers host-abstracted and deterministic on ordered in-memory inputs.
- `tokmd-wasm` exposes the browser-facing `lang`, `module`, `export`, and browser-safe `analyze` entrypoints.
- CI includes wasm compile/tests plus native-vs-wasm parity coverage for the browser-safe modes.
- `web/runner` boots the real wasm bundle in a dedicated worker, renders capabilities, shows the latest successful result, and supports JSON download.
- Public browser repo loading uses the GitHub tree and contents APIs to materialize ordered `{ path, text }` inputs locally.
- The `tokmd-wasm` browser bundle is now a versioned release artifact consumed from `web/runner/vendor/tokmd-wasm`.
- Browser runner guardrails already landed, including caching, authenticated fetch options, and rate-limit/progress handling.

### Supported browser-safe contract today

- Modes: `lang`, `module`, `export`
- Analyze presets: `receipt`, `estimate`
- Input contract: ordered in-memory rows, not filesystem paths
- Acquisition strategy: GitHub tree + contents API, not zipball fetch

Host-backed enrichers remain explicit capability misses in browser mode. Git-history signals such as hotspots and churn are intentionally unavailable there today.

### Current browser constraints

- Broaden browser analysis only where the preset can stay rootless and capability-honest.

### Non-goals for v1.9.0

- No browser-side git-history churn/hotspot metrics or other heavy host tooling.
- No browser zipball ingestion as the primary supported path while tree+contents is the stable browser-safe acquisition strategy.
