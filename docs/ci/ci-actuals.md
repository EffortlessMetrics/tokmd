# ci-actuals.json

`cargo xtask ci-actuals` emits the rolling per-job timing artifact that
PR 14's budget guard and PR 15's learned estimates feed off.

## When it runs

The aggregator job in `.github/workflows/ci.yml` writes the literal
`${{ toJson(needs) }}` payload to `target/ci/needs.json`, optionally
appends per-job durations from a sidecar JSON (`target/ci/timings.json`),
and then runs:

```bash
cargo xtask ci-actuals \
  --needs target/ci/needs.json \
  --timings target/ci/timings.json \
  --json-out target/ci/ci-actuals.json
```

## Output schema

```json
{
  "schema_version": 1,
  "repo": "tokmd",
  "sha": "<HEAD>",
  "workflow": "CI",
  "jobs": [
    {
      "name": "quality_gate",
      "runner": "ubuntu_latest",
      "estimated_lem": 8,
      "actual_seconds": 320.0,
      "actual_lem": 5.33,
      "conclusion": "success",
      "cache_hit": null,
      "risk_packs": []
    }
  ]
}
```

`actual_lem = (actual_seconds / 60) × runner_multiplier`.

## Inputs

- `--needs <PATH>` — JSON file containing the workflow's `needs`
  context. The aggregator step writes this directly via
  `echo "$NEEDS_JSON" > target/ci/needs.json`.
- `--timings <PATH>` (optional) — `{ "<job-id>": <seconds> }`.
- `--lanes <PATH>` (optional, defaults to
  `policy/ci-lane-whitelist.toml`) — used to look up per-job static
  `estimated_lem` and the runner multiplier.

## Status

- **PR 13** (this PR) — adds the command and the artifact, plus
  `.config/nextest.toml` so a future migration to nextest fits the
  same pipeline. The artifact is **purely advisory**: nothing reads
  it yet.
- **PR 14** — the soft budget guard reads `estimated_lem` and warns
  when totals exceed the elevated band; uses actuals when present.
- **PR 15** — replaces static `base_lem` with learned p50/p90/p95.

The artifact starts empty (no actuals collected yet). It exists as a
schema-stable receipt so the pipeline can land before the first
calibration window.
