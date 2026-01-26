# Product Contract: tokmd

> This document defines the core product philosophy, invariants, and boundaries of `tokmd`.

## The Core Promise

**tokmd turns *tokei’s* scan into a *receipt*: a compact, deterministic artifact humans can paste into PRs/chats and pipelines/LLMs can parse without shell glue.**

It is not just a counter. It is a **packaging layer** that converts raw counts into trusted artifacts.

## The Problems We Solve

1.  **"Counting" is easy. Using the count is the pain.**
    *   `tokei` gives numbers. Real work needs pasteable summaries, machine-readable payloads, and monorepo views.
    *   `tokmd` replaces fragile `jq | column` chains with a single cross-platform binary.

2.  **LLM workflows need a map, not a dump.**
    *   Pasting source code wastes tokens. Agents need a structured inventory first: What languages? Which modules? Which files are "heavy"?
    *   `tokmd` provides this map as a compact, structured dataset.

3.  **Automation fails by "confident narration".**
    *   Failure mode: "I scanned the repo." (Text is untrusted).
    *   Solution: "Here is the receipt." (Artifacts are trusted).
    *   `tokmd` emits deterministic, versioned, machine-verifiable receipts.

## Product Invariants

These are the rules that make `tokmd` infrastructure, not just a script.

### 1. One Scan, Many Views
Run the scan once. Derive all views (Lang, Module, Export) from that single source of truth.

### 2. Deterministic Output is a Feature
*   Stable sorting (tie-breaks by name/path).
*   Normalized paths (`/` everywhere, even on Windows).
*   Stable schema versioning.
*   Stable redaction hashing.
If the output changes for the same input, it is a bug.

### 3. Receipts Beat Reassurance
Every structured output carries provenance:
*   `schema_version`
*   `tool` version
*   `mode`
*   `scan` args
*   `totals` + `rows`

### 4. Shape, Not Grade
`tokmd` is **not** a productivity metric tool. It avoids "velocity" or "performance" framing. It is a sensor for inventory, distribution, and blast radius.

## Safety Posture

**"If you wouldn't email it, don't paste."**

`tokmd` supports safe sharing via:
*   **Path Redaction**: Hashing file paths and module names (`--redact`).
*   **Blast Radius Control**: Filters (`--max-rows`, `--min-code`) to limit context usage.
*   **Meta Safety**: Ensure no sensitive paths leak in metadata when redaction is active.

## Capabilities

| Capability | Feature |
| :--- | :--- |
| **Human Summary** | Markdown tables, TSV, Top-N compaction. |
| **Machine Receipt** | JSON envelopes with strict schema. |
| **Pipeline Feed** | Streaming JSONL/CSV exports. |
| **Monorepo View** | Module rollup (`crates/`, `packages/`). |
| **Safety** | Redaction, path normalization, ignore profiles. |

## Future Direction

*   **`tokmd run`**: Write canonical receipt bundles to `.runs/tokmd/`.
*   **`tokmd diff`**: Compare two receipts (artifact-to-artifact).
*   **`tokmd.toml`**: Persistent configuration for views and profiles.
