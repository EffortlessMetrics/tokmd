# NOW.md — Current State of tokmd

> Last updated: 2025-07-24

## Where We Are

v1.7.2 shipped. 56 crates in the workspace. **2,496+ new tests** merged
across 34 PRs. Deep coverage across all crates. CI green on main.

---

## NOW — current status

- **2,496+ new tests merged** across 34 PRs.
- **Deep coverage across all 56 crates.** Every tier covered with unit,
  integration, snapshot, BDD, and E2E tests.
- **Full property testing.** `proptest` suites across 10+ crates for
  invariant verification.
- **Full BDD test suites.** Every `tokmd-analysis-*` microcrate has
  dedicated `tests/bdd.rs` scenarios.
- **Full snapshot testing (insta).** Format renderers, analysis outputs,
  and CLI commands covered with golden snapshots.
- **Full CLI E2E testing.** All major subcommands exercised with
  `assert_cmd` + `predicates`.
- **CI green on main.** Full gate passing.

---

## NEXT — immediate priorities

- **Verify CI green after massive test merge wave.** Monitor for flaky
  tests or regressions introduced by the 34-PR merge.
- **Monitor for snapshot drift.** Ensure insta snapshots remain stable
  as codebase evolves.
- **v1.7.x release preparation.** Changelog, version bumps, publish
  dry-run.
- **WASM preparation (v1.8 track).** Host IO abstraction, in-memory
  scan pipeline, `wasm` feature profile.

---

## LATER — roadmap horizon

- **v1.9 "Browser runner."** WASM API crate (`tokmd-wasm`), zipball
  ingestion, in-browser receipt generation without server-side compute.
- **EffortlessMetrics/adze integration track.** AST seams for
  tree-sitter/Adze-based complexity and function extraction.
- **Full mutation testing coverage.** Expand `cargo-mutants` across all
  crates for comprehensive test quality verification.
