# ADR-0007: Independent schema-family versioning

- Status: accepted
- Date: 2026-05-06

## Context

tokmd emits multiple receipt families: core inventory receipts, analysis
receipts, cockpit receipts, handoff manifests, context receipts, context bundle
manifests, sensor reports, and baselines. A single global schema version would
force unrelated consumers to react to changes in surfaces they do not use.

## Decision

Receipt families version independently. A breaking structure or semantic change
increments the schema version for the affected family only. Additive optional
fields may remain within the current family version when existing consumers can
ignore them safely.

## Consequences

- Consumers can pin the receipt family they actually use.
- Active surfaces can evolve without forcing unrelated migrations.
- Documentation and checks must track multiple schema constants instead of one
  workspace-wide number.

## Alternatives

- Use one global schema version for all tokmd JSON outputs.
- Avoid version bumps and rely on tool version alone.

Both alternatives were rejected because they either over-notify consumers or
make compatibility harder to reason about.

## Enforcement

- Keep `docs/SCHEMA.md`, formal schema files, and source constants in sync.
- Add docs checks for new schema constants or family-specific schema files.
- Bump the affected family version when changing a required field, field type,
  field meaning, or envelope shape.

## Related specs

- `docs/specification.md`
- `docs/SCHEMA.md`
- `docs/schema.json`
- `docs/handoff.schema.json`
