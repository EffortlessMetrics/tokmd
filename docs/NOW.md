# NOW.md — Current State of tokmd

> Last updated: 2025-07-28

## Where We Are

v1.7.2 shipped. 56 crates in the workspace. **11,991+ tests** total.
55/56 crates have `tests/` directories. CI green on main (macOS gated
to main-only pushes). Two full test-expansion cycles complete.

---

## NOW — active work

- **Test infrastructure comprehensive.** 11,991+ tests across unit,
  integration, snapshot, BDD, proptest, and E2E. 16 test-expansion PRs
  merged in the latest cycle on top of the earlier 36-PR wave.
- **CI stabilized.** macOS jobs gated to main-only pushes (#409). Nix
  clippy lint and rustfmt fixes landed (#407, #390). Full gate green.
- **Perf improvement merged.** Reduced allocations in token stream
  formatting (top-of-tree commit).
- **Determinism hardened.** Byte-stable output regression suite and
  deterministic ordering locks in place.

---

## NEXT — immediate priorities

- **v1.8 WASM readiness.** Host IO abstraction, in-memory scan
  pipeline, `wasm` feature profile, WASM CI builds.
- **Bindings parity.** Ensure Python and Node.js bindings cover the
  full workflow surface (analyze, diff, sensor).
- **Schema hardening.** Lock remaining schema versions with contract
  tests; publish updated `docs/schema.json`.
- **Mutation testing expansion.** Broaden `cargo-mutants` coverage
  across more crates for test quality verification.

---

## LATER — roadmap horizon

- **v1.9 browser runner.** `tokmd-wasm` crate, zipball ingestion,
  in-browser receipt generation without server-side compute.
- **v2.0 MCP server.** `tokmd serve` for native Claude/MCP-client
  integration, streaming analysis, plugin system.
- **Adze AST integration.** Tree-sitter/Adze-based complexity and
  function extraction (v4.0 long-term track).
