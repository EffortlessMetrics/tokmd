# Coverage

Coverage is an execution-surface signal.

It answers: did tests execute this Rust surface?

It does not answer:
- whether the tests would catch wrong behavior,
- whether mutation adequacy is strong,
- whether proof-policy is complete,
- whether WASM/browser behavior is validated,
- whether release packaging is valid.

Those are separate lanes: mutation, proof-policy, affected proof, WASM, Nix, publish-surface, cargo-deny, and release checks.

## Coverage Lane

The coverage workflow is label/main/manual gated:
- `push` to main
- `workflow_dispatch`
- PRs labeled `coverage` or `full-ci`

Coverage runs on:
- Ubuntu latest
- Rust MSRV (1.92)
- `cargo-llvm-cov` with `--workspace --all-features --locked`

Outputs:
- `coverage.json` — JSON report (Codecov compatible)
- `coverage.txt` — text summary
- `lcov.info` — LCOV format (standard for coverage tools)
- GitHub Actions artifact: `coverage-report`

## Codecov Integration

Coverage is uploaded to [Codecov](https://codecov.io/gh/EffortlessMetrics/tokmd) when `CODECOV_TOKEN` is present in GitHub secrets.

Codecov configuration (`codecov.yml`) is advisory:
- Project coverage threshold: 5% delta, informational
- Patch coverage target: 70%, informational
- Comments: disabled (reduces PR noise)
- GitHub check annotations: disabled

The coverage dashboard and badge are the durable receipts. Codecov comments are not used during the advisory baseline phase.

## Durable Receipts

- `coverage.json` — per-run JSON coverage report
- `coverage.txt` — per-run text summary
- `lcov.info` — per-run LCOV artifact
- Codecov dashboard — aggregated coverage trends
- Codecov badge — link in README

## When Coverage Runs

Coverage is not gated by default. It runs on:
- `push` to main (always)
- `workflow_dispatch` (manual)
- PR with `coverage` label (manual)
- PR with `full-ci` label (manual)

## CI Claim Boundary

Coverage verifies that:
- Tests execute the intended Rust surface
- Coverage data is generated and reported

Coverage does not verify:
- Test adequacy (use mutation testing)
- Policy completion (use proof-policy)
- WASM surface (use WASM/browser tests)
- Release validity (use publish-surface)
- Security posture (use cargo-deny, security audit)

## See Also

- `.github/workflows/coverage.yml` — workflow definition
- `codecov.yml` — Codecov configuration
- `policy/ci-lane-whitelist.toml` — lane metadata
- `ci/proof.toml` — proof-policy scope definitions
