# tokmd CI inventory snapshot

Snapshot of CI lanes as of 2026-05-09. Generated as the human-readable
companion to `policy/ci-lane-whitelist.toml`. Update on rollout PRs.

> **Stale snapshot.** This table predates the phase-2/3 single-tight-gate
> consolidation (#226, #299). The routed Rust Small frontdoor
> (`Route Tokmd Rust Small`, `Tokmd Rust Small on Self Hosted`/`GitHub Hosted`,
> `Tokmd Rust Small Result`) and the `CI (Required)` aggregator were retired;
> their check/test work is now the single required `Tokmd Rust Result` gate on a
> runner chosen by the advisory `Route CI runner` job. For the current lane set
> use `policy/ci-lane-whitelist.toml`, and read `docs/ci/default-pr-gate.md` and
> `docs/specs/ub-review-ci-gate.md` for the authoritative gate shape. The rows
> below are retained as a historical rollout snapshot pending a catalogue
> refresh.

## Frontdoor (cheap default)

| Lane ID | Job | Runner | Base LEM | Notes |
|---------|-----|--------|----------|-------|
| `msrv_check` | MSRV Check | ubuntu | 5 | MSRV cargo check. |
| `quality_gate` | Quality Gate | ubuntu | 8 | `cargo xtask gate --check`. |
| `proof_policy` | Proof Policy | ubuntu | 3 | `cargo xtask proof-policy --check`. |
| `affected_proof_plan` | Affected Proof Plan | ubuntu | 4 | PR affected proof artifacts. |
| `ci_detect_risk_packs` | Detect risk packs | ubuntu | 1 | Rust-owned risk-pack output flags. |
| `fast_proof_run_advisory` | Fast Proof Run (Advisory) | ubuntu | 5 | Advisory fast proof observation. |
| `feature_boundaries` | Feature Boundaries | ubuntu | 10 | Analysis feature/module boundaries. |
| `typos` | Typos | ubuntu | 1 | crate-ci/typos. |
| `cargo_deny` | Cargo Deny | ubuntu | 4 | Advisories + licenses. |
| `version_consistency` | Version consistency | ubuntu | 2 | Release metadata alignment. |
| `docs_check` | Docs Check | ubuntu | 4 | `cargo xtask docs --check`. |
| `build_test_linux` | Build & Test (Linux) | ubuntu | 12 | Linux all-features tests. |
| `publish_surface` | Publish Surface | ubuntu | 8 | Publish-surface dry-run checks. |
| `ci_lane_whitelist` | CI Lane Whitelist | ubuntu | 3 | Advisory CI policy inventory. |
| `pr_cockpit_report` | PR Cockpit Report | ubuntu | 3 | PR cockpit metrics report. |
| `no_panic_family` | No-panic Family | ubuntu | 3 | Panic-family policy checker. |
| `pr_plan_advisory` | PR Plan (advisory) | ubuntu | 1 | LEM-aware PR plan. |
| `ripr_advisory` | ripr (advisory) | ubuntu | 2 | Static oracle-gap signal. |
| `scoped_coverage_executor_non_required` | Scoped Coverage Executor (Non-Required) | ubuntu | 12 | Advisory proof executor. |
| `ci_required` | CI (Required) | ubuntu | 1 | Aggregator. |
| `no_bare_self_hosted` | No Bare Self-Hosted Routing | ubuntu | 1 | Runner routing policy guard. |
| `tokmd_rust_small_route` | Route Tokmd Rust Small | ubuntu | 1 | Swarm route selector. |
| `tokmd_rust_small_result` | Tokmd Rust Small Result | ubuntu | 20 | Aggregate budget for one selected routed implementation. |

## Risk-gated / expensive lanes

| Lane ID | Job | Runner | Base LEM | Trigger |
|---------|-----|--------|----------|---------|
| `build_test_windows` | Build & Test (Windows) | Windows | 20 | push, `windows`, `full-ci`, or Windows path risk. |
| `wasm_compile_test` | Wasm Compile & Test | ubuntu | 25 | push, `wasm`, `full-ci`, or WASM path risk. |
| `nix_pr_package_gate` | Nix PR Package Gate | ubuntu | 35 | push, `nix`, `release-check`, `full-ci`, or release path risk. |
| `mutation_required` | Mutation Testing | ubuntu | 45 | push, `mutation`, or `full-ci`. |
| `proptest_smoke` | Proptest Smoke | ubuntu | 8 | push, `property-tests`, `full-ci`, or core-receipts path risk. |
| `rust_coverage` | Codecov Coverage | ubuntu | 30 | push, workflow dispatch, `coverage`, or `full-ci`. |

## Conditional routed implementation lanes

These lanes are listed in the whitelist for ownership and evidence tracking,
but they are mutually exclusive at runtime. PR Plan counts the route selector
and aggregate result lane by default instead of summing every skipped target.

| Lane ID | Job | Runner | Base LEM | Trigger |
|---------|-----|--------|----------|---------|
| `tokmd_rust_small_self_hosted` | Tokmd Rust Small on Self Hosted | em-ci-small | 12 | selected by routed Rust Small when trusted healthy capacity exists. |
| `tokmd_rust_small_github` | Tokmd Rust Small on GitHub Hosted | ubuntu | 20 | selected by routed Rust Small fallback. |

## Push / main-only

| Lane ID | Job | Runner | Base LEM | Notes |
|---------|-----|--------|----------|-------|
| `build_macos_push` | Build & Test (macOS) | macOS | 60 | `if: github.event_name == 'push'`. |

## Estimated default-PR LEM today

```text
msrv_check                  5
quality_gate                8
proof_policy                3
affected_proof_plan         4
ci_detect_risk_packs        1
fast_proof_run_advisory     5
feature_boundaries         10
typos                       1
cargo_deny                  4
version_consistency         2
docs_check                  4
build_test_linux           12
publish_surface             8
ci_lane_whitelist           3
pr_cockpit_report           3
no_panic_family             3
pr_plan_advisory            1
ripr_advisory               2
scoped_coverage_executor_non_required  12
ci_required                 1
no_bare_self_hosted         1
tokmd_rust_small_route      1
tokmd_rust_small_result    20
                          ----
                           114  (high-cost band; below hard override ceiling)
```

Expensive Windows, WASM, Nix, mutation, proptest, and coverage lanes are now
label, path-risk, push, or dispatch routed instead of ordinary PR defaults.
Routed Rust Small implementation jobs are represented in the default estimate
by the aggregate `tokmd_rust_small_result` lane so skipped route targets do not
force a budget override.
