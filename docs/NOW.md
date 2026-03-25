# NOW / NEXT / LATER

> One-screen operational truth during `1.9.0` release readiness.

## NOW (active)

- **Release aftermath is closed** for `1.8.1` and `1.9.0` readiness is now the active stabilization focus while keeping `main` on the normal development lane.
- **Main must stay boring**: keep CI green, keep `--no-default-features` builds honest, and avoid reintroducing release-only branch noise or operator caveats.
- **Docs and operator surfaces should match reality**: keep roadmap, release instructions, architecture docs, and repo-native commands aligned with the `1.9.0` browser/WASM lane that is being prepared to ship.

## NEXT (short horizon)

- **WASM-ready continuation**: keep wiring `tokmd-io-port` through scan and walk paths so the in-memory substrate stops being just a seam and becomes a real execution path.
- **Define the next WASM proof bar**: add explicit wasm CI/parity goals for the next milestone instead of leaving the work implied.
- **Low-blast-radius follow-ons**: prefer narrow docs, compat, and workflow fixes that preserve the newly boring release path and the new effort-estimation surfaces.

## LATER (roadmap)

- **Browser runner**: zipball ingestion + in-browser receipt generation.
- **MCP/server mode**: streaming analysis, plugin system, and server surfaces.
- **AST depth**: higher-resolution syntax/AST integration on a later horizon.
