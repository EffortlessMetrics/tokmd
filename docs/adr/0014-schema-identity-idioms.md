# ADR-0014: Schema identity idioms (integer family versions vs namespaced string ids)

- Status: accepted
- Date: 2026-06-24
- Related specs:
  - `docs/SCHEMA.md`
  - `docs/specs/evidence-packet-workflow.md`
  - `docs/specs/syntax-receipts.md`
  - `docs/specs/tokmd-packets-render.md`
  - `docs/sensor-report-v1.md`
- Related ADRs:
  - `docs/adr/0007-schema-family-versioning.md`

## Context

tokmd emits two kinds of machine-readable JSON contracts:

1. **Receipt envelopes** — long-lived scan and analysis outputs with a
   `schema_version: u32` field and per-family integer constants such as
   `SCHEMA_VERSION`, `ANALYSIS_SCHEMA_VERSION`, and `HANDOFF_SCHEMA_VERSION`.
   ADR-0007 governs independent versioning across those families.

2. **Namespaced string schema ids** — self-describing contract identifiers on
   orchestration surfaces that index, bundle, or hand off existing receipts across
   tools. Examples already in the repo include `tokmd.evidence-packet/v1`,
   `tokmd.packets/v1`, `tokmd.syntax_receipt.v1`, `tokmd.syntax_receipts.v1`,
   `tokmd.ast_shadow.v1`, and the fleet-wide `sensor.report.v1`.

Issue #224 asked whether the string idiom should replace integers, whether
integers should remain universal, or whether a documented split is required before
more cross-tool consumers (cockpitctl handoff routing, unsafe-review packet
bundles, ripr downstream specs) proliferate `match schema` branches.

## Decision

**Keep both idioms deliberately.** They answer different consumer questions:

| Question | Idiom | JSON field | Example |
| --- | --- | --- | --- |
| Which tokmd receipt family and revision is this? | Integer family version | `schema_version` | `9` on an analysis receipt |
| Which cross-tool contract is this artifact? | Namespaced string id | `schema` | `tokmd.evidence-packet/v1` |

### Use integer `schema_version` when

- The artifact is a **tokmd receipt envelope** with `{ schema_version, tool, …, data }`
  shape (or a documented subset such as context or handoff manifests that already
  use the integer field).
- The surface is consumed primarily by tokmd-aware pipelines that already pin
  per-family integers (`lang`, `module`, `export`, `diff`, `run`, `analyze`,
  `cockpit`, `context`, handoff, baseline, tool definitions).
- Version bumps follow ADR-0007: increment only the affected family constant.

### Use a namespaced string `schema` id when

- The artifact is a **manifest, bundle index, workflow receipt, or orchestration
  envelope** meant for cross-tool discovery (evidence packets, render bundles,
  syntax receipt packets, xtask/CI proof receipts, fleet sensor reports).
- Consumers need a **greppable, self-describing** contract key before parsing
  payload fields.
- The artifact **indexes** integer-versioned receipts rather than replacing them
  (for example `manifest.json` pointing at `analyze.json` with
  `schema_version: 9`).

### String id format rules

- **New tokmd-owned cross-tool surfaces** use `tokmd.<family>/vN` (slash before
  the major version). Example: `tokmd.evidence-packet/v1`, `tokmd.packets/v1`.
- **Fleet or ecosystem contracts outside the tokmd namespace** use
  `<tool>.<family>.vN` (dot before the major version). Example:
  `sensor.report.v1`.
- **Existing dot-form tokmd ids remain stable** (`tokmd.syntax_receipt.v1`,
  `tokmd.syntax_receipts.v1`, `tokmd.ast_shadow.v1`). Do not rename shipped ids
  for cosmetic alignment.
- A breaking structure or semantic change increments the major version suffix
  (`/v2`, `.v2`) and ships a new constant; additive optional fields stay within
  the current major id when existing consumers can ignore them safely.

### Migration

- **No migration** of established integer receipt families to string ids. The
  compatibility and documentation cost outweighs the ergonomics gain for surfaces
  that already have stable `schema_version` consumers and `cargo xtask bump`
  support.
- String-id surfaces may reference integer-versioned receipts without re-encoding
  their versions. Downstream `match` code should branch on the top-level `schema`
  string first, then read nested `schema_version` only after selecting the
  indexed receipt file.
- Org-wide tools (unsafe-review, ripr, cockpitctl) should treat tokmd string ids
  as the handoff key for packet manifests and fleet ids (`sensor.report.v1`) as
  the sensor envelope key; they should continue to read tokmd receipt
  `schema_version` only inside the referenced artifact files.

## Consequences

- Consumers handle two top-level discriminators, but each applies to a distinct
  artifact class with a clear selection rule.
- `docs/SCHEMA.md` documents integer families; surface specs document string
  ids with JSON Schema `const` fields where formal schemas exist.
- New cross-tool work does not pressure-fit manifests into receipt envelopes or
  vice versa.
- Minor historical inconsistency (`/v1` vs `.v1` within tokmd-owned ids) is
  accepted; new surfaces standardize on `/vN`.

## Alternatives

- **String ids everywhere** — rejected. Would force a breaking migration across
  core receipts, bindings, golden tests, and `xtask bump` without improving
  cross-tool ergonomics for surfaces that are already integer-stable.
- **Integers everywhere** — rejected. Cross-tool manifests need self-describing
  keys that survive outside tokmd's receipt envelope and are easy to grep across
  repositories.
- **Defer the decision** — rejected. Multiple string-id surfaces already ship;
  downstream consumers are writing `match schema` code now.

## Enforcement

- Integer families: keep `SCHEMA_VERSION` constants, `docs/SCHEMA.md` version
  table, `crates/tokmd/tests/schema_sync.rs`, and `cargo xtask bump --schema`
  aligned (per ADR-0007).
- String families: define a `pub const …_SCHEMA: &str` in the owning crate,
  document the id in the surface spec's header (`Schema family, if any:`), and
  lock the value in JSON Schema with a `const` on `schema` when a formal schema
  file exists.
- ADR-0007 remains authoritative for **when** to bump integer family versions;
  this ADR is authoritative for **which idiom** a new surface uses.
- PRs introducing a new externally consumed JSON artifact must declare the idiom
  in the spec or ADR reference and must not introduce a third discriminator
  (for example `schema_id` alongside both fields on the same object).

## Related specs

- `docs/SCHEMA.md` — integer receipt families and the sensor envelope row
- `docs/specs/evidence-packet-workflow.md` — `tokmd.evidence-packet/v1`
- `docs/specs/syntax-receipts.md` — `tokmd.syntax_receipt.v1` /
  `tokmd.syntax_receipts.v1`
- `docs/specs/tokmd-packets-render.md` — `tokmd.packets/v1`
- `docs/sensor-report-v1.md` — `sensor.report.v1`
