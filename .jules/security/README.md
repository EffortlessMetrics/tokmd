# Security (Sentinel)

This directory tracks security improvements, audits, and hardening efforts for `tokmd`.

## Structure

- `ledger.json`: Append-only log of all security runs.
- `runs/`: Detailed logs for each run (one file per run).
- `envelopes/`: Machine-readable receipts for each run.
- `notes/`: ZK-style notes on reusable security patterns.

## Protocol

1. **Selection**: Random or Priority from `friction/open/` (Lane A) or Scout Discovery (Lane B).
2. **Execution**: Options A/B -> Decision -> Implementation -> Verification.
3. **Glass Cockpit**: All PRs must follow the PR Glass Cockpit template.
