# Spec: Machine-Readable Progress Events

- Status: active
- Schema family, if any: `tokmd.progress.v1`
- Related ADRs: n/a
- Related proof scopes: `tokmd_cli`, `project_truth_docs`

## Contract

When `TOKMD_PROGRESS_EVENTS` is set in the environment, the `tokmd` CLI progress
helper emits one newline-delimited JSON object per progress event to **stderr**.

Progress events are informational. They must not change command stdout, receipt
schemas, exit-code policy, or required CI behavior.

Command receipts and human-readable command output remain on **stdout**. Spinner
and progress-event output remain on **stderr**.

Machine readers that want JSON progress without spinner glyphs should combine
`TOKMD_PROGRESS_EVENTS` with `--no-progress`.

## Inputs

| Input | Owner | Effect |
| --- | --- | --- |
| `TOKMD_PROGRESS_EVENTS` | Process environment | When present (any value), progress events are emitted to stderr. When absent, no progress events are emitted. |
| `TOKMD_NO_PROGRESS` | Process environment | Disables interactive spinner output when stderr is a TTY. Does **not** disable machine-readable progress events when `TOKMD_PROGRESS_EVENTS` is set. |
| `NO_COLOR` | Process environment | Disables interactive spinner output when stderr is a TTY. Does **not** disable machine-readable progress events when `TOKMD_PROGRESS_EVENTS` is set. |
| `--no-progress` | CLI flag | Disables interactive spinner output. Does **not** disable machine-readable progress events when `TOKMD_PROGRESS_EVENTS` is set. |
| Non-TTY stderr | Runtime | Disables interactive spinner output. Does **not** disable machine-readable progress events when `TOKMD_PROGRESS_EVENTS` is set. |

Activation rules:

- Progress events are opt-in through `TOKMD_PROGRESS_EVENTS` only.
- Spinner suppression controls (`TOKMD_NO_PROGRESS`, `NO_COLOR`, `--no-progress`,
  non-TTY stderr) affect indicatif output only.
- Progress events may be emitted on builds with or without the `ui` feature.

## Outputs

Each emitted line is a single JSON object with this shape:

```json
{"event":"tokmd.progress","schema_version":1,"kind":"update","message":"Scanning codebase..."}
```

Required fields:

| Field | Type | Value |
| --- | --- | --- |
| `event` | string | Always `tokmd.progress`. |
| `schema_version` | integer | Progress-event grammar version. Current value is `1`. |
| `kind` | string | Event phase. Current allowed values: `update`, `finish`. |
| `message` | string | Human-readable progress text from the active command. |

Kind semantics:

- `update`: progress text changed while work is still running.
- `finish`: the active progress indicator completed. The message is the final
  status text supplied by the caller; the default completion message is `done`.

Serialization rules:

- One JSON object per physical line on stderr.
- Embedded control characters in `message` must be JSON-escaped; emitted lines
  must not contain raw newline characters.
- No trailing comma, prefix, suffix, or wrapper envelope.
- Object key order is not part of the contract; consumers must compare parsed
  JSON semantically.

Golden fixtures live under `fixtures/progress-events/` and are validated by
`cargo test -p tokmd progress_event_fixtures_match_emitted_json`.

## Compatibility

`schema_version` is the progress-event grammar version, independent of tokmd
receipt schema versions (`SCHEMA_VERSION`, analysis/cockpit/handoff families).

When `schema_version` increases:

- existing consumers must treat unknown versions as forward-incompatible with
  their parser unless they implement explicit fallback;
- new fields may be added only in a new `schema_version`;
- existing field names, types, and allowed `kind` values for prior versions must
  remain stable;
- fixture files and tests for the prior version must remain until that version
  is retired.

This spec does not define browser worker progress messages. Those use worker
protocol v2 (`type: "progress"`) and remain documented in
`docs/progress-events.md`.

User-facing examples and browser worker notes remain in `docs/progress-events.md`.
This file is the durable behavior contract.

## Proof Requirements

```bash
cargo test -p tokmd progress --lib --verbose
cargo xtask docs --check
cargo xtask doc-artifacts --check
git diff --check
```

Optional smoke for Action authors:

```bash
TOKMD_PROGRESS_EVENTS=1 tokmd run --no-progress --output-dir target/tokmd-progress-smoke crates/tokmd
```

## Open Questions

- Whether a formal JSON Schema document should be published for third-party
  validators.
- Whether additional `kind` values (for example `start`) should be added in
  `schema_version` 2.
