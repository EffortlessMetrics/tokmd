# Performance Smoke Receipts

Lane 3 (measured performance and CI feedback) uses `cargo xtask perf-smoke` to
collect repeatable timing evidence before optimization work. This guide is for
maintainers and agents deciding whether a performance change is justified.

## When to use it

Run perf-smoke when:

- a PR claims a performance improvement on `lang`, `module`, `export`, or a
  bounded `analyze` preset;
- you need a baseline before touching clone hot paths or analysis enrichers;
- you are scoping Lane 3 work and want machine-readable timings instead of ad
  hoc `time` output.

Do **not** use perf-smoke as a required CI gate or release verdict. It is an
opt-in measurement receipt, not proof of stable production performance.

## Quick start

Core workflows only (lang / module / export):

```bash
cargo xtask perf-smoke \
  --target-repo . \
  --output target/perf/perf-smoke.json \
  --sha "$(git rev-parse HEAD)"
```

Add bounded analysis presets when the optimization touches analysis:

```bash
cargo xtask perf-smoke \
  --target-repo . \
  --output target/perf/perf-smoke-health.json \
  --analysis-preset health \
  --analysis-max-files 500 \
  --analysis-max-bytes 52428800 \
  --sha "$(git rev-parse HEAD)"
```

Repeat `--analysis-preset` to time multiple presets in one receipt.

## Receipt contract

| Field | Meaning |
| --- | --- |
| `schema` | `tokmd.perf_smoke.v1` |
| `schema_version` | `2` (adds optional `analysis_workflows`) |
| `sha` | Commit recorded in the receipt (`--sha` or `GITHUB_SHA` or `HEAD`) |
| `target.paths_redacted` | Always `true`; raw scan paths are not emitted |
| `workflows[]` | Core `lang` / `module` / `export` phase timings |
| `analysis_workflows[]` | Optional bounded `analyze` preset timings |
| `status.ok` | `true` when all requested workflows completed |

Each `analysis_workflows[]` row records `preset`, `total_ms`, `row_count`,
`language_count`, `warning_count`, `enabled_reports`, and the bounded limits used
so comparisons remain apples-to-apples.

## Baseline capture workflow

1. Check out the baseline commit (usually `main` or the PR base).
2. Build with the same feature set you will use for the comparison run.
3. Run perf-smoke with explicit `--sha` and a dated output path, for example
   `target/perf/baseline-<sha>.json`.
4. Check out the candidate commit and rerun with the **same** flags and limits.
5. Compare `total_ms` per workflow/preset; note row counts and warning deltas.

Store receipts locally or attach them to PR discussion. Do not commit transient
`target/perf/*.json` files unless a plan explicitly owns a checked-in baseline
fixture.

## Comparison rules

Compare like with like:

- same checkout corpus (`--target-repo`);
- same `analysis-max-*` limits when timing analysis;
- same release vs debug build policy (prefer `--release` for product-facing
  claims);
- same machine class when claiming cross-machine stability.

A single local run is a **lead**, not proof of a durable improvement. Repeat on
the same machine or cite multiple receipts before claiming a measured win.

## Relationship to CI feedback

Perf-smoke measures **local workflow runtime**. It does not replace:

- `cargo xtask ci-actuals` (hosted job LEM and timing receipts);
- PR Plan LEM estimates (`docs/ci/pr-plan.md`);
- advisory fast proof-run or scoped coverage observations.

Use perf-smoke for product hot-path decisions; use CI actuals for pipeline
economics. Lane 3 optimization PRs should cite perf-smoke when they touch
scanned workflows; CI restructuring needs fresh `ci-actuals` evidence.

## Claim boundary

- **Establishes**: repeatable local timings for core scan workflows and
  optionally bounded analysis presets, with redacted paths and explicit limits.
- **Does not establish**: production SLA, cross-platform stability, CI wall-clock
  improvement, or permission to skip existing proof gates.

## See also

- [debugging.md](../debugging.md#performance-debugging) — short perf-debug entry
- [ROADMAP.md](../ROADMAP.md) — Lane 3 measured performance criteria
- [timing-refresh-2026-07.md](timing-refresh-2026-07.md) — hosted CI timing evidence and floor retuning
- [ci-actuals.md](ci-actuals.md) — hosted CI timing receipts
- [lem-budgeting.md](lem-budgeting.md) — LEM economics for PR CI
