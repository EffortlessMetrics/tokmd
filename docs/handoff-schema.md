# Handoff Schema (manifest.json)

The handoff bundle is a directory of artifacts intended for LLM review and automation.  
The authoritative index is `manifest.json`, validated by `docs/handoff.schema.json`.

## Versioning

- `schema_version` is a single integer for the manifest.
- **Additive changes** (new optional fields) are allowed within a version.
- **Breaking changes** (removed/renamed fields, changed meanings) bump `schema_version`.
- Current manifest version: **3**.

## Required Fields (v3)

The following fields are required in `manifest.json`:

- `schema_version` (const `3`)
- `generated_at_ms`
- `tool` (`name`, `version`)
- `mode` (const `handoff`)
- `inputs` (paths scanned)
- `output_dir` (directory written)
- `budget_tokens`, `used_tokens`, `utilization_pct`
- `strategy`, `rank_by`
- `capabilities`
- `artifacts`
- `included_files`
- `excluded_paths`, `excluded_patterns`
- `total_files`, `bundled_files`
- `intelligence_preset`

## Excluded Path Reason Codes

`excluded_paths[].reason` uses stable reason codes for deterministic filtering:

- `output_dir` — the handoff output directory itself

## Artifacts

Artifacts listed in `manifest.json`:

- `manifest.json` (self)
- `map.jsonl`
- `intelligence.json`
- `code.txt`

Artifacts include size and optional hash. Hashing uses **blake3**.

## Related Docs

- `docs/handoff.md` — user-facing overview and CLI usage
- `docs/tokmd-in-cockpit.md` — cockpit integration (separate from handoff)
