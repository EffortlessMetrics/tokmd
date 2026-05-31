# Learned LEM estimates

`cargo xtask ci-plan` can consume past `ci-actuals.json` artifacts when
`--actuals-dir <DIR>` is provided. The planner walks the directory, collects
`duration_seconds` per job id, and computes:

```text
estimate     = max(static_floor, p50_recent_actual × 1.15)
warning      = p90_recent_actual         (reported alongside)
hard ceiling = p95_recent_actual         (reported alongside)
```

The static floor in `policy/ci-lane-whitelist.toml :: base_lem` is the fallback
when no actuals exist for a lane. This guarantees a brand-new lane never
reports `0 LEM` because no calibration window has elapsed.

The hosted PR Plan workflow currently uses static estimates because it does not
provide `--actuals-dir`. Learned estimates are available to local or future
hosted runs once a durable actuals cache is passed to the planner.

## Output

Each lane in `lanes_selected` now has:

```json
{
  "id": "build_test_linux",
  "estimated_lem": 12,
  "estimate_source": "learned-p50",
  "learned_p50_lem": 10.5,
  "learned_p90_lem": 14.0,
  "learned_p95_lem": 16.0,
  ...
}
```

`estimate_source` is `static` until at least one valid sample exists
for the lane; `learned-p50` once one or more samples are present. The
percentile fields are omitted when the estimate is static.

## Storage

`ci-actuals.json` can be emitted by `cargo xtask ci-actuals`. For durable
history across runs, copy artifacts into a long-lived store (for example, an
object bucket or an ad-hoc nightly that aggregates recent runs) and pass that
local cache as `--actuals-dir`.

The planner also accepts legacy cached artifacts that used `actual_seconds`,
but the Rust-owned `ci-actuals` receipt writes `duration_seconds`.

The first calibration window is intentionally small: a handful of
runs is enough to start beating the static floor on lanes that vary
significantly with cache state.

## Outliers

The current model uses simple sorted-rank percentiles. A failed run
contributes `duration_seconds == 0`, which `load_actuals` filters out, so
failures don't drag percentiles down. A pathological cold-cache run
contributes a high p95 — that's intentional; reviewers should see the
real worst-case.

A future slice may layer in median-absolute-deviation or quantile-cap trimming
if outlier behavior becomes a problem in practice.
