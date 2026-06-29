# Index Freshness

When adding run packets, always consider if the generated rollup indexes need updating.
The `cargo xtask jules-index` command will parse all `envelope.json` and `decision.md` files in `.jules/runs/` and generate a fresh `RUNS_ROLLUP.md`.
Running this manually prevents drift.
