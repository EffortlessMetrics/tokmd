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
| Sibling bundle files | Producer | Indexed by manifest `inputs_present` / `inputs_absent`; not read directly in phase 1 |

The manifest schema id must be exactly `tokmd.packets/v1`.

## Outputs

Markdown document with:

1. Preset title and id
2. Rendered `sections` for the requested preset (when present)
3. Explicit `## Limitations` when `preset_inputs`, sections, or required bundle
   inputs are absent — never an empty or all-clear document
4. `## Bundle inputs absent` listing manifest `inputs_absent`
5. `## Non-claims` reproducing manifest `non_claims` verbatim

Stdout is used when `--output` is omitted.

## Claim boundary

This renderer proves:

- the bundle manifest parsed and matched `tokmd.packets/v1`;
- the requested preset name is known;
- formatted output carries producer limitations and non-claims through.

It does **not** prove:

- undefined-behavior presence or absence;
- memory safety;
- Miri-clean or UB-free status;
- merge readiness or policy promotion;
- that absent bundle files would not change conclusions if present.

## Compatibility

Schema id follows the namespaced string convention in
`docs/adr/0014-schema-identity-idioms.md` (for example
`tokmd.evidence-packet/v1`, `tokmd.packets/v1`).

Phase 1 intentionally does not consume raw `manual-candidates.json` or
`cards.json` directly; producers must populate `preset_inputs`.

## Proof Requirements

```bash
cargo test -p tokmd-types tokmd_packets
cargo test -p tokmd-format tokmd_packets
cargo test -p tokmd render_packets
```

Fixture bundle: `fixtures/tokmd-packets/minimal/`.

## Phase 2 (partial)

- JSON Schema validation at CLI boundary (`jsonschema` crate) — implemented in
  `tokmd render --from-packets` manifest load path

## Deferred (phase 2+)

- Direct consumption of sibling bundle files when `preset_inputs` is partial
- unsafe-review export migration off de-facto schema ownership (producer repo)
- ripr repair-packet preset family (shape-adjacent per #222 cross-links)
- Schema ownership migration on unsafe-review export path (#222 comment)
