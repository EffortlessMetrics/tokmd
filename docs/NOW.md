# NOW / NEXT / LATER

> One-screen operational truth. Updated after the `1.10.0-rc.1` release candidate on 2026-04-29.

## NOW (active)

- **RC proof lane is active**: `1.10.0-rc.1` shipped and `main` is focused on release-candidate proof, regression triage, and stable-promotion readiness.
- **Keep `main` boring under RC constraints**: keep CI green, keep `--no-default-features` builds honest, and avoid scope creep that widens the release surface before `1.10.0`.
- **Docs and operator surfaces must match shipped RC reality**: keep roadmap, release instructions, architecture docs, and repo-native commands aligned with what shipped in `1.10.0-rc.1`.

## NEXT (short horizon)

- **Stable promotion prep**: close only high-signal follow-ons needed to promote `1.10.0-rc.1` to `1.10.0`.
- **Browser runtime polish planning**: stage low-risk work for the `1.11.0` lane (cache behavior, progress events, retry/rate-limit UX, authenticated fetch).
- **Low-blast-radius follow-ons**: prefer narrow docs, compat, and workflow fixes that preserve the RC proof chain and release boringness.

## LATER (roadmap)

- **Browser runner**: zipball ingestion + in-browser receipt generation.
- **MCP/server mode**: streaming analysis, plugin system, and server surfaces.
- **AST depth**: higher-resolution syntax/AST integration on a later horizon.
