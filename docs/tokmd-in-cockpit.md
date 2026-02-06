# tokmd in Cockpit

`tokmd` provides the **Change Surface** sensor in a multi-sensor cockpit.

## Canonical Artifacts

When using `tokmd cockpit --artifacts-dir`, tokmd writes:

```
artifacts/tokmd/
├── report.json   # full cockpit receipt (JSON)
└── comment.md    # compact summary (3–8 bullets)
```

These paths are the stable integration contract for cockpit directors.

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
