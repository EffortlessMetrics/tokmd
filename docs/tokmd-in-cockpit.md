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

## Choosing Evidence Inputs

Start with the smallest packet that can answer the review question:

| PR shape | Add these inputs | What the packet can say |
| --- | --- | --- |
| Code-only change | `--review-packet-dir` | What changed, what to review first, and which cockpit evidence is available, missing, stale, degraded, skipped, or unavailable. |
| Source-of-truth docs, plans, ADRs, templates, `.jules/goals/**`, or doc-artifact policy changed | `--doc-artifacts-check target/docs/doc-artifacts-check.json` | Whether the documentation-control checker receipt is present and whether source-of-truth artifact shape, links, active goal state, and policy routing passed. |
| Required proof was planned or run | `--proof-run-summary`, `--proof-observation` | Which required proof applied to the reviewed change, whether it passed, and whether imported proof freshness matches the cockpit head. |
| Advisory proof or coverage exists | `--executor-observation`, `--coverage-receipt` | Which advisory evidence exists, which evidence is missing, and which evidence must not be treated as a required gate. |

Do not add evidence inputs just to make the packet look complete. A missing or
unavailable evidence bucket is better than a stale artifact that looks green.
`review-map.md` and `review-map.json` carry reproduction commands for evidence
classes that can be regenerated locally.

## Hosted Action Quickstart

Use the composite Action when reviewers need hosted comments and downloadable
artifacts:

```yaml
name: PR Cockpit
on:
  pull_request:

permissions:
  contents: read
  pull-requests: write

jobs:
  cockpit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0

      - uses: EffortlessMetrics/tokmd@v1
        with:
          version: '1.11.0'
          mode: cockpit
          base: origin/${{ github.base_ref }}
          head: HEAD
          review-packet: 'true'
          artifact: 'true'
          comment: 'true'
```

With `artifact: 'true'`, the `tokmd-receipts` artifact contains `.tokmd/review/`
and `target/tokmd/review-packet-check.json`. The hosted PR comment uses a copy
of `.tokmd/review/comment.md` with workflow-run and artifact links appended;
the packet-local `comment.md` is not mutated after manifest hashes are written.

## What Verification Means

`cargo xtask review-packet-check` verifies the packet contract:

- manifest artifact paths are packet-local;
- hosted comment copies are not listed in the manifest;
- `manifest.json`, `evidence.json`, and `review-map.json` match their schemas;
- BLAKE3 hashes match the final packet-local artifact contents;
- optional copied proof and docs evidence artifacts are hash-checked when the
  manifest lists them.

Verification does not mean the PR is mergeable, the prose is correct, missing
evidence is acceptable, proof gates were promoted, or Codecov uploads are
enabled. It means the packet is internally consistent enough to review from.

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
