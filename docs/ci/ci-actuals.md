# CI Actuals Receipt

`cargo xtask ci-actuals` emits `tokmd.ci_actuals.v1`, a small receipt for
GitHub Actions job results and optional measured durations.

The command is intentionally receipt-only. It does not decide budget policy,
change required gates, or infer learned estimates. Later CI-economics work can
consume the receipt once enough observations exist.

## Inputs

```bash
cargo xtask ci-actuals \
  --needs target/ci/needs.json \
  --timings target/ci/timings.json \
  --output target/ci/ci-actuals.json
```

- `--needs` reads the literal `${{ toJson(needs) }}` payload from an aggregate
  GitHub Actions job.
- `--timings` is optional. It may contain either `{ "job-id": 12.5 }` or
  `{ "job-id": { "duration_seconds": 12.5, "runner": "ubuntu-latest" } }`.
- `--output` writes the receipt path, creating parent directories as needed.

When timing data is absent, the receipt records `timing_status: "missing"` and
leaves duration fields `null`. Missing timing is not coerced to zero.

## Output

```json
{
  "schema": "tokmd.ci_actuals.v1",
  "schema_version": 1,
  "repo": "tokmd",
  "workflow": "CI",
  "sha": "<commit>",
  "jobs": [
    {
      "name": "docs-check",
      "result": "success",
      "output_keys": [],
      "runner": "ubuntu-latest",
      "duration_seconds": 75.0,
      "duration_minutes": 1.25,
      "timing_status": "measured",
      "cache_hit": true
    }
  ],
  "status": {
    "ok": true,
    "job_count": 1,
    "timed_job_count": 1,
    "missing_timing": [],
    "unused_timing": []
  }
}
```

The receipt is sorted by job name so downstream summaries remain stable.
