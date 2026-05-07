# External Services Policy

This policy governs third-party services that receive repository data, post
review or status output, upload artifacts, or require non-GitHub secrets. It
does not replace normal GitHub Actions guidance; it applies when a workflow,
app, or bot introduces an external service boundary.

## Default Rule

External services are opt-in product and security decisions. A service workflow
must not be merged until this document or a linked ADR records:

- the service name and purpose;
- the data or artifacts sent to the service;
- required secrets and their expected scopes;
- fork pull request behavior;
- status/check failure behavior;
- whether the service is advisory or required;
- how maintainers disable or rotate the integration.

The default posture is advisory. Branch protection should not require an
external service unless maintainers explicitly promote it after successful
repository-local and CI evidence.

## Approved Services

| Service | Purpose | Secrets | Status |
| --- | --- | --- | --- |
| Codecov | Advisory Rust coverage telemetry from `cargo-llvm-cov` LCOV artifacts. | `CODECOV_TOKEN` when token-based upload is needed. | Advisory; upload steps use non-blocking failure behavior. |
| CodeRabbit | Advisory PR review comments and status. | Managed outside repository workflows. | Advisory unless branch protection explicitly says otherwise. |
| Factory Droid via `EffortlessMetrics/droid-action-safe` | Advisory same-repo PR review, trusted maintainer `@droid` review commands, and scheduled/manual security scans. | `FACTORY_API_KEY` for Factory, `MINIMAX_API_KEY` for the MiniMax BYOK model bridge. | Advisory external review service; raw debug artifact upload is disabled and branch protection must not require Droid unless maintainers explicitly promote it. |
| GitGuardian | Secret scanning status from the configured GitHub integration. | Managed outside repository workflows. | Security signal; branch-protection status must remain an explicit maintainer decision. |

## Held Services

| Service | Reason |
| --- | --- |
| None currently. | New services still require an approved policy or ADR before workflow introduction. |

## Factory Droid Guardrails

`tokmd` uses the pinned safe action wrapper
`EffortlessMetrics/droid-action-safe@01e76b659e4b1e5f23feedc8cfabf8dc14c7485f`.
Repository workflows must not call `Factory-AI/droid-action` directly for
secrets-backed BYOK runs.

The Droid workflows have these trust boundaries:

- `Droid Auto Review` runs only for same-repository pull requests and skips
  titles containing `[skip-review]`.
- `Droid Tag` accepts `@droid` only from `OWNER`, `MEMBER`, or
  `COLLABORATOR` authors.
- `Droid Security Scan` runs on manual dispatch and the scheduled weekly scan.
- `upload_debug_artifacts` must remain `false` for standard runs.
- `show_full_output` should remain `false` unless a maintainer explicitly
  opens a debugging window.

Required secrets:

- `FACTORY_API_KEY`: authorizes the Factory Droid action.
- `MINIMAX_API_KEY`: configures the `custom:MiniMax-M2.7-0` BYOK model in
  `$HOME/.factory/settings.local.json` during the workflow.

Secret handling rules:

- keep `MINIMAX_API_KEY` scoped only to repositories in the Droid rollout batch;
- rotate `MINIMAX_API_KEY` after suspected exposure or rollout-scope changes;
- confirm `FACTORY_API_KEY` remains valid during smoke tests;
- do not print either secret, upload `$HOME/.factory/**`, or preserve raw
  Droid debug prompt artifacts.

Operational proof lives in `docs/agent-context/droid-smoke-tests.md`, and the
shared rollout invariants live in `agents/shared/droid-migration.md`.

## Secrets

Secrets used by external services must:

- use a clear service-prefixed name such as `CODECOV_TOKEN`;
- have the narrowest service scope that supports the workflow;
- never be printed, cached, embedded in artifacts, or written into generated receipts;
- have an owner and rotation trigger documented here or in an ADR;
- be rotated after suspected exposure, maintainer turnover, or service scope changes.

Workflows must assume secrets are unavailable to untrusted fork pull requests.
If an integration needs comments or write permissions on fork PRs, it needs a
separate maintainer-approved design that explains why `pull_request_target` or
equivalent privilege is safe for the checked-out code path.

## Failure Behavior

External-service failures should not block routine PRs by default. A blocking
service must document:

- why local or GitHub-native proof is insufficient;
- how maintainers distinguish service outages from real repo failures;
- the bypass or disable path for incidents;
- the evidence required to make the check required.

Advisory services may still fail their own upload or analysis step when the
local artifact generation is broken. The distinction is important: a broken
repository-owned proof artifact is different from an unavailable third-party
service.

## New Service Checklist

Before adding or enabling a new service:

1. Add an entry under Approved Services or create an ADR for an experiment.
2. Document every secret name, scope, owner, and rotation trigger.
3. Document fork PR behavior and whether comments or artifacts are produced.
4. Keep the first integration advisory unless maintainers approve a hard gate.
5. Add local or GitHub-native proof for workflow syntax and generated artifacts.
6. Record the disable path in this document or the linked service docs.

## Review Notes

External-service PRs should be reviewed as product/security changes, not routine
CI cleanup. They are mergeable when the repository can explain what data leaves
GitHub, what secret authorizes it, what happens on forks, and what maintainers
should do when the service is unavailable.
