# Routed Rust Small runner health runbook

Status: operating guide for the self-hosted Rust Small pool.

This runbook is for maintainers and agents diagnosing the routed Rust Small
frontdoor. The policy contract lives in `docs/ci/routed-ci-policy.md`; this
page describes what to inspect and how to keep a sick runner out of the route
without changing the workflow logic.

## Scope

Rust Small should use self-hosted capacity only when the route helper can prove
that the event is trusted and the selected runner state is useful. When state is
missing, stale, degraded, quarantined, low on disk, low on scratch, or otherwise
unknown, route GitHub-hosted before any self-hosted implementation job queues.

The current workflow writes two routed artifacts:

1. `route-rust-small` with `target/ci/route-rust-small.json`
2. `routed-rust-small-result` with `target/ci/routed-rust-small-result.json`

Open `routed-rust-small-result.json` first. It records the selected
implementation job, the sibling job results, selected-job timing when GitHub
reported it, cache policy notes, and the router fields that drove the run. Open
`route-rust-small.json` next when the route reason or runner counts need more
detail. When the route consumed a runner health receipt, `route-rust-small.json`
also records the resolved `health`, `health_age_seconds`, `disk_free_bytes`,
`scratch_free_bytes`, and `min_free_bytes` fields that bounded the decision.

## Health receipt helper

`cargo xtask ci-runner-health` emits a checked health receipt using schema
`tokmd.ci_runner_health.v1`. The route helper can consume that receipt with
`cargo xtask ci-route --health-json <path>` when a lane wires a health artifact
into routing.

Example health receipt command:

```bash
cargo +1.95.0 xtask ci-runner-health \
  --json target/ci/runner-health.json \
  --runner-name "$RUNNER_NAME" \
  --label self-hosted \
  --label linux \
  --label x64 \
  --label em-ci \
  --label trusted-pr \
  --label rust-small \
  --disk-free-bytes "$DISK_FREE_BYTES" \
  --scratch-free-bytes "$SCRATCH_FREE_BYTES" \
  --min-free-bytes 8589934592
```

The helper records:

- runner name;
- labels;
- timestamp;
- status: `healthy`, `degraded`, or `quarantined`;
- reason;
- disk and scratch free bytes when supplied;
- Rust, `git`, and optional Docker availability;
- warnings and errors.

Do not put secrets, absolute host paths, or user-specific scratch paths in the
receipt. The helper rejects secret-looking and absolute-path strings.

## Quarantine and drain

Use quarantine when a runner should not receive Rust Small work even if GitHub
reports it online and idle.

For a health-integrated route, publish a health receipt like this:

```bash
cargo +1.95.0 xtask ci-runner-health \
  --json target/ci/runner-health.json \
  --runner-name "$RUNNER_NAME" \
  --label self-hosted \
  --label linux \
  --label x64 \
  --label em-ci \
  --label trusted-pr \
  --label rust-small \
  --status quarantined \
  --reason manual_quarantine
```

When the active workflow does not have a fresh health receipt wired into the
route, drain the runner outside the workflow instead: take it offline, remove it
from the eligible pool, or dispatch the routed workflow with
`force-github-hosted` for diagnostic work. Do not edit branch protection or add
runner-specific required checks to work around a sick machine.

Unquarantine only after the next health receipt is fresh, `healthy`, and above
the disk and scratch guards.

## Disk, scratch, and cache cleanup

The self-hosted implementation uses run-scoped scratch paths:

```text
/mnt/ci-scratch/tmp/<run-id>-<attempt>
/mnt/ci-scratch/target/<run-id>-<attempt>
/mnt/ci-scratch/cargo-home/<run-id>-<attempt>
```

Clean run-scoped `tmp`, `target`, and `cargo-home` directories first. The
routed Rust Small workflow does not depend on a shared Cargo home; a previous
shared-cache attempt made selected self-hosted jobs vulnerable to cross-run
cache ownership drift. If a future lane reintroduces a shared Cargo cache,
preserve that cache unless the runner is already degraded for cache pressure or
a maintainer explicitly assigns cache cleanup.

If scratch cleanup is needed to make the runner usable, mark the runner
degraded or quarantined before cleanup work starts so new PR runs fall back to
GitHub-hosted.

The workflow preflight guards currently check:

```text
ci-disk-guard /mnt/ci-scratch 45
```

Treat those failures as runner health failures, not code failures.

## Fallback proof

Use workflow dispatch proof modes to verify routing behavior without waiting for
real fleet pressure:

| Mode | Expected route | Use |
| --- | --- | --- |
| `force-github-hosted` | GitHub-hosted | Confirm hosted execution and avoid self-hosted capacity. |
| `simulate-full` | GitHub-hosted | Prove full self-hosted capacity falls back before dispatch. |
| `simulate-unhealthy` | GitHub-hosted | Prove degraded health falls back before dispatch. |
| `simulate-api-unavailable` | GitHub-hosted | Prove missing runner API state falls back safely. |
| `simulate-untrusted` | GitHub-hosted | Prove untrusted state cannot select self-hosted. |
| `force-self-hosted` | Self-hosted if trusted | Diagnostic only; do not use on unsafe events. |

Every proof mode should still finish through `Tokmd Rust Result` on the runner
the advisory `Route CI runner` job selects.

## Agent boundaries

Agents may:

- inspect route and result receipts;
- request a hosted diagnostic run;
- cite route reasons in PR comments or release evidence;
- clean run-scoped artifacts created by the current lane.

Agents must not:

- modify runner labels, runner groups, or branch protection unless assigned;
- mark partial route evidence as a runner failure;
- remove shared Cargo caches without a maintainer-owned cleanup instruction;
- rerun self-hosted jobs repeatedly when the route already explains capacity or
  health fallback.

The stable PR contract remains `Tokmd Rust Result`. Runner selection by the
advisory `Route CI runner` job is an implementation detail and must not become a
required check.
