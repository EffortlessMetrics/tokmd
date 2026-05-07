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
| GitGuardian | Secret scanning status from the configured GitHub integration. | Managed outside repository workflows. | Security signal; branch-protection status must remain an explicit maintainer decision. |

## Held Services

| Service | Reason |
| --- | --- |
| Factory Droid | Held until maintainers approve the service, API key handling, fork PR behavior, permission model, and failure policy. |

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
