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

## Local Review Packet Quickstart

Use this flow when reviewing a branch locally in the tokmd checkout:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review

cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

Open the packet in this order:

1. `.tokmd/review/comment.md` for the short review summary.
2. `.tokmd/review/review-map.md` for what to inspect first and how to
   reproduce evidence claims.
3. `.tokmd/review/evidence.json` for the exact available, missing, stale,
   degraded, skipped, or unavailable evidence state.
4. `.tokmd/review/manifest.json` for packet-local artifact paths and hashes.
5. `target/tokmd/review-packet-check.json` for the verifier receipt.

When the PR changes source-of-truth docs, plans, ADRs, templates,
`.jules/goals/**`, or doc-artifact policy, generate the documentation-control
receipt first and import it:

```bash
cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json

tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --doc-artifacts-check target/docs/doc-artifacts-check.json \
  --review-packet-dir .tokmd/review

cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

The imported doc-artifacts receipt is review evidence only. It shows whether
the source-of-truth artifact shape, links, active goal state, and policy routing
checked by `cargo xtask doc-artifacts --check` were valid. It does not prove
the prose is correct, decide whether to merge, promote proof gates, or enable
Codecov uploads.

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
