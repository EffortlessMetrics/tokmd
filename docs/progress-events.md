# Progress Events

Normative contract: [`docs/specs/progress-events.md`](specs/progress-events.md).

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

## Browser Worker Progress

The browser runner uses worker protocol v2 progress messages while keeping final
`result` and `error` messages as the receipt-carrying contract. A progress
message has this shape:

```json
{"type":"progress","requestId":"run-1","phase":"fetch","mode":"lang","message":"Fetching in-memory inputs"}
```

Fields:

- `type`: always `progress`.
- `requestId`: the active worker run id.
- `phase`: one of `start`, `fetch`, `analyze`, `done`, or `error`.
- `mode`: the active browser-safe run mode when available.
- `message`: human-readable progress text.
- `current` and `total`: optional numeric counters.

Worker runs emit `start`, then `fetch`, then `done` for `lang`, `module`, and
`export`. Analyze runs emit `start`, `fetch`, `analyze`, then `done`. Failed
runs emit `error` after the last phase reached. These events are informational
and do not change the final result payload or native receipt schemas.

## Example

```console
$ TOKMD_PROGRESS_EVENTS=1 tokmd run --no-progress --output-dir target/tokmd
{"event":"tokmd.progress","schema_version":1,"kind":"update","message":"Scanning codebase..."}
{"event":"tokmd.progress","schema_version":1,"kind":"finish","message":"done"}
```
