# NOW / NEXT / LATER

> One-screen operational truth. Updated after the `1.10.0-rc.1` release candidate.

## NOW (active)

- **RC aftermath is closed**: `1.10.0-rc.1` shipped on April 29, 2026, the prerelease lane proved green end-to-end, and `main` is in stabilization mode for `1.10.0`.
- **Main must stay boring**: keep CI green, keep `--no-default-features` and browser-capability checks honest, and avoid reintroducing release-only branch noise or operator caveats.
- **Docs and operator surfaces should match reality**: keep roadmap, release instructions, architecture docs, and repo-native commands aligned with what actually shipped in `1.10.0-rc.1`.

## NEXT (short horizon)

- **Stable cut readiness**: finish low-blast-radius polish so `1.10.0` promotion is a metadata/tagging event, not a late code shuffle.
- **WASM-ready continuation**: keep wiring `tokmd-io-port` through scan and walk paths so the in-memory substrate stops being just a seam and becomes a real execution path.
- **Define the next WASM proof bar**: add explicit wasm CI/parity goals for the post-`1.10.0` milestone instead of leaving the work implied.

## LATER (roadmap)

- **Browser runner**: zipball ingestion + in-browser receipt generation.
- **MCP/server mode**: streaming analysis, plugin system, and server surfaces.
- **AST depth**: higher-resolution syntax/AST integration on a later horizon.
