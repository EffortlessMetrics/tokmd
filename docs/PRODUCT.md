# Product Contract: tokmd

> This document defines the core product philosophy, invariants, and boundaries of `tokmd`.

## The Core Promise

**tokmd transforms code scans into actionable intelligence: receipts for automation, metrics for understanding, and signals for decision-making.**

It is not just a counter. It is the **fast deterministic code-intelligence and
review-receipt engine** in the Effortless Metrics evidence stack. It converts
raw counts into trusted code artifacts and derived insights without trying to
own the whole evidence transport layer.

## The Problems We Solve

1.  **"Counting" is easy. Using the count is the pain.**
    *   `tokei` gives numbers. Real work needs pasteable summaries, machine-readable payloads, and monorepo views.
    *   `tokmd` replaces fragile `jq | column` chains with a single cross-platform binary.

2.  **LLM workflows need a map, not a dump.**
    *   Pasting source code wastes tokens. Agents need a structured inventory first: What languages? Which modules? Which files are "heavy"? Will it fit in context?
    *   `tokmd` provides this map as a compact, structured dataset with token estimates.
    *   `tokmd context` intelligently packs files into context windows within token budgets.

3.  **Automation fails by "confident narration".**
    *   Failure mode: "I scanned the repo." (Text is untrusted).
    *   Solution: "Here is the receipt." (Artifacts are trusted).
    *   `tokmd` emits deterministic, versioned, machine-verifiable receipts.

4.  **Understanding requires more than counts.**
    *   Raw numbers don't tell you where the risk is, what's stale, or how effort is distributed.
    *   `tokmd analyze` derives actionable signals from receipt data.

## Product Invariants

These are the rules that make `tokmd` infrastructure, not just a script.

### 1. One Scan, Many Views
Run the scan once. Derive all views (Lang, Module, Export, Analysis) from that single source of truth.

### 2. Deterministic Output is a Feature
*   Stable sorting (tie-breaks by name/path).
*   Normalized paths (`/` everywhere, even on Windows).
*   Stable schema versioning.
*   Stable redaction hashing.
*   Integrity hashes for verification.
If the output changes for the same input, it is a bug.

### 3. Receipts Beat Reassurance
Every structured output carries provenance:
*   `schema_version`
*   `tool` version
*   `mode`
*   `scan` args
*   `totals` + `rows`
*   `integrity` hash

### 4. Shape, Not Grade
`tokmd` is **not** a productivity metric tool. It avoids "velocity" or "performance" framing. It is a sensor for inventory, distribution, risk signals, and blast radius.

### 5. Signals, Not Scores
Analysis provides information, not judgments:
*   "Doc density is 12%" — not "Documentation is poor"
*   "File changed 47 times" — not "This is a problem file"
*   Users interpret signals in their context.

### 6. Code Evidence, Not The Backplane
`tokmd` produces deterministic code evidence. It does not own the universal
evidence bundle, merge verdict, or multi-tool inventory. In the wider Effortless
Metrics stack, `evidencebus` is the schema-first backplane that validates,
bundles, inventories, and exports evidence from `tokmd`, `mergecode`, CI
sensors, gates, perf tools, and other producers.

## Safety Posture

**"If you wouldn't email it, don't paste."**

`tokmd` supports safe sharing via:
*   **Path Redaction**: Hashing file paths and module names (`--redact`).
*   **Blast Radius Control**: Filters (`--max-rows`, `--min-code`) to limit context usage.
*   **Meta Safety**: Ensure no sensitive paths leak in metadata when redaction is active.
*   **Resource Limits**: Caps on files, bytes, and commits scanned (`--max-*` flags).

## Capabilities

| Capability | Feature |
| :--- | :--- |
| **Human Summary** | Markdown tables, TSV, Top-N compaction, tree views. |
| **Machine Receipt** | JSON envelopes with strict schema, CycloneDX SBOM. |
| **Pipeline Feed** | Streaming JSONL/CSV exports. |
| **Monorepo View** | Module rollup (`crates/`, `packages/`). |
| **Safety** | Redaction, path normalization, ignore profiles. |
| **Derived Analytics** | Doc density, test density, distribution, COCOMO. |
| **Git Intelligence** | Hotspots, freshness, coupling, bus factor. |
| **Context Planning** | Token estimation, window fit analysis. |
| **Context Packing** | Budget-aware file selection for LLM context windows. |
| **Visualization** | SVG badges, Mermaid diagrams, HTML reports, tree output. |

## Analysis Presets

| Preset | Scope | Use Case |
| :--- | :--- | :--- |
| `receipt` | Derived metrics only | Quick health check |
| `health` | + TODO density | Code hygiene review |
| `risk` | + Git metrics | Risk assessment |
| `supply` | + Assets + deps | Dependency audit |
| `architecture` | + Import graph | Structure analysis |
| `topics` | + Semantic topics | Domain discovery |
| `security` | + License + entropy | Security review |
| `identity` | + Archetype + fingerprint | Project profiling |
| `git` | + Predictive churn | Trend analysis |
| `deep` | Everything | Comprehensive review |
| `fun` | Novelty outputs | Team morale |

## Boundaries (Non-Goals)

`tokmd` explicitly does **not**:
*   Format or lint code (use rustfmt, eslint)
*   Implement vulnerability detection (tokmd delegates to cargo-audit/npm audit but does not maintain its own advisory database)
*   Replace test runners (proof planning may route `cargo test`, pytest, or
    coverage commands, but those tools remain the source of build/test truth)
*   Own the evidence transport backplane or global merge decision
*   Parse AST deeply (uses heuristics, not full parsers)
*   Score or rank developers
*   Provide absolute quality judgments

## Future Direction

*   **MCP Server**: Future server/resource integration with Claude and MCP-compatible tools; `tokmd tools` already covers tool schema generation
*   **Watch Mode**: Continuous analysis during development
*   **Plugin System**: WASM-based extensible enrichers
*   **Smart Suggestions**: Context-aware file recommendations for LLM workflows
