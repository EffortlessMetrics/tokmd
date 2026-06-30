# ub-review tokmd Sensor Recipe

Status: ready for review-bot and local use. This recipe shows how an
`ub-review` lane can attach scoped `tokmd` evidence without inventing a custom
undefined-behavior detector.

## Purpose

Use `tokmd` as a bounded evidence sensor for high-risk native-boundary review.
The artifacts answer what changed, whether the requested base/head refs
resolved, where effort and review risk concentrate, and what source context was
included, truncated, or skipped.

`tokmd` does not decide whether undefined behavior exists or is absent. It
packages reproducible review evidence for the bot, reviewer, and next agent.

## Inputs

Run the recipe from the repository under review, usually a Bun checkout or
fork. Pass the changed paths as positional arguments.

Set explicit refs:

```bash
BASE="${BASE:-origin/main}"
HEAD="${HEAD:-HEAD}"
```

The refs are part of the evidence contract. If either explicit ref does not
resolve, `tokmd analyze --preset bun-ub` fails and names the bad ref. A bot
should treat that as invalid evidence and should not attach a valid-looking
artifact.

## Review-Bot Recipe

```bash
BASE="${BASE:-origin/main}"
HEAD="${HEAD:-HEAD}"

mkdir -p sensors/tokmd
rm -f sensors/tokmd/syntax.json

tokmd analyze \
  --preset bun-ub \
  --format md \
  --effort-base-ref "$BASE" \
  --effort-head-ref "$HEAD" \
  --no-progress \
  "$@" \
  > sensors/tokmd/analyze.md

tokmd analyze \
  --preset bun-ub \
  --format json \
  --effort-base-ref "$BASE" \
  --effort-head-ref "$HEAD" \
  --no-progress \
  "$@" \
  > sensors/tokmd/analyze.json

tokmd context \
  --budget 64000 \
  "$@" \
  > sensors/tokmd/context.md

if tokmd syntax --help >/dev/null 2>&1; then
  tokmd syntax \
    --no-progress \
    "$@" \
    > sensors/tokmd/syntax.json
fi

tokmd evidence-packet \
  --preset bun-ub \
  --base "$BASE" \
  --head "$HEAD" \
  "$@"
```

Attach these artifacts:

| Artifact | Consumer | Use |
| --- | --- | --- |
| `sensors/tokmd/manifest.json` | bot, ledger, agent | Packet index with refs, paths, status, artifact paths, warnings, errors, non-claims, and reproduction commands. |
| `sensors/tokmd/analyze.md` | reviewer | First-read human summary of scoped risk evidence. |
| `sensors/tokmd/analyze.json` | bot, ledger, agent | Machine-readable receipt with preset, refs, warnings, and signals. |
| `sensors/tokmd/context.md` | reviewer, agent | Context budget audit showing included, truncated, and skipped files. |
| `sensors/tokmd/syntax.json` | reviewer, bot, agent | Optional advisory parser evidence and review signals for syntax-backed priority. |

Use the [evidence packet contract](../evidence-packet.md) for
`sensors/tokmd/manifest.json`. `tokmd evidence-packet` writes that manifest
from the same `BASE`, `HEAD`, and changed paths used to generate the receipts.
It exits nonzero for failed packets while leaving the manifest on disk for
inspection.

On Windows PowerShell, prefer tokmd output flags where available or explicit
UTF-8 file writes for redirected JSON. `tokmd evidence-packet` rejects
non-UTF-8 JSON artifacts instead of silently indexing them.

`tokmd syntax` is optional and requires a binary built with the `ast` feature.
When syntax is unavailable, omit `sensors/tokmd/syntax.json`; the manifest still
indexes the required `analyze` and `context` artifacts.

## Local Reviewer Recipe

Use the same command shape locally so bot artifacts and reviewer artifacts are
reproducible:

```bash
BASE=origin/main
HEAD=HEAD

tokmd analyze \
  --preset bun-ub \
  --format md \
  --effort-base-ref "$BASE" \
  --effort-head-ref "$HEAD" \
  --no-progress \
  src/runtime/api \
  > sensors/tokmd/analyze.md

tokmd context \
  --budget 64000 \
  src/runtime/api \
  > sensors/tokmd/context.md
```

Use the actual changed paths instead of `src/runtime/api` when the review scope
is narrower.

## Expected Behavior

- Analysis stays scoped to the paths passed at the end of the command.
- Bad explicit refs fail clearly instead of producing a generic baseline gap.
- `bun-ub` includes effort delta, git/churn, imports, complexity, API surface,
  and duplication signals.
- `bun-ub` excludes supply-chain, license, asset, dependency, novelty, and
  whole-repo deep scans.
- Context output shows charged tokens, full-file tokens, policy, and code lines
  so the budget can be reconciled from the rows.
- Optional syntax output can add parser-backed review signals and
  `review_priority` entries to the packet manifest.

For step-by-step triage using `review_signals`, see
[syntax-evidence-guide.md](../workflows/syntax-evidence-guide.md).

## Fallback

If `bun-ub` is unavailable in an older `tokmd` binary, the weaker generic
fallback is:

```bash
tokmd analyze \
  --preset estimate \
  --format md \
  --effort-base-ref "$BASE" \
  --effort-head-ref "$HEAD" \
  --no-progress \
  "$@"
```

Use this only as generic effort evidence. It does not request the same import,
churn, complexity, API-surface, or duplication signals as `bun-ub`.

## Non-Claims

This recipe does not:

- prove that undefined behavior exists or is absent;
- replace a reviewer;
- run CI proof;
- promote coverage, mutation, fuzz, release, signing, or publish lanes;
- require whole-repo analysis unless `.` is passed as the changed scope.
- prove public reachability or guard adequacy from syntax signals.
