# Browser To Native

Use this path after a browser trial when the next job needs native `tokmd`,
CI, or a coding-agent handoff.

Browser mode is a no-install inspection lens. Native mode is the review, proof,
and handoff instrument.

## Start In Browser

Run first: open the browser runner and load a public GitHub repository, local
files, or a local directory.

Open first: the browser UI summary.

Download when you need a saved trail:

```text
tokmd-browser-summary.md
tokmd-browser-receipt.json
```

The browser receipt is useful supporting evidence for an inspection. It is not
CI proof, a PR review packet, a policy gate, or an agent handoff.

## What Browser Mode Can Prove

Browser mode can show:

- language, module, and file inventory for browser-loaded inputs;
- browser-safe analysis presets exposed by the loaded WASM capability matrix;
- deterministic downloadable receipts for that browser run.

This is enough to answer:

```text
What is in this repo or file set?
Is it worth installing native tokmd for review, proof, or handoff work?
```

## What Requires Native Mode

Move to native `tokmd` when the next job needs:

- PR review packets from `tokmd cockpit`;
- review-packet verification from `cargo xtask review-packet-check`;
- git-history enrichers such as churn, hotspots, or freshness;
- policy gates and baselines;
- source context bundles or `.handoff/` directories;
- proof planning, proof observations, or publishing evidence.

Browser receipts can support those workflows, but they do not replace the
native artifacts.

## Move To PR Review

Run from a native checkout:

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

This packet tells you what to review first, what evidence is present or
missing, and which commands reproduce the evidence. It is not a merge verdict.

## Move To Agent Handoff

Run from a native checkout:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --out-dir .handoff
```

When review and proof artifacts exist, link them:

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
2. `.handoff/code.txt`
3. `.handoff/review-links.json` and `.handoff/proof-links.json`, when present

The handoff tells an agent what source bundle it has, what evidence is linked,
what proof is expected, and when to stop. It does not verify the linked review
or proof artifacts itself.

## Move To CI Evidence

Use the GitHub Action when you want hosted artifacts:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    mode: cockpit
    review-packet: 'true'
    artifact: 'true'
    comment: 'false'
```

Use contributor proof planning when you are working inside this repository:

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

Proof planning tells you what should run. It does not mean proof has executed.
Advisory fast proof, scoped coverage, mutation, and Codecov upload stay
advisory unless policy explicitly promotes them.

## Reading Order

Use this order when moving from browser to native:

1. Browser UI summary: decide whether the repo or file set is worth deeper
   native inspection.
2. `tokmd-browser-summary.md`: save the browser-safe human summary if needed.
3. `tokmd-browser-receipt.json`: save machine-readable browser evidence if a
   later workflow needs it.
4. `.tokmd/review/review-map.md`: review PR work in native mode.
5. `.handoff/work-order.md`: give a coding agent the bounded work order.
6. `target/proof/affected.json` and `target/proof/proof-plan.json`: read proof
   routing and required/advisory expectations.

## Boundaries

- Browser mode does not claim native filesystem behavior.
- Browser mode does not produce cockpit packets, gates, handoff bundles, or
  proof receipts.
- Browser mode does not make AST-backed capability claims.
- Native cockpit and handoff artifacts are review and agent evidence, not merge
  verdicts.
- CI proof and coverage artifacts remain advisory unless policy explicitly
  promotes them.
