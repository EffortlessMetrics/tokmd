# Copy-Ready Workflows

Use this page after choosing a path from [User Paths](user-paths.md). These are
short command sequences for common jobs. They do not add new behavior; they
compose existing `tokmd` and `xtask` surfaces.

## Inspect A Repository

Run:

```bash
tokmd --format md --top 8
tokmd analyze --preset risk --format md
```

Open first: terminal output.

This gives a quick repo shape, then a risk-oriented analysis pass. It does not
run tests, review a PR, or prove release readiness.

## Review A PR

Run:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review

cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

Open first:

1. `.tokmd/review/review-map.md`
2. `.tokmd/review/comment.md`
3. `.tokmd/review/evidence.json`
4. `target/tokmd/review-packet-check.json`

This gives a review work order, packet summary, evidence state, and packet
verifier receipt. It is not a merge verdict.

## Plan CI Proof Evidence

Run:

```bash
cargo xtask affected \
  --base origin/main \
  --head HEAD \
  --json-output target/proof/affected.json

cargo xtask proof \
  --profile affected \
  --base origin/main \
  --head HEAD \
  --plan \
  --plan-json target/proof/proof-plan.json \
  --evidence-json target/proof/proof-evidence.json
```

Open first:

1. `target/proof/affected.json`
2. `target/proof/proof-plan.json`
3. `target/proof/proof-evidence.json`

This tells you which files changed, which proof scopes matched, and which
commands are required or advisory. It does not execute the planned proof.

## Summarize Proof Observations

Run when observation artifacts already exist:

```bash
cargo xtask proof-observation-status \
  --observations-dir target/proof-observations \
  --json target/proof-observations/proof-observation-decision.json \
  --summary-md target/proof-observations/proof-observation-decision.md

cargo xtask proof-observation-status-check \
  --decision target/proof-observations/proof-observation-decision.json \
  --json target/proof-observations/proof-observation-decision-check.json
```

Open first:

1. `target/proof-observations/proof-observation-decision.md`
2. `target/proof-observations/proof-observation-decision.json`
3. `target/proof-observations/proof-observation-decision-check.json`

This summarizes observed proof status and promotion criteria. It does not
promote proof, enable Codecov upload, or make advisory proof required.

## Prepare A Coding-Agent Handoff

Run after generating review and proof artifacts:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --review-packet-dir .tokmd/review \
  --review-packet-check target/tokmd/review-packet-check.json \
  --affected target/proof/affected.json \
  --proof-plan target/proof/proof-plan.json \
  --out-dir .handoff
```

Open first:

1. `.handoff/work-order.md`
2. `.handoff/manifest.json`
3. `.handoff/code.txt`
4. `.handoff/review-links.json`
5. `.handoff/proof-links.json`

Give the agent `work-order.md` first. Treat missing, stale, degraded, or
unavailable evidence as work to resolve, not as passing proof.

## Try Browser Mode, Then Move Native

Run: open the browser runner, load a GitHub repo or local files, and download
the browser-safe receipt.

Open first: the browser UI summary, then the downloaded receipt.

Next native command for real PR review:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review
```

Browser mode is a no-install trial lens. It does not provide native filesystem
access, git-history enrichers, cockpit packets, gates, context bundles, handoff
bundles, or AST capability claims.

## Check Publishing Evidence

Run:

```bash
cargo xtask publish-surface --json --verify-publish
cargo xtask version-consistency
```

Open first: publish-surface output or saved JSON, then version-consistency
output.

This checks package-surface and version-readiness evidence. It does not publish
crates, create tags, create GitHub releases, or approve release mutation.

For release-facing files, pair this with affected proof planning as shown in
[Release readiness](release-readiness.md).
