# tokmd-progress

Optional terminal progress helpers for long tokmd runs.

## Problem

Use this crate when you want a spinner or ETA bar in interactive terminals,
but nothing in CI, non-TTYs, or user sessions that disable progress.

## What it gives you

- `Progress`
- `ProgressBarWithEta`
- `ui` feature gate for `indicatif`
- `NO_COLOR` and `TOKMD_NO_PROGRESS` opt-outs
- no-op fallbacks when `ui` is disabled

## Quick use / integration notes

```toml
[dependencies]
tokmd-progress = { workspace = true, features = ["ui"] }
```

Without `ui`, the API stays callable but renders nothing.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Troubleshooting](../../docs/troubleshooting.md)
Reference: [Source](src/lib.rs)
Explanation: [Architecture](../../docs/architecture.md)
