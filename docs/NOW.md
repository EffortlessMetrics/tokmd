# NOW / NEXT / LATER

> One-screen operational truth. Updated after the `1.10.0` release.

## NOW (active)

- **Release aftermath is closed**: `1.10.0` is out, the release pipeline proved green end-to-end with the CI control plane, trust hardening, WASM truth, and proof stability work complete. `main` is back to the normal development lane.
- **Main must stay boring**: keep CI green, keep `--no-default-features` builds honest, and avoid reintroducing release-only branch noise or operator caveats.
- **Docs and operator surfaces should match reality**: keep roadmap, release instructions, architecture docs, and repo-native commands aligned with what actually shipped in `1.10.0`.

## NEXT (short horizon)

- **Browser runtime polish (v1.11.0)**: define cache key and invalidation semantics, emit progress events, improve retry and rate-limit UX, and partition authenticated fetch/cache behavior safely.
- **Low-blast-radius follow-ons**: prefer narrow docs, compat, and workflow fixes that preserve the newly boring release path and the new effort-estimation surfaces.

## LATER (roadmap)

- **Browser runner**: zipball ingestion remains later; in-browser receipt generation shipped in `1.9.0`.
- **MCP/server mode**: streaming analysis, plugin system, and server surfaces.
- **AST depth**: higher-resolution syntax/AST integration on a later horizon.
