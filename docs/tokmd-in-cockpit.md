# tokmd in Cockpit

`tokmd` provides the **Change Surface** sensor in a multi-sensor cockpit.

## Canonical Artifacts

When using `tokmd cockpit --artifacts-dir`, tokmd writes:

```
artifacts/tokmd/
├── cockpit.json  # raw cockpit receipt (JSON)
├── report.json   # full cockpit receipt (JSON)
└── comment.md    # compact summary (3–8 bullets)
```

These paths are the stable integration contract for cockpit directors.

For packet-shaped PR-review artifacts, use
`tokmd cockpit --review-packet-dir .tokmd/review`. The review-packet contract is
documented separately in [`review-packet.md`](review-packet.md). It is an
additive PR review artifact shape, not a replacement for the shipped
`--artifacts-dir` contract. The packet includes `review-map.json` and
`review-map.md` derived from the cockpit review plan.

## Default Policy

tokmd is **informational by default**. A repo may choose to gate on tokmd output,
but cockpit displays should treat it as a non-blocking sensor unless configured.

Example director policy:

```toml
[sensors.tokmd]
blocking = false
missing = "warn"
highlights = 5
```

## Comment Budget

`comment.md` is intentionally short:

- 3–8 bullets
- diff stats
- risk score/level
- top hotspots
- top review plan items

The full receipt lives in `report.json`.
