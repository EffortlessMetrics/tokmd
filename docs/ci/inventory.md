# tokmd CI inventory snapshot

Snapshot of CI lanes as of 2026-05-07. Generated as the human-readable
companion to `policy/ci-lane-whitelist.toml`. Update on rollout PRs.

## Frontdoor (cheap default)

| Lane ID | Job | Runner | Base LEM | Notes |
|---------|-----|--------|----------|-------|
| `msrv_check` | MSRV Check | ubuntu | 5 | MSRV cargo check. PR 04 moves to 1.93. |
| `quality_gate` | Quality Gate | ubuntu | 8 | `cargo xtask gate --check`. |
| `proof_policy` | Proof Policy | ubuntu | 3 | `cargo xtask proof-policy --check`. |
| `affected_proof_plan` | Affected Proof Plan | ubuntu | 4 | Wrapped by PR 08 PR Plan. |
| `feature_boundaries` | Feature Boundaries | ubuntu | 10 | Analysis microcrate boundaries. |
| `typos` | Typos | ubuntu | 1 | crate-ci/typos. |
| `cargo_deny` | Cargo Deny | ubuntu | 4 | Advisories + licenses. |
| `version_consistency` | Version consistency | ubuntu | 2 | Release metadata alignment. |
| `docs_check` | Docs Check | ubuntu | 4 | `cargo xtask docs --check`. |
| `ci_required` | CI (Required) | ubuntu | 1 | Aggregator. |

## Expensive default — needs exception during rollout

| Lane ID | Job | Runner | Base LEM | Exception |
|---------|-----|--------|----------|-----------|
| `build_test_linux_windows` | Build & Test | mixed | 40 | `ci_exception_windows_full_test_default` |
| `wasm_compile_test` | Wasm Compile & Test | ubuntu | 25 | `ci_exception_wasm_default` |
| `nix_pr_package_gate` | Nix PR Package Gate | ubuntu | 35 | `ci_exception_nix_default` |
| `mutation_required` | Mutation Testing (Required) | ubuntu | 45 | `ci_exception_mutation_default` |
| `proptest_smoke` | Proptest Smoke | ubuntu | 8 | (no exception; fits frontdoor band) |
| `publish_surface` | Publish Surface | ubuntu | 8 | (no exception; cheap dry-run) |

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
feature_boundaries         10
typos                       1
cargo_deny                  4
version_consistency         2
docs_check                  4
build_test_linux_windows   40
wasm_compile_test          25
nix_pr_package_gate        35
mutation_required          45
proptest_smoke              8
publish_surface             8
ci_required                 1
                          ----
                           203  (high-cost / override band)
```

PR 10–12 demote the bottom block to risk-pack routing. The target frontdoor
band drops to roughly 30–40 LEM for an ordinary Rust-only PR.
