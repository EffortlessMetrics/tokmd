# Routed CI policy

Status: **Retired (phase 3, #299)** — historical routing contract.

The dedicated routed Rust Small frontdoor workflow (`em-routed-rust-small.yml`)
was removed in phase 3 of the single-tight-gate migration (#226). Runner
routing now lives in the advisory `Route CI runner` job of
`.github/workflows/ci.yml` (self-hosted primary, GitHub-hosted overflow), and
the check/test work is the single required `Tokmd Rust Result` gate. Branch
protection already required only `Tokmd Rust Result`, so no required check
changed. The normative gate contract is `docs/specs/ub-review-ci-gate.md`.

This document is retained as the historical contract for the routed model.
References below to `Tokmd Rust Small Result` as the required check describe the
retired frontdoor, not current branch protection.

This document defines how `tokmd-swarm` should choose between self-hosted and
GitHub-hosted execution for routine PR proof. It is intentionally a routing
contract, not a request to move every workflow to self-hosted runners.

## Goal

CI should be fast, predictable, and kind to the self-hosted runner fleet.

For normal PRs:

1. A cheap router job runs on GitHub-hosted infrastructure.
2. The router chooses exactly one implementation target before any
   self-hosted implementation job is queued.
3. The selected implementation job runs.
4. One normalized result job reports the required check.
5. A small route receipt explains the target and reason.

Branch protection should require only:

```text
Tokmd Rust Small Result
```

Do not require the router job or conditional implementation jobs directly. They
may be skipped by design when a different route is selected.

## GitHub Actions constraints

GitHub Actions runner choice is job-local. A job's `runs-on` value selects the
destination runner environment, and self-hosted jobs are matched by labels.
There is no automatic "try self-hosted, then fall through to GitHub-hosted"
behavior after a queued self-hosted job waits for capacity.

For that reason, fallback must happen before dispatch:

```text
route first
  -> select self-hosted or GitHub-hosted
  -> run one implementation job
  -> normalize the result
```

Racing self-hosted and GitHub-hosted implementations is not the default model.
It doubles load precisely when the fleet is under pressure.

References:

- [Choosing the runner for a job](https://docs.github.com/en/actions/using-jobs/choosing-the-runner-for-a-job)
- [Control the concurrency of workflows and jobs](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/control-the-concurrency-of-workflows-and-jobs)

## Scope

The first routed lane is Rust Small.

In scope:

- same-repo PR Rust Small proof;
- trusted branch push Rust Small proof;
- merge queue Rust Small proof;
- manual Rust Small dispatch;
- GitHub-hosted fallback for Rust Small when fallback is safe and allowed.

Out of scope until Rust Small routing is boring:

- release, publish, signing, tag, Docker, GHCR, crates.io, and `v1` alias jobs;
- full matrix migration;
- Nix full;
- fuzzing;
- mutation;
- Codecov upload policy;
- macOS and Windows lanes;
- default routing for expensive hosted replacements.

## Trust boundary

Self-hosted runners are for trusted work only.

Allowed on self-hosted:

- `workflow_dispatch` from trusted maintainers;
- `merge_group`;
- same-repository `pull_request`;
- trusted branch `push`.

Never route to self-hosted by default:

- fork PRs;
- untrusted code paths;
- secret-heavy jobs;
- release, publish, signing, tag, alias, or package mutation jobs unless a
  separate release policy explicitly assigns them.

Fork PRs should route directly to GitHub-hosted proof. Unknown trust state is a
hosted fallback reason for Rust Small, not a reason to queue on self-hosted.

## Self-hosted eligibility

Use self-hosted only when all of these are true:

- the event is trusted;
- at least one eligible runner is online;
- the eligible runner is not busy;
- required labels match the lane pool;
- the runner is not quarantined;
- runner health is fresh;
- disk and scratch space are above guard thresholds;
- the route budget says a self-hosted slot is available.

Otherwise route Rust Small to GitHub-hosted when fallback is allowed. If the
runner API, runner token, or health data is unavailable, choose GitHub-hosted
for Rust Small rather than occupying a self-hosted queue blindly.

Target-state runner labels should describe a pool, for example:

```text
self-hosted, linux, x64, em-ci, trusted-pr, rust-small
```

Machine-specific labels can remain implementation details while the fleet is
being migrated, but branch protection and user-facing checks must not depend on
a specific machine name.

## Runner health

GitHub can report whether a runner is online and busy. Local conditions still
need a separate health signal.

A runner health receipt should include:

- runner name or id;
- labels;
- timestamp;
- disk free;
- scratch free;
- Rust toolchain state;
- Docker availability when the lane needs Docker;
- `git` availability;
- status: `healthy`, `degraded`, or `quarantined`.

Treat stale health as degraded. The default freshness window is 15 minutes for
Rust Small. A stale, degraded, quarantined, low-disk, or low-scratch runner
routes to GitHub-hosted when fallback is allowed.

## Capacity policy

Self-hosted routing should not define "full" by observing that a job has been
queued for a long time. That is too late.

The router should decide capacity before dispatch:

```text
eligible_runners > busy_runners
healthy_runners > quarantined_runners
self_hosted_pressure < lane_capacity
```

The first Rust Small policy is conservative:

- do not queue on self-hosted if all eligible runners are busy;
- reserve capacity for interactive or manual work when possible;
- fall back to GitHub-hosted immediately when capacity is full;
- do not queue expensive hosted substitutes unless a label or manual input
  explicitly authorizes that cost.

## Job layout

The routed Rust Small shape is:

```text
route-rust-small
  runs-on: ubuntu-latest
  outputs:
    target: self-hosted | github-hosted | none
    reason: ...
    eligible_runners: ...
    busy_runners: ...
    health: ...

rust-small-self-hosted
  if: route target == self-hosted
  runs-on: self-hosted pool label set

rust-small-github-hosted
  if: route target == github-hosted
  runs-on: ubuntu-latest

tokmd-rust-small-result
  if: always()
  checks the selected implementation job
```

The required public contract remains the aggregate result check, not either
conditional implementation job.

Result semantics:

- selected implementation succeeds -> `Tokmd Rust Small Result` succeeds;
- selected implementation fails, times out, or is cancelled -> result fails;
- non-selected implementation jobs are ignored when skipped;
- non-selected implementation jobs that run unexpectedly -> result fails;
- router failure without a safe fallback -> result fails.

## Manual proof modes

`workflow_dispatch` exposes routed Rust Small proof modes so maintainers can
exercise fallback behavior without waiting for real fleet pressure:

| Mode | Expected route | Purpose |
| --- | --- | --- |
| `auto` | policy decision | Normal trust/capacity/health routing. |
| `force-github-hosted` | GitHub-hosted | Hosted diagnostic run without touching self-hosted capacity. |
| `force-self-hosted` | self-hosted if trusted | Self-hosted diagnostic run; still denied for unsafe events. |
| `simulate-full` | GitHub-hosted | Proves full self-hosted capacity falls back before dispatch. |
| `simulate-unhealthy` | GitHub-hosted | Proves degraded health falls back before dispatch. |
| `simulate-api-unavailable` | GitHub-hosted | Proves missing runner API state falls back safely. |
| `simulate-untrusted` | GitHub-hosted | Proves untrusted event state cannot select self-hosted. |

Simulation modes are proof inputs for the router. They do not mark the runner
fleet as actually full, unhealthy, API-unavailable, or untrusted.

## Route receipt

Every routed run should write a small JSON receipt and include the same
pre-dispatch decision summary in the workflow summary.

Example:

```json
{
  "schema": "tokmd.ci_route.v1",
  "lane": "rust-small",
  "target": "github-hosted",
  "reason": "self_hosted_capacity_full",
  "trusted_event": true,
  "eligible_runners": 2,
  "busy_runners": 2,
  "healthy_runners": 2,
  "health": "healthy",
  "health_age_seconds": 12,
  "disk_free_bytes": 17179869184,
  "scratch_free_bytes": 17179869184,
  "min_free_bytes": 8589934592,
  "fallback_allowed": true,
  "selected_runner_label": "ubuntu-24.04"
}
```

The route receipt must not contain secrets. It explains the target, reason,
trust decision, runner counts, health state, fallback allowance, and selected
runner label/name when a self-hosted runner is chosen. For label/group based
self-hosted dispatch, the route receipt's selected runner name is a
pre-dispatch idle-runner candidate, not a pin. The normalized result receipt
owns the actual runner name reported by GitHub Actions after dispatch. When a runner health
receipt is available, it also records health age plus disk and scratch guard
inputs. It is diagnostic routing evidence; the branch-protection contract is
still the normalized result check.

The normalized `Tokmd Rust Small Result` job writes
`target/ci/routed-rust-small-result.json`. That result receipt owns post-run
fields: selected implementation job, selected result, sibling job results,
run attempt, rerun count, and best-effort selected-job telemetry such as
duration, queue time, runner name, and cache policy note. Missing telemetry is
reported as unavailable, not converted into zero duration.

## Concurrency and anti-thrash

Routed PR checks should cancel stale work from the same PR branch without
serializing unrelated PRs.

Use workflow-specific concurrency groups:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.repository }}-${{ github.event.pull_request.number || github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

For merge queue, main, release, and publication side validation, keep runs
commit-scoped unless the lane policy explicitly says cancellation is safe.

Do not use one global concurrency group for all PRs. Do not cancel release or
publication evidence merely because a newer commit exists elsewhere.

## Required-check policy

Require:

```text
Tokmd Rust Small Result
```

Do not require:

```text
Route Tokmd Rust Small
Tokmd Rust Small on Self Hosted
Tokmd Rust Small on GitHub Hosted
```

Conditional route and implementation jobs can be skipped for valid reasons. A
required check tied to a skipped implementation job blocks merges for the wrong
reason and hides the aggregate contract.

## Related docs

- [Default PR gate](default-pr-gate.md)
- [Swarm publication model](swarm-routing.md)
- [CI cache and cancellation policy](cache-and-cancellation.md)
- [CI actuals](ci-actuals.md)
- [Routed Rust Small dogfood](routed-rust-small-dogfood.md)
- [Runner health runbook](runner-health-runbook.md)
