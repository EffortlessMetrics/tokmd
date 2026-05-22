# Summary

<!-- AI-FILL:SUMMARY -->
<!-- Brief description of what this PR does -->

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

- **Repo-graph evidence:** <!-- e.g. cargo xtask repo-graph --publication public/main --swarm origin/main --expect swarm-descends-publication -->

---

## CI economics

<!-- See docs/ci/cost-and-verification-policy.md and docs/ci/lem-budgeting.md -->

- **Default PR LEM impact:** <!-- estimated band (e.g. 0-35 normal) -->
- **Workflows touched:** <!-- list of .github/workflows/*.yml files -->
- **Branch protection impact:** <!-- none / adds required job / removes required job -->
- **Failure mode caught:** <!-- one sentence on what proof this PR buys -->
- **Cheaper signal considered:** <!-- what was rejected and why -->
- **Rollback path:** <!-- how to revert without losing the receipt model -->

---

<details>
<summary>Receipts</summary>

<!-- AI-FILL:RECEIPTS -->
```json
<!-- Full cockpit JSON receipt -->
```
<!-- /AI-FILL:RECEIPTS -->

</details>
