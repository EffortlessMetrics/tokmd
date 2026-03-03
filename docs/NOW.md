# NOW.md — Current State of tokmd

> Last updated: 2026-02-26

## Where We Are

v1.7.2 shipped. 56 crates in the workspace. **6,593** `#[test]` annotations,
**250** `proptest!` blocks, **19** fuzz targets, mutation testing on CI.

Deep test coverage expansion complete: **~2,240 new tests** landed across all
tiers via 36+ merged PRs. CI green on main with full gate passing.

---

## NOW — completed in latest wave

- **Deep test coverage expansion complete.** 2,240+ new tests across all tiers
  covering boundary verification, determinism regression, error handling,
  snapshot tests for all format renderers, deep analysis crate tests
  (complexity, halstead, near-dup, topics, entropy), CLI E2E tests for all
  major subcommands, FFI and workflow integration tests, and property tests
  expanded across 14 crates.
- **36+ PRs merged** in the latest wave. BDD scenarios, property tests,
  snapshot tests, integration tests, E2E tests, fuzz targets, and mutation
  testing added across all crates. Every `tokmd-analysis-*` microcrate now
  has dedicated `tests/bdd.rs`, `tests/properties.rs`, and snapshot suites.
- **CI stabilization and PR merge wave landed.** All test-addition branches
  merged. Green matrix across the board.
- **Schema/doc synchronization hardened.** CLAUDE.md, AGENTS.md, GEMINI.md,
  ROADMAP.md, and docs/ in sync with actual crate topology (56 crates).

---

## NEXT — v1.8 "WASM-ready core" track

- **Host IO abstraction (ports).** Enumerate files, read bytes, clock,
  optional logging/progress. Native uses FS; WASM uses in-memory substrate.
- **In-memory scan pipeline.** Scan path accepting `Vec<(path, bytes)>`
  instead of filesystem `PathBuf`s.
- **WASM feature profile.** `wasm` feature that disables OS-bound pieces
  (`git`, `dirs`, `std::process`) and enables in-memory I/O.
- **WASM CI builds + conformance tests.** `cargo build --target
  wasm32-unknown-unknown` in CI; golden tests verifying native/WASM parity.
- **PackPlan unification.** Converge `context` and `handoff` file-selection
  logic into a shared plan/policy layer.
- **Cockpit accuracy improvements.** Diff coverage precision, determinism
  gate robustness, baseline comparison fidelity.
- **Stale branch cleanup.** ~150 merged test branches to prune.

---

## LATER — roadmap horizon

- **v1.9 "Browser runner."** WASM API crate (`tokmd-wasm`), zipball
  ingestion, in-browser receipt generation without server-side compute.
- **EffortlessMetrics/adze integration track.** AST seams for
  tree-sitter/Adze-based complexity and function extraction (v3.0+/v4.0
  horizon).
- **Python/Node binding parity tests.** All workflows (`lang`, `module`,
  `export`, `analyze`, `diff`) returning identical receipts to CLI.
- **crates.io stable release (v1.8.0).** Publishability audit for all 56
  crates, dependency ordering, `cargo xtask publish` dry-run clean.
