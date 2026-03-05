# NOW / NEXT / LATER

> One-screen operational truth. Updated per-cycle.

## NOW (active)

- **CI stabilization**: Main is red (Quality Gate fmt + pyo3 exclusion). PR #508 in flight.
- **w58 test expansion**: 6 PRs (#499–#504) rebased, CI running — test-only, low risk.
- **Quality gate DX**: PR #506 merged — gate accumulates all failures with ✅/❌ indicators.

## NEXT (merge queue)

- **v1.8 WASM-ready seams**: Abstract host I/O (fs, path) behind trait ports for `wasm32-unknown-unknown` target.
- **Determinism hardening**: Stable ordering tie-breaks, CRLF/LF normalization, path canonicalization.
- **Schema docs sync**: Ensure docs/SCHEMA.md, docs/schema.json match code-generated output.
- **Feature boundary expansion**: Broaden `xtask boundaries-check` to cover more tier constraints.

## LATER (roadmap)

- **v1.9 browser runner**: Zipball ingestion + in-browser receipt generation.
- **v2.0 MCP server**: Streaming analysis, plugin system, server mode.
- **v4.0 Adze AST**: Full AST integration (long-horizon).
- **Bindings parity**: tokmd-core run_json + Python/Node bindings with explicit parity tests.
