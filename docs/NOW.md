# NOW.md — Current State of tokmd

> Last updated: 2026-02-25

## Where We Are

v1.7.2 shipped. 56 crates in the workspace. ~4,350 `#[test]` annotations,
218 `proptest!` blocks, 16 fuzz targets, mutation testing on CI.

---

## NOW — actively in-flight

- **Massive test coverage expansion.** BDD scenarios, property tests,
  snapshot tests, integration tests, E2E tests, fuzz targets, and mutation
  testing being added across all crates. Most analysis-* crates now have
  dedicated `tests/bdd.rs`, `tests/properties.rs`, and snapshot suites.
- **CI stabilization and PR merge wave.** Multiple test-addition branches
  open and landing. Green-lining the matrix before moving to new features.
- **Schema/doc synchronization hardening.** CLAUDE.md, AGENTS.md, GEMINI.md,
  ROADMAP.md, and docs/ kept in sync with actual crate topology (56 crates).
- **Analysis crate test coverage.** Each `tokmd-analysis-*` microcrate
  getting BDD, property, and snapshot tests. Enricher contract verification.

---

## NEXT — after current wave lands

- **v1.8 "WASM-ready core" preparation.** Host IO abstraction (ports),
  in-memory scan path (`Vec<(path, bytes)>`), clap-free library hardening,
  `wasm32-unknown-unknown` CI build target.
- **PackPlan unification.** Converge `context` and `handoff` file-selection
  logic into a shared plan/policy layer.
- **Cockpit accuracy improvements.** Diff coverage precision, determinism
  gate robustness, baseline comparison fidelity.
- **Feature-gated enricher cleanup.** Ensure `halstead`, `content`, `git`,
  `walk` feature flags compile cleanly in isolation.
- **Bindings fidelity.** Python and Node.js parity — all workflows
  (`lang`, `module`, `export`, `analyze`, `diff`) returning identical
  receipts to CLI.

---

## LATER — roadmap horizon

- **v1.9 "Browser runner."** WASM API crate (`tokmd-wasm`), zipball
  ingestion, in-browser receipt generation without server-side compute.
- **Adze integration track.** AST seams for tree-sitter/Adze-based
  complexity and function extraction (v3.0+/v4.0 horizon).
- **Advanced analysis enrichers.** API surface coverage, architecture
  graph visualization, semantic diff intelligence.
- **crates.io publication readiness.** Publishability audit for all 56
  crates, dependency ordering, `cargo xtask publish` dry-run clean.
