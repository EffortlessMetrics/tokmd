# Progress Events

`tokmd` keeps normal command output on stdout. Progress output is stderr-only.

When `TOKMD_PROGRESS_EVENTS` is set, the CLI progress helper emits one
newline-delimited JSON object per progress event to stderr. For machine
readers, combine it with `--no-progress` to suppress spinner output while
keeping explicit events enabled.

## Grammar

Each event is a single JSON object:

```json
{"event":"tokmd.progress","schema_version":1,"kind":"update","message":"Scanning codebase..."}
```

Fields:

- `event`: always `tokmd.progress`.
- `schema_version`: progress event grammar version, currently `1`.
- `kind`: event phase. Current values are `update` and `finish`.
- `message`: human-readable progress message from the active command.

Events are informational. They do not change command stdout, receipt schemas, or
exit-code policy.

## Example

```console
$ TOKMD_PROGRESS_EVENTS=1 tokmd run --no-progress --output-dir target/tokmd
{"event":"tokmd.progress","schema_version":1,"kind":"update","message":"Scanning codebase..."}
{"event":"tokmd.progress","schema_version":1,"kind":"finish","message":"done"}
```
