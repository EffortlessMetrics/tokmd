# NOW / NEXT / LATER

> One-screen operational truth. Updated after the v1.11 browser runtime polish implementation.

## NOW (active)

- **Browser runtime polish is closed on main**: cache semantics, worker/repo-load progress, retry/rate-limit guidance, and session-only authenticated fetch UX are implemented.
- **Main must stay boring**: keep CI green, keep `--no-default-features` builds honest, and avoid reintroducing release-only branch noise or operator caveats.
- **Docs and operator surfaces should match reality**: keep roadmap, release instructions, architecture docs, and repo-native commands aligned with what is actually implemented.

## NEXT (short horizon)

- **Cockpit/review evidence hardening**: keep improving cockpit as the PR-review evidence surface before adding any separate `review` command.
- **Architecture consolidation prep**: prefer proof-scoped SRP module consolidation over new implementation microcrates.

## LATER (roadmap)

- **Browser runner**: zipball ingestion remains later; in-browser receipt generation shipped in `1.9.0`.
- **MCP/server mode**: streaming analysis, plugin system, and server surfaces.
- **AST depth**: higher-resolution syntax/AST integration on a later horizon.
