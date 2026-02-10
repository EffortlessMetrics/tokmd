# Gemini Context: tokmd

## Project Overview

**tokmd** is a code intelligence tool for humans, machines, and LLMs. It analyzes codebases to produce deterministic "receipts" (schema-compliant JSON/Markdown artifacts) and derived insights (metrics, risk analysis, context packing). It is built on top of `tokei`.

**Key Value Proposition:**
*   **Deterministic Receipts:** Stable output for CI/CD and diffing.
*   **LLM Ready:** Token estimation and context window packing.
*   **Git Analysis:** Hotspots, freshness, and coupling metrics.
*   **Multi-language Support:** Native CLI, plus Python and Node.js bindings.

## Architecture

The project follows a **tiered microcrate architecture** to ensure strict separation of concerns and dependency rules.

*   **Tier 0 (Contracts):** `tokmd-types`, `tokmd-analysis-types`, `tokmd-envelope`, `tokmd-substrate`. Pure data structures, no heavy dependencies.
*   **Tier 1 (Core):** `tokmd-scan` (tokei wrapper), `tokmd-model` (aggregation), `tokmd-redact` (hashing).
*   **Tier 2 (Adapters):** `tokmd-format` (rendering), `tokmd-walk` (fs), `tokmd-content` (scanning), `tokmd-git`.
*   **Tier 3 (Orchestration):** `tokmd-analysis` (enrichers), `tokmd-analysis-format`, `tokmd-gate`.
*   **Tier 4 (Facade):** `tokmd-config`, `tokmd-core` (library facade + FFI).
*   **Tier 5 (Products):** `tokmd` (CLI binary), `tokmd-python`, `tokmd-node`.

**Dependency Rule:** Lower tiers must **never** depend on higher tiers.

## Build & Development

The project uses `cargo` and `just` for task management.

### Key Commands

| Task | Command | Description |
| :--- | :--- | :--- |
| **Build** | `cargo build` | Build all crates. |
| **Test (Unit)** | `cargo test` | Run unit tests. |
| **Test (All)** | `cargo test --workspace` | Run all tests in workspace. |
| **Lint** | `just lint` | `cargo clippy --all-features -- -D warnings` |
| **Format** | `just fmt` | `cargo fmt` |
| **Publish Plan** | `just publish-plan` | Dry-run publish sequence. |

### Testing Strategy

1.  **Determinism is King:** Outputs must be byte-stable. Use `insta` for snapshot testing.
    *   **CRITICAL:** Line endings must be LF-normalized.
    *   If output logic changes, verify diffs carefully before running `cargo insta review`.
2.  **Property Testing:** `proptest` used for invariant checking (hashing, normalization).
3.  **Fuzzing:** `libfuzzer` targets in `fuzz/` for parsing and security-critical paths.
4.  **Mutation Testing:** `cargo-mutants` runs on CI to ensure test coverage quality.

## Coding Conventions

*   **Style:** Standard Rust (rustfmt).
*   **Linting:** Strict clippy (`-D warnings`).
*   **Schema Versioning:**
    *   Core receipts: `SCHEMA_VERSION`
    *   Analysis receipts: `ANALYSIS_SCHEMA_VERSION`
    *   **Rule:** Increment version on breaking changes (renaming fields, changing types).
*   **Path Normalization:** Always normalize paths to use forward slashes (`/`), even on Windows.
*   **Context Awareness:** When adding features, consider "Is this deterministic?" and "Does this belong in a specific tier?"

## Subsystem: Sensors

`tokmd` supports a multi-sensor pipeline via `tokmd-sensor` and `tokmd-envelope`.
*   **Substrate:** `RepoSubstrate` is built once (scan) and shared.
*   **Report:** Sensors emit `SensorReport` (standardized envelope).

## Reference Files

*   `Cargo.toml`: Workspace configuration.
*   `Justfile`: Task runner shortcuts.
*   `docs/architecture.md`: Detailed architectural guidelines.
*   `docs/SCHEMA.md`: Output data shapes.
