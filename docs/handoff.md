# Handoff Bundles

`tokmd handoff` creates a **self-contained bundle** for LLM review and automation.  
It is intended to be pasted or uploaded as a stable, deterministic artifact.

## CLI

```bash
# Default output to .handoff/
tokmd handoff

# Custom output directory
tokmd handoff --out-dir ./artifacts/handoff

# Control token budget and strategy
tokmd handoff --budget 128k --strategy spread

# Disable git enrichment
tokmd handoff --no-git
```

## Output Tree

```
<out-dir>/
├── manifest.json      # authoritative index (schema v3)
├── map.jsonl          # full file inventory (JSONL)
├── intelligence.json  # summary signals (payload-only)
└── code.txt           # token-budgeted code bundle
```

## Consumption Pattern

1. **Read `manifest.json` first.**  
   It is the authoritative index, lists artifacts, included files, and exclusions.
2. **Use `map.jsonl`** for full inventory or downstream tooling.
3. **Use `intelligence.json`** as a warning label (tree, hotspots, derived).
4. **Use `code.txt`** as the LLM bundle content.

## Determinism Notes

- Output directory is excluded from scans by construction.
- All selection strategies and ordering are deterministic.

## Schema

See `docs/handoff.schema.json` and `docs/handoff-schema.md`.
