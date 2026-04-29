# FFI Contract Specification (`tokmd-core`)

## Status
Accepted

## Entry Point
`ffi::run_json(mode, args_json) -> String`

- `mode` identifies workflow: `lang`, `module`, `export`, `analyze`, `diff`, `version`.
- `args_json` MUST be valid JSON object payload for the selected mode.

## Response Envelope
All responses are serialized JSON objects:

```json
{
  "ok": true,
  "data": {"...": "mode-specific payload"},
  "error": null
}
```

or on failure:

```json
{
  "ok": false,
  "data": null,
  "error": {
    "code": "invalid_args",
    "message": "human-readable summary"
  }
}
```

## Behavioral Requirements

- The top-level envelope shape is stable across modes.
- On success, `ok=true` and `error` MUST be `null`.
- On failure, `ok=false` and `data` MUST be `null`.
- Error messages SHOULD be human-readable; `code` SHOULD be machine-stable.
- Versioned payloads MUST include the matching schema version family where applicable.

## Compatibility Requirements

- Additive fields in `data` are allowed in minor releases when existing fields are preserved.
- Renaming/removing fields in existing payloads requires a schema version bump and migration note in `CHANGELOG.md`.

## Binding Notes

- Python bindings return native dictionaries derived from this envelope.
- Node bindings return Promise-resolved objects derived from this envelope.
