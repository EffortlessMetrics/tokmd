# Spec: Packet Preset Renderer (`tokmd render`)

- Status: active
- Schema family: `tokmd.packets/v1` (`crates/tokmd/schemas/tokmd-packets.schema.json`)
- Related ADRs: n/a
- Related proof scopes: `tokmd_cli`, `project_truth_docs`
- Tracking: issue #222; producer rail `unsafe-review-swarm/docs/dogfood/tokmd-bun-packet-presets.md`

## Contract

`tokmd render --from-packets` consumes a cross-tool packet bundle directory and
renders audience-specific Markdown for Bun UB manual-candidate workflows.

unsafe-review (and future sibling tools) **produce** `tokmd-packets.json` and
related bundle files. tokmd **owns** the input schema and rendering contract.
Producers must validate exports against the published schema; tokmd rejects
unknown schema ids and unknown preset names.

This command is a formatting surface only. It does not run witnesses, execute
Miri, edit source, or post comments.

### Supported presets (phase 1)

| Preset | Audience |
| --- | --- |
| `bun-ub-handoff` | Rust lane implementer |
| `bun-ub-pr-body` | Upstream maintainer |
| `bun-ub-ledger-note` | Bun burndown ledger maintainer |
| `bun-ub-review-map` | Reviewer prioritization |
| `bun-ub-next-pick` | Lane coordinator |

Preset section requirements follow the unsafe-review dogfood rail. Phase 1
renders `preset_inputs[preset].sections` when present and records limitations
for absent inputs — it does not yet enforce every required section name.

## Inputs

| Input | Owner | Used for |
| --- | --- | --- |
| `--from-packets DIR` | Operator | Bundle root containing `tokmd-packets.json` |
| `--preset NAME` | Operator | Audience preset to render |
| `--output PATH` | Operator | Optional Markdown output file |
| `tokmd-packets.json` | Producer (e.g. unsafe-review) | Manifest with `preset_inputs`, `non_claims`, input presence |
| Sibling bundle files | Producer | Indexed by manifest `inputs_present` / `inputs_absent`; ingested when listed present |

Supported sibling files in phase 2:

| File | Schema id | Role |
| --- | --- | --- |
| `manual-candidates.json` | `manual-candidates/v1` | Manual candidate index; supplements absent or empty manifest `preset_inputs` |
| `cards.json` | producer-defined (`0.2` in fixtures) | ReviewCard snapshot; supplements `bun-ub-review-map` when manifest sections are absent |

The manifest schema id must be exactly `tokmd.packets/v1`.

When manifest `preset_inputs[preset].sections` is non-empty, it takes precedence
over sibling-derived sections. Sibling ingestion is partial and always records
limitations for missing producer sections.

## Outputs

Markdown document with:

1. Preset title and id
2. Rendered `sections` for the requested preset (when present)
3. Explicit `## Limitations` when `preset_inputs`, sections, or required bundle
   inputs are absent — never an empty or all-clear document
4. `## Bundle source inputs` when sibling files contributed or supplementation was attempted
5. `## Bundle inputs absent` listing manifest `inputs_absent`
6. `## Non-claims` reproducing manifest `non_claims` verbatim

Stdout is used when `--output` is omitted.

## Claim boundary

This renderer proves:

- the bundle manifest parsed and matched `tokmd.packets/v1`;
- the requested preset name is known;
- formatted output carries producer limitations and non-claims through;
- sibling files listed in `inputs_present` were read when present on disk.

It does **not** prove:

- undefined-behavior presence or absence;
- memory safety;
- Miri-clean or UB-free status;
- merge readiness or policy promotion;
- that absent bundle files would not change conclusions if present;
- that sibling-derived sections are complete versus producer `preset_inputs`.

## Compatibility

Schema id follows the namespaced string convention in
`docs/adr/0014-schema-identity-idioms.md` (for example
`tokmd.evidence-packet/v1`, `tokmd.packets/v1`).

Producer migration note: unsafe-review's legacy `tokmd-packets/v1` export (with a
`packets[]` array and `schema_version` field) is a producer-side shape. tokmd's
owned consumer contract is `tokmd.packets/v1` in `tokmd-packets.json` with
top-level `preset_inputs`. Producers should validate against
`crates/tokmd/schemas/tokmd-packets.schema.json` before handoff. See
`docs/interop/sibling-tools.md` for cross-repo tracking.

## Proof Requirements

```bash
cargo test -p tokmd-types tokmd_packets
cargo test -p tokmd-format tokmd_packets
cargo test -p tokmd render_packets
```

Fixture bundles:

- `fixtures/tokmd-packets/minimal/` — manifest `preset_inputs` populated
- `fixtures/tokmd-packets/sibling-derived/` — empty manifest `preset_inputs`, sibling files present

## Phase 2 (partial)

- JSON Schema validation at CLI boundary (`jsonschema` crate) — implemented in
  `tokmd render --from-packets` manifest load path
- Sibling bundle ingestion for `manual-candidates.json` and `cards.json` when
  manifest `preset_inputs` is absent or has empty `sections`

## Deferred (phase 2+)

- Additional sibling files (`comment-plan.json`, `witness-plan.md`, repair queues)
- unsafe-review export migration off de-facto schema ownership (producer repo)
- ripr repair-packet preset family (shape-adjacent per #222 cross-links)
- Schema ownership migration on unsafe-review export path (#222 comment)
