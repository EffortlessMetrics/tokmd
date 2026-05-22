# Default PR gate

After PR 10, the ordinary `pull_request` gate runs only the cheap
"frontdoor" lanes plus the existing proof / cockpit / typos jobs.
Expensive lanes are gated on labels and on push-to-main:

| Job | Now triggers on PR when... |
|-----|----------------------------|
| `Build & Test (Linux)` | always |
| `Build & Test (Windows)` | label `windows` / `full-ci` (still on every push) |
| `Build & Test (macOS)` | push-only (unchanged) |
| `Wasm Compile & Test` | label `wasm` / `full-ci` |
| `Nix PR Package Gate` | label `nix` / `release-check` / `full-ci` |
| `Mutation Testing` | label `mutation` / `full-ci` (replaced by ripr advisory in PR 11) |
| `Proptest Smoke` | label `property-tests` / `full-ci` |
| `MSRV Check` | always |
| `Quality Gate` | always |
| `Cargo Deny` | always |
| `Typos` | always |
| `Proof Policy` | always |
| `Affected Proof Plan` | pull_request only |
| `Feature Boundaries` | always |
| `Publish Surface` | always (small dry-run) |
| `Version consistency` | always |
| `Docs Check` | always |

## CI (Required) summary

The aggregator's `if: always()` posture means **skipped jobs do not fail**
the summary — only `failure` and `cancelled` results do. So a default PR
that skips Windows, WASM, Nix, mutation, and proptest will still see a
green `CI (Required)` row provided the lanes that *did* run all passed.

## Default-PR LEM after the slimming

Roughly (per `docs/ci/inventory.md`, with advisory proof/cockpit lanes now
included in the inventory):

```text
msrv_check                   5
quality_gate                 8
proof_policy                 3
affected_proof_plan          4
ci_detect_risk_packs         1
fast_proof_run_advisory      5
feature_boundaries          10
typos                        1
cargo_deny                   4
version_consistency          2
docs_check                   4
build_test_linux            12
publish_surface              8
ci_lane_whitelist            3
pr_cockpit_report            3
no_panic_family              3
pr_plan_advisory             1
ripr_advisory                2
scoped_coverage_executor_non_required 12
ci_required                  1
tokmd_rust_small_route       1
tokmd_rust_small_result     20
                          ----
                           113   tokmd-swarm default PR (was ~203)
```

That remains below the hard override ceiling, but it is intentionally reported
as high-cost while the advisory proof executor, proof-run observation lanes,
and routed Rust Small frontdoor collect real timing evidence.

`tokmd-swarm` workbench PRs also run the routed Rust Small frontdoor. The
router and aggregate result are default PR lanes. The lane catalogue also
includes the conditional implementation jobs for CPX42, CX43, CX53, and
GitHub-hosted fallback, but those jobs are mutually exclusive and are not
counted as ordinary default PR lanes. The aggregate result carries a
conservative one-route estimate, so a small swarm PR budgets the selected
route without counting every skipped implementation target.

## Anti-patterns

- Don't use `full-ci` to dodge a real failure; the deep lanes catch
  things the default lane is *intentionally* skipping.
- Don't apply per-pack labels to silence routing — fix the change.
- Don't depend on the matrix entry name "windows" appearing under
  `build` — the matrix split is intentional so `if:` can gate Windows
  independently.
