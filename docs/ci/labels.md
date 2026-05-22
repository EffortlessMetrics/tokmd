# tokmd CI labels

Labels select expensive lanes that ordinary PRs skip. The PR Plan job (PR
08) is the source of truth for which labels apply to a given PR.

## Lane-selection labels

| Label | Effect |
|-------|--------|
| `full-ci` | Run every default-blocking lane: Windows, WASM, Nix, mutation, proptest expansion, all-features. |
| `wasm` | Add WASM compile + Node/browser runner tests. |
| `windows` | Add Windows guardrail lane. |
| `nix` | Add Nix flake + package build. |
| `release-check` | Add publish surface + version consistency + Nix. |
| `mutation` | Add cargo-mutants on changed files. |
| `property-tests` | Expand proptest smoke past the default short budget. |

## Budget labels

| Label | Effect |
|-------|--------|
| `ci-budget-ack` | Acknowledge an estimate above the elevated band. Suppresses warning, does not bypass hard ceiling. |
| `ci-budget-override` | Bypass the >125 LEM hard ceiling. Use sparingly. |

Use `ci-budget-override` for a narrow PR only when the PR body or review
comment cites why the estimate is an overcount. The common case is the routed
Rust Small catalogue: PR Plan may count mutually exclusive implementation lanes
that the hosted check rollup proves did not all run.

## Advisory labels

| Label | Effect |
|-------|--------|
| `ripr-waive` | After PR 16, ack a high-confidence ripr finding without changing tests. |

## Anti-patterns

- Don't apply `full-ci` to dodge a real failure. The deep lanes catch
  things the default lane is *intentionally* skipping.
- Don't apply `ci-budget-override` to ship a PR that's broad because it
  bundles unrelated work. Split the PR instead.
- Don't treat an older failed PR Plan attempt as current after a label-triggered
  rerun has passed for the same head SHA.
- Labels do not retroactively change branch protection. The required
  summary job still has to pass.
