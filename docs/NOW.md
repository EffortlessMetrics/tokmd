# NOW / NEXT / LATER

> One-screen operational truth. Updated during stabilization.

## NOW (active)

- **CI cleanup**: macOS Nix is out of the main CI workflow; Linux Nix remains in the fast path.
- **Queue cleanup**: the broad draft PR backlog has been collapsed into narrow merges and honest follow-ups.
- **Stabilization focus**: keep `main` green, keep `--no-default-features` builds honest, and reduce branch noise from automation artifacts.

## NEXT (short horizon)

- **Main CI boringness**: confirm the cleaned workflow shape stays green on normal pushes.
- **Agent boundary cleanup**: keep `.claude/` and `.jules/` as checked-in adapter surfaces; keep only runtime state out of git. Root `.jules/runs/` stays untracked, while curated `.jules/deps/**` history stays checked in.
- **Low-blast-radius devex**: continue small workflow, docs, and compat fixes that reduce false red and review noise.
- **Release discipline**: keep release work paused until `main` is boring again.

## LATER (roadmap)

- **WASM-ready seams**: continue wiring `tokmd-io-port` into scan and walk paths.
- **Browser runner**: zipball ingestion + in-browser receipt generation.
- **MCP/server mode**: streaming analysis, plugin system, and server surfaces.
- **AST depth**: higher-resolution syntax/AST integration on a later horizon.
