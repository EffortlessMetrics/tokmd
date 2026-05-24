# Summary

<!-- AI-FILL:SUMMARY -->
<!-- Brief description of what this PR does -->

## Why

<!-- What concrete reviewer, agent, user, proof, or workflow ambiguity does this remove? -->

## Type of Change

<!-- Check the relevant option(s) -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that changes existing behavior)
- [ ] Refactoring (no functional changes)
- [ ] Documentation update
- [ ] Tests/CI improvement

---

## Glass Cockpit

<!-- AI-FILL:COCKPIT -->
| Metric | Value |
|--------|-------|
| **Change Surface** | |
| Commits | <!-- commits --> |
| Files changed | <!-- files_changed --> |
| Lines (+/-) | <!-- insertions/deletions --> |
| Net lines | <!-- net_lines --> |
| **Composition** | |
| Code | <!-- code_pct -->% |
| Tests | <!-- test_pct -->% |
| Docs | <!-- docs_pct -->% |
| Config | <!-- config_pct -->% |
| **Contracts** | |
| API changed | <!-- api_changed --> |
| CLI changed | <!-- cli_changed --> |
| Schema changed | <!-- schema_changed --> |
<!-- /AI-FILL:COCKPIT -->

---

## Trend

<!-- SECTION:TREND -->
<!-- AI-FILL:TREND -->
| Metric | Current | Previous | Delta | Direction |
|--------|---------|----------|-------|-----------|
| Health | <!-- health_current --> | <!-- health_previous --> | <!-- health_delta --> | <!-- health_direction --> |
| Risk | <!-- risk_current --> | <!-- risk_previous --> | <!-- risk_delta --> | <!-- risk_direction --> |
| Complexity | <!-- complexity_summary --> | | | <!-- complexity_direction --> |
<!-- /AI-FILL:TREND -->
<!-- /SECTION:TREND -->

---

## Review Plan

<!-- AI-FILL:REVIEW_PLAN -->
<!-- Risk-ranked file order for review -->
| Priority | File | Reason |
|----------|------|--------|
<!-- /AI-FILL:REVIEW_PLAN -->

---

## Verification

<!-- Summarize the actual proof run for this PR. Keep planned proof distinct
     from executed proof. -->

- **Proof summary:** <!-- commands run + result, hosted check IDs/URLs if useful -->
- **Routed fallback note:** <!-- If `Tokmd Rust Small Result` first failed with
  `no_idle_runner`, record the failed run, the fallback authorization label or
  dispatch used, and the newer successful routed run. Label-only edits do not
  start a new routed check. -->

- [ ] `cargo build` compiles
- [ ] `cargo test` passes
- [ ] `cargo clippy` clean
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG updated (if applicable)

---

## Repo Boundary

<!-- Check exactly one repo-boundary path. -->

- [ ] Normal swarm PR: targets `EffortlessMetrics/tokmd-swarm`, contains no
      release/publish/signing/tag/Docker/v1 alias mutation, and should squash
      merge after required checks pass.
- [ ] Publication import PR: targets `EffortlessMetrics/tokmd`, imports a
      `tokmd-swarm` head, and must merge with a merge commit, not squash.
- [ ] Other: explain why this PR is outside the normal swarm/publication loop.

- **Repo-graph evidence:** <!-- Replace with the matching command and result.
  Normal swarm PR: cargo xtask repo-graph --publication public/main --swarm HEAD --expect swarm-ahead
  Publication import PR: cargo xtask repo-graph --publication origin/main --swarm HEAD --expect swarm-ahead
  Post-publication fast-forward: cargo xtask repo-graph --publication public/main --swarm origin/main --expect aligned
-->
- **Publication import fields:** <!-- For publication import PRs only. Record
  Swarm-Head, Swarm-Range, imported swarm PRs or commits, required swarm checks,
  publication checks, and the post-merge fast-forward command/result. State
  explicitly that the PR must merge with a merge commit, not squash. -->
- **Post-fast-forward branch health:** <!-- If branch CI starts after the graph
  is aligned, record repo/run IDs, shared headSha, active jobs, and the boundary
  that in_progress jobs are not passing proof. -->

## Claim Boundary

<!-- State what this PR proves and what it does not prove. Call out unchanged
     release, publish, signing, Docker, v1 alias, proof-promotion, Codecov,
     AST, evidencebus, or public CLI behavior when relevant. -->

## Rollback

<!-- How to revert or park this change without breaking the swarm/publication
     graph or losing needed evidence. -->

---

## CI economics

<!-- See docs/ci/cost-and-verification-policy.md and docs/ci/lem-budgeting.md -->

- **Default PR LEM impact:** <!-- estimated band (e.g. 0-35 normal) -->
- **Workflows touched:** <!-- list of .github/workflows/*.yml files -->
- **Branch protection impact:** <!-- none / adds required job / removes required job -->
- **Failure mode caught:** <!-- one sentence on what proof this PR buys -->
- **Cheaper signal considered:** <!-- what was rejected and why -->
- **CI rollback path:** <!-- how to revert CI changes without losing the receipt model -->

---

<details>
<summary>Receipts</summary>

<!-- AI-FILL:RECEIPTS -->
```json
<!-- Full cockpit JSON receipt -->
```
<!-- /AI-FILL:RECEIPTS -->

</details>
