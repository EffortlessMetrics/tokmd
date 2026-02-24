# Handoff Schema (manifest.json)

The handoff bundle is a directory of artifacts intended for LLM review and automation.  
The authoritative index is `manifest.json`, validated by `docs/handoff.schema.json`.

## Versioning

- `schema_version` is a single integer for the manifest.
- **Additive changes** (new optional fields) are allowed within a version.
- **Breaking changes** (removed/renamed fields, changed meanings) bump `schema_version`.
- Current manifest version: **5**.

## Required Fields (v5)

The following fields are required in `manifest.json`:

- `schema_version` (const `5`)
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
- `smart_excluded_files`
- `total_files`, `bundled_files`
- `intelligence_preset`

## Optional Fields (v5)

- `rank_by_effective` — effective ranking metric if fallback occurred
- `fallback_reason` — reason for fallback if rank_by_effective differs from rank_by
- `excluded_by_policy` — files excluded by per-file cap / classification policy
- `token_estimation` — token estimation envelope with uncertainty bounds
- `code_audit` — post-bundle audit comparing actual code bundle bytes to estimates

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
