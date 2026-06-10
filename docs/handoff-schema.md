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
- `work-order.md`
- `map.jsonl`
- `intelligence.json`
- `code.txt`
- `review-links.json` when `tokmd handoff` is given `--review-packet-dir` or
  `--review-packet-check`
- `proof-links.json` when `tokmd handoff` is given `--proof-route`,
  `--affected`, or `--proof-plan`, or when `--review-packet-dir` contains
  `proof/proof-pack-route.json`
- `evidence-packet-links.json` when `tokmd handoff` is given
  `--evidence-packet`

Artifacts include size and optional hash. Hashing uses **blake3**.
`work-order.md` is an agent-readable consumption guide generated from the
manifest inputs, selected files, and optional review/proof/evidence-packet link
inputs. It may summarize readable linked receipts for triage, but it does not
verify external receipts and does not replace their source artifacts.
The link artifacts are packet-local JSON files that point at external review,
proof, or evidence-packet receipts. They do not copy those external receipts
into the handoff and do not replace the review-packet verifier or the linked
evidence packet manifest.
When the linked proof route comes from `--review-packet-dir`, the route remains
owned by the cockpit review packet and should be verified with the packet
manifest/checker before relying on it.

## Related Docs

- `docs/handoff.md` — user-facing overview and CLI usage
- `docs/tokmd-in-cockpit.md` — cockpit integration (separate from handoff)
