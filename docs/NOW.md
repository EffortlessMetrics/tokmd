# NOW / NEXT / LATER

> One-screen operational truth. Updated per-cycle.

## NOW (active)

- **CI validation**: 46+ PRs merged in massive wave (Waves 58-65). Waiting for CI run on latest main commit.
- **Test expansion (Wave 66)**: 6 agents adding determinism hardening, schema contracts, cross-crate integration, enricher depth, CLI edge cases, and gate/cockpit/config deep tests.
- **Test count**: 21,884 tests across 56+ crates (up from 11,991 at v1.7.2).
- **Quality Gate**: Fixed — excludes tokmd-python (PyO3 needs Python headers).
- **WASM foundation**: `tokmd-io-port` crate merged (ReadFs trait + MemFs).

## NEXT (merge queue)

- **v1.7.3 release**: Tag once CI confirms green. Includes: massive test expansion, Quality Gate fix, sensor determinism (BTreeSet), io-port crate, performance improvements, .jules cleanup.
- **v1.8 WASM-ready seams**: Wire `tokmd-io-port` into `tokmd-scan` and `tokmd-walk`. Abstract host I/O for `wasm32-unknown-unknown`.
- **Determinism hardening**: Wave 66 adds dedicated determinism tests. Follow up with CRLF/LF edge cases.
- **Schema docs sync**: Ensure docs/SCHEMA.md, docs/schema.json match code-generated output.

## LATER (roadmap)

- **v1.9 browser runner**: Zipball ingestion + in-browser receipt generation.
- **v2.0 MCP server**: Streaming analysis, plugin system, server mode.
- **v4.0 Adze AST**: Full AST integration (long-horizon).
- **Bindings parity**: tokmd-core run_json + Python/Node bindings with explicit parity tests.
