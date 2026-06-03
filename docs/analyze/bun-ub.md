# Bun UB Analysis Preset

Status: implemented. `tokmd analyze --preset bun-ub` is a native, git-aware
analysis preset for producing scoped review evidence for Bun undefined-behavior
burndown work.

## Purpose

`bun-ub` packages on-diff evidence for native-boundary review. It is intended
for review bots, local reviewers, and coding agents that need a deterministic
packet over changed paths, base/head effort delta, import/API/complexity
signals, and enough context to decide what to inspect first.

It is not a custom undefined-behavior detector. It does not prove that UB exists
or is absent. It makes the review evidence reproducible and scoped.

## Local Usage

Run from a Bun checkout or fork with explicit refs and explicit changed paths:

```bash
tokmd analyze \
  --preset bun-ub \
  --format md \
  --effort-base-ref BASE \
  --effort-head-ref HEAD \
  --no-progress \
  <changed paths>
```

For machine-readable output:

```bash
tokmd analyze \
  --preset bun-ub \
  --format json \
  --effort-base-ref BASE \
  --effort-head-ref HEAD \
  --no-progress \
  <changed paths>
```

Use a directory when the review scope is a surface, for example
`src/runtime/api`. Use specific files when the review scope is a smaller diff.
`tokmd analyze <path>` keeps file-backed enrichers inside the requested input
scope; use `tokmd analyze .` only when a whole-repo analysis is intended.

## Review Bot Artifacts

A review bot can write the Markdown and JSON receipts as stable sensor
artifacts:

```bash
mkdir -p sensors/tokmd

tokmd analyze \
  --preset bun-ub \
  --format md \
  --effort-base-ref BASE \
  --effort-head-ref HEAD \
  --no-progress \
  <changed paths> \
  > sensors/tokmd/analyze.md

tokmd analyze \
  --preset bun-ub \
  --format json \
  --effort-base-ref BASE \
  --effort-head-ref HEAD \
  --no-progress \
  <changed paths> \
  > sensors/tokmd/analyze.json
```

Open `sensors/tokmd/analyze.md` first for the human review summary. Use
`sensors/tokmd/analyze.json` for bot ingestion, ledger storage, and exact field
checks.

## Required Refs

For review-bot evidence, pass both refs explicitly:

```bash
--effort-base-ref BASE --effort-head-ref HEAD
```

If an explicit base or head ref cannot be resolved, `tokmd analyze` fails and
names the bad ref. Treat that as invalid evidence, not as a weaker successful
receipt. A bot should not attach `bun-ub` artifacts when the diff window cannot
be resolved.

## What The Preset Includes

The first `bun-ub` version is a thin preset contract over existing analysis
surfaces:

| Signal | Purpose |
| --- | --- |
| Effort estimate and delta | Show the size and blast change between base and head. |
| Git and churn signals | Highlight files or modules with change history. |
| Imports | Show dependency and module-boundary movement inside the requested scope. |
| Complexity | Surface files that deserve reviewer attention first. |
| API surface | Expose public/native-boundary-shaped declarations when available. |
| Duplicate signals | Flag repeated source shapes that can hide review risk. |

The preset intentionally avoids supply-chain, license, asset, dependency
lockfile, novelty, and whole-repo deep analysis signals.

## Expected Markdown Sections

The Markdown output is intended to be readable without downloading additional
artifacts. Useful review-bot sections include:

- `Effort estimate`
- `Delta`
- `Top offenders`
- `Doc density by language`
- `Integrity`

Use these sections as review evidence. Do not treat them as a merge verdict or
as proof that a native-boundary change is safe.

## Path Scope

The positional inputs are the analysis scope:

```bash
tokmd analyze --preset bun-ub src/runtime/api
tokmd analyze --preset bun-ub src/runtime/api/MarkdownObject.rs
```

File-backed enrichers walk only those requested paths, rebased under the
checkout root. Unrelated fixtures, including dangling symlinks outside the
requested scope, should not degrade a scoped analysis.

## Context For Handoff

Pair `bun-ub` analysis with a bounded context bundle when a reviewer or agent
needs source text:

```bash
tokmd context \
  --budget 64000 \
  <changed paths>
```

The default context list shows charged tokens, full-file tokens, inclusion
policy, and code lines so reviewers can see what was included, truncated, or
skipped before handing the bundle to an agent.

For a Bun native-boundary handoff, pass the same changed paths used for
analysis. Common surfaces are:

```bash
tokmd context \
  --budget 64000 \
  src/runtime/api \
  src/bun.js/bindings \
  src/bun.js/api
```

Open the default list output first. It is the audit view for the context pack:

| Column | Meaning |
| --- | --- |
| `Used` | Tokens charged against the handoff budget. |
| `Tokens` | Full-file token estimate before truncation or policy. |
| `Policy` | `full`, `head+tail`, `summary`, `skipped`, or a policy reason. |
| `Code` | Code lines in the selected file. |

If `Used` is lower than `Tokens`, the bundle is not full-file evidence for that
row. If `Policy` reports `head+tail`, tell the agent that middle content was
omitted. If a file is skipped by policy, the handoff should name that gap rather
than imply the source text was included.

When the handoff needs source text instead of the audit list, use bundle output
after checking the list:

```bash
tokmd context \
  --budget 64000 \
  --mode bundle \
  --output sensors/tokmd/context.txt \
  <changed paths>
```

For bot or ledger ingestion, emit JSON instead of the list:

```bash
tokmd context \
  --budget 64000 \
  --mode json \
  <changed paths> \
  > sensors/tokmd/context.json
```

## Failure Modes

| Failure | Meaning | Next action |
| --- | --- | --- |
| Bad base/head ref | The requested diff window did not resolve. | Fix the ref and rerun before attaching artifacts. |
| Missing git history | Git-backed delta or churn evidence cannot be computed. | Run in a checkout with the needed commits or record the gap. |
| Empty scoped input | The requested paths did not yield analyzable files. | Check the changed-path list or ignore rules. |
| Partial evidence warning | A scoped file could not be read or enriched. | Inspect the named path; do not generalize the receipt beyond successful inputs. |

## Fallback

If the Bun-specific contract is not needed, use the generic effort preset:

```bash
tokmd analyze \
  --preset estimate \
  --format md \
  --effort-base-ref BASE \
  --effort-head-ref HEAD \
  --no-progress \
  <changed paths>
```

`estimate` is weaker for Bun UB review because it does not request the same
import, churn, complexity, API-surface, and duplicate signals.

## Non-Claims

`bun-ub` does not:

- detect undefined behavior;
- prove memory safety;
- inspect the entire Bun repository unless `.` is supplied;
- replace cockpit review packets;
- execute CI proof;
- promote advisory proof, coverage, mutation, or release checks;
- change release, publish, signing, tag, or package behavior.
