# GitHub Action Quickstart

Use the root `EffortlessMetrics/tokmd` Action when you want CI to install a
released `tokmd` binary, produce artifacts, and optionally post a pull request
comment.

This page is the adoption path. For every input, output, and mode, use the
[GitHub Action reference](github-action.md).

For the planned `sensors/tokmd/` evidence packet Action path, see
[PR evidence packet workflows](packet-workflows.md). This page documents the
currently implemented root Action modes.

## Minimal Receipt Workflow

Use this when you want a repository summary and machine-readable receipt
without introducing review comments or gates:

```yaml
name: tokmd receipt

on:
  pull_request:

permissions:
  contents: read

jobs:
  tokmd:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: EffortlessMetrics/tokmd@v1
        with:
          version: '1.11.0'
          paths: .
          artifact: 'true'
          comment: 'false'
```

Open first: the `tokmd-receipts` artifact, then `tokmd-summary.md`.

This produces a summary and receipt for the checked-out tree. It does not review
a PR, execute repo tests, promote advisory proof, upload Codecov by default, or
make a merge verdict.

## PR Review Packet Workflow

Use this when reviewers should start from a cockpit review packet:

```yaml
name: tokmd review packet

on:
  pull_request:

permissions:
  contents: read
  pull-requests: write

jobs:
  tokmd:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0

      - uses: EffortlessMetrics/tokmd@v1
        with:
          version: '1.11.0'
          mode: cockpit
          head: HEAD
          review-packet: 'true'
          artifact: 'true'
          comment: 'false'
```

Open first:

1. the `tokmd-receipts` artifact;
2. `.tokmd/review/review-map.md`;
3. `.tokmd/review/comment.md`;
4. `.tokmd/review/evidence.json`;
5. `target/tokmd/review-packet-check.json`.

With `review-packet: 'true'`, the Action runs `tokmd cockpit` with
`--review-packet-dir .tokmd/review`, verifies the packet, writes
`target/tokmd/review-packet-check.json`, and uploads the packet and verifier
receipt when `artifact: 'true'`.

Set `comment: 'true'` only when you want the Action to post the packet summary
back to the pull request. The posted comment is still review evidence, not a
merge verdict.

## Base And Head

For pull request events, the Action can infer the base ref. Use
`fetch-depth: 0` for cockpit review packets so the base and head commits are
available.

Set `base` only when you need an explicit compare ref:

```yaml
      - uses: EffortlessMetrics/tokmd@v1
        with:
          version: '1.11.0'
          mode: cockpit
          base: origin/main
          head: HEAD
          review-packet: 'true'
          artifact: 'true'
          comment: 'false'
```

## What The Action Proves

The Action can prove:

- a released `tokmd` binary was installed or already available;
- the selected `tokmd` mode produced its expected files;
- requested artifacts were uploaded;
- review packets were verified when `mode: cockpit` and
  `review-packet: 'true'` are used.

It does not prove:

- repository tests passed;
- affected proof executed;
- advisory proof became required;
- scoped coverage, mutation, or Codecov upload is a gate;
- a PR is safe to merge;
- release or publishing mutation is approved.

## Next Actions

- For local first-run usage, use [Install and try tokmd](install-and-try.md).
- For every Action input and output, use [GitHub Action reference](github-action.md).
- For review-packet artifact meanings, use [Review packet contract](review-packet.md).
- For proof planning and evidence reading order, use [Copy-Ready Workflows](workflows.md).
