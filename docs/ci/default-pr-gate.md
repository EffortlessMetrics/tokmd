# Default PR gate (phase 2, #226)

The required merge check is **`Tokmd Rust Result`** — a single tight gate with
advisory `route`, concurrent `cargo xtask gate --check`, `cargo test
--all-features`, `cargo xtask proof-policy --check`, and advisory `ub-review`.

Parallel satellite lanes still run for MSRV, docs, deny, typos, risk-gated
builds, and proof planning. They are visible on the PR but are **not** aggregated
into a `CI (Required)` job; branch protection should require only `Tokmd Rust
Result` after the admin step in issue #226.

| Job | Now triggers on PR when... |
|-----|----------------------------|
| `Tokmd Rust Result` | always (required) |
| `Route CI runner` | always (advisory) |
| `MSRV Check` | always (satellite) |
| `Cargo Deny` | always (satellite) |
| `Typos` | always (satellite) |
| `Docs Check` | always (satellite) |
| `Feature Boundaries` | always (satellite) |
| `Publish Surface` | always (satellite) |
| `Version consistency` | always (satellite) |
| `Affected Proof Plan` | pull_request only |
| `Build & Test (Windows)` | label `windows` / `full-ci` (still on every push) |
| `Build & Test (macOS)` | push-only (unchanged) |
| `Wasm Compile & Test` | label `wasm` / `full-ci` |
| `Nix PR Package Gate` | label `nix` / `release-check` / `full-ci` |
| `Mutation Testing` | label `mutation` / `full-ci` |
| `Proptest Smoke` | label `property-tests` / `full-ci` |

## Retired lanes (folded into `Tokmd Rust Result`)

- `Quality Gate` → `cargo xtask gate --check` in the gate job background
- `Build & Test (Linux)` → `cargo test --all-features` in the gate job background
- `Proof Policy` → `cargo xtask proof-policy --check` in the gate job background
- `CI (Required)` → replaced by single required check + `CI Actuals (Advisory)`
- `Tokmd Rust Small Result` + routed `em-routed-rust-small.yml` frontdoor
  (`Route Tokmd Rust Small`, `Tokmd Rust Small on Self Hosted`, `Tokmd Rust
  Small on GitHub Hosted`) → retired in phase 3 (#299); routing folded into the
  `Route CI runner` lane of `ci.yml` and the required `Tokmd Rust Result` gate

## Advisory lane summary

`CI Actuals (Advisory)` publishes LEM receipts and a non-blocking lane table.
Only `failure` on **`Tokmd Rust Result`** blocks merge once branch protection is
updated.

Default-PR lanes marked `always` and `blocking` in the lane catalog must not be
moved behind a same-repository guard unless the PR also adds a separate hosted
fork-safe path. This includes cheap static proof such as `Typos` and the CI
Policy workflow's `No Bare Self-Hosted Routing` guard.

## Default-PR LEM after phase 2

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
no_bare_self_hosted          1
                          ----
                            93   tokmd-swarm default PR (was ~203)
```

That remains below the hard override ceiling, but it is intentionally reported
as high-cost while the advisory proof executor and proof-run observation lanes
collect real timing evidence.

The duplicate routed Rust Small frontdoor (`em-routed-rust-small.yml`) was
retired in phase 3 (#299). Runner routing for `tokmd-swarm` workbench PRs now
lives in the `Route CI runner` lane of `ci.yml`, and its check/test work is the
single required `Tokmd Rust Result` gate. The historical routing contract is
preserved in `docs/ci/routed-ci-policy.md`.

Phase-2 target for issue #226 is a single tight gate documented in
`docs/specs/ub-review-ci-gate.md`, with `cargo xtask ci-gate-contract`
enforcing the reference fixture and reporting the live `ci.yml` gap
advisory until migration lands.

## Anti-patterns

- Don't use `full-ci` to dodge a real failure; the deep lanes catch
  things the default lane is *intentionally* skipping.
- Don't apply per-pack labels to silence routing — fix the change.
- Don't depend on the matrix entry name "windows" appearing under
  `build` — the matrix split is intentional so `if:` can gate Windows
  independently.
