# ripr evidence policy

`ripr` is the static "mutation-testing-lite" lane. It does not run mutants.
It asks whether changed behavior appears exposed to a meaningful test
discriminator. It is intentionally cheaper than `cargo-mutants` and gives
mutation-shaped oracle-gap signal at static-analysis prices.

This file pins how we read ripr findings, when they are advisory, and when
(eventually, after PR 16) they soft-gate.

## Severities

| ripr finding | Severity (default) | Meaning |
|--------------|--------------------|---------|
| `exposed` | info | A meaningful oracle path exists. |
| `weakly_exposed` | warning | Path exists but discriminator is weak. |
| `reachable_unrevealed` | warning | Code is reachable but no test would observe a behavior change. |
| `no_static_path` | note | Static analyzer could not link the change to any test. |
| `infection_unknown` | note | Could not determine whether the mutation would reach a test. |
| `propagation_unknown` | note | Could not determine whether infection would propagate to an oracle. |
| `static_unknown` | note | Analyzer abstained — usually macros / generated code / cfg gates. |

## Advisory phase (PR 11 → PR 16)

- ripr runs on production Rust diffs only.
- Findings are uploaded as JSON / SARIF / Markdown artifacts.
- The job does **not** block merge.
- `mutation` / `full-ci` labels still run real `cargo-mutants`.

## Soft-gate phase (PR 16)

After several weeks of advisory data, soft-gate only narrow cases:

```text
new reachable_unrevealed or weakly_exposed
+ production Rust changed
+ no nearby test changed
+ not suppressed
+ high-confidence finding
```

Allowed override labels:

- `ripr-waive` — acknowledge a specific finding intentionally.
- `full-ci` — the deep mutation lane will run anyway.
- `ci-budget-ack` — the PR is otherwise within scope.

The soft gate does **not** apply to `static_unknown`, `no_static_path`, or
baseline findings.

## Suppressions

ripr suppressions live in `policy/ripr-suppressions.toml`. Each entry has one
suppression family field plus the identifiers required by that family:

- `kind` — suppression family, either `exposure_gap` or `test_efficiency`.
- `finding_id` — required for `exposure_gap` entries.
- `test` — required for `test_efficiency` entries.
- `path` — optional repo-relative file path for `test_efficiency`.
- `owner`, `reason`, `expires` — reviewed ownership, rationale, and optional expiry.

Don't suppress because a finding is annoying. Suppress because the test
gap is intentional and explained.
