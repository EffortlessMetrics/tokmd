# Handoff Bundles

`tokmd handoff` creates a **self-contained bundle** for LLM review and
automation. It is intended to be pasted or uploaded as a stable, deterministic
artifact when a coding agent needs the right source slice without the whole
repository.

Use handoff when the job is:

```text
Give my coding agent the right context and proof expectations.
```

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

## Agent Workflow

For a plain repository handoff, start with the current risk preset and a bounded
budget:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --out-dir .handoff
```

Give the agent these files in order:

1. `.handoff/manifest.json` for the authoritative artifact index, token budget,
   exclusions, and included-file list.
2. `.handoff/intelligence.json` for tree, hotspot, complexity, and derived
   signals.
3. `.handoff/code.txt` for the selected source bundle.
4. `.handoff/map.jsonl` when the agent needs full inventory or path lookup.

For PR repair or review work in this repository, pair the handoff with cockpit
and proof receipts:

```bash
cargo xtask proof --profile affected --base origin/main --head HEAD --plan \
  --plan-json target/proof/proof-plan.json

cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json

tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --doc-artifacts-check target/docs/doc-artifacts-check.json \
  --review-packet-dir .tokmd/review

cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json

tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --out-dir .handoff
```

Then give the agent the handoff plus the review evidence:

- `.tokmd/review/comment.md` for the short review summary.
- `.tokmd/review/review-map.md` for what to inspect first and reproduction
  commands.
- `.tokmd/review/evidence.json` for available, missing, stale, degraded,
  skipped, and unavailable evidence.
- `target/proof/proof-plan.json` for expected proof commands.
- `target/tokmd/review-packet-check.json` for packet verification.

The current `handoff` command does not yet import review-map or proof-plan
artifacts into `.handoff/` automatically. Keep the directories adjacent and pass
both to the agent when proof state matters.

## Output Tree

```
<out-dir>/
├── manifest.json      # authoritative index (schema v5)
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

## Agent Guardrails

When using handoff output as an agent work order:

- Treat missing, stale, degraded, skipped, or unavailable evidence as work to
  resolve, not as passing proof.
- Run reproduction commands from `.tokmd/review/review-map.md` before claiming a
  repair is proven.
- Keep generated receipts with the work when they explain review or proof state.
- Do not promote advisory proof, enable default Codecov upload, or turn cockpit
  into a merge verdict from a handoff bundle.

## Determinism Notes

- Output directory is excluded from scans by construction.
- All selection strategies and ordering are deterministic.

## Schema

See `docs/handoff.schema.json` and `docs/handoff-schema.md`.
