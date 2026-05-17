# Start Here

Use this page when you know the job you want `tokmd` to do, but do not want to
learn the whole control plane first.

If a workflow gives you an artifact name and you need to know what it means,
open the [Artifact glossary](artifacts.md).

If you have not installed `tokmd` yet or want the first useful local command,
open [Install and try tokmd](install-and-try.md).

If you need the shortest map from user job to command, artifact, meaning, and
next action, open [User paths](user-paths.md).

If you need to see the physical output layout before running a workflow, open
[Sample artifact trees](examples/README.md).

## 1. Tell Me What This Repo Is

Start with the smallest useful receipt:

```bash
tokmd --format md --top 8
```

Then widen the view only if you need it:

```bash
tokmd module --module-depth 2
tokmd export --format jsonl --max-rows 500 > repo-files.jsonl
tokmd analyze --preset risk --format md
```

Open the Markdown summary first. Use JSON or JSONL only when another tool needs
stable input.

More detail:

- [Tutorial](tutorial.md) for a first walkthrough.
- [Recipes](recipes.md) for analysis and export examples.
- [Artifact glossary](artifacts.md) for receipt and packet names.
- [Schema](SCHEMA.md) for receipt contracts.

## 2. Tell Me What Changed

For a quick branch comparison:

```bash
tokmd diff origin/main HEAD
```

For a saved before/after trail, write run artifacts first:

```bash
tokmd run --analysis receipt --output-dir .runs/baseline
tokmd run --analysis receipt --output-dir .runs/current
tokmd diff .runs/baseline .runs/current
```

Use this path when you need a deterministic change receipt. Use cockpit when
the next step is PR review.

More detail:

- [CLI reference](reference-cli.md) for diff inputs and output formats.
- [Recipes](recipes.md) for saved receipts and comparisons.

## 3. Help Me Review This PR

Generate a cockpit review packet:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review
```

Verify the packet before treating it as review evidence:

```bash
cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

Open these files in order:

1. `.tokmd/review/comment.md`
2. `.tokmd/review/review-map.md`
3. `.tokmd/review/evidence.json`
4. `.tokmd/review/manifest.json`
5. `target/tokmd/review-packet-check.json`

If the PR changes source-of-truth docs, plans, ADRs, templates,
`.jules/goals/**`, or doc-artifact policy, generate the documentation-control
receipt first and import it:

```bash
cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json

tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --doc-artifacts-check target/docs/doc-artifacts-check.json \
  --review-packet-dir .tokmd/review
```

The packet is review evidence, not a merge verdict. It can show missing,
degraded, stale, skipped, unavailable, required, and advisory evidence without
promoting advisory checks.

More detail:

- [Review packet contract](review-packet.md).
- [Artifact glossary](artifacts.md) for packet and verifier names.
- [tokmd in Cockpit](tokmd-in-cockpit.md).
- [Proof evidence import contract](cockpit-proof-evidence.md).

## 4. Give CI Stable Evidence And Gates

Start with artifact-producing CI before adding gates:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    mode: cockpit
    review-packet: 'true'
    artifact: 'true'
    comment: 'false'
```

Use policy gates only when you have a clear policy file and a rollback path:

```bash
tokmd run --analysis receipt --output-dir .runs/current
tokmd gate --receipt .runs/current/receipt.json --policy tokmd-gate.toml
```

For this repository's contributor workflow, affected proof planning is owned by
`xtask`:

```bash
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan.json
```

Fast proof, scoped coverage, mutation, and Codecov upload stay advisory unless a
maintainer explicitly promotes them.

More detail:

- [GitHub Action quickstart](action-quickstart.md).
- [GitHub Action reference](github-action.md).
- [Artifact glossary](artifacts.md) for proof and coverage receipts.
- [Publishing evidence](publishing-evidence.md) for release-facing package
  surface and metadata checks.
- [Coverage guidance](ci/coverage.md).
- [Proof policy](../ci/proof.toml).

## 5. Give My Coding Agent Context And Proof Expectations

For a plain bounded source bundle:

```bash
tokmd context --budget 128k --mode bundle --output context.txt
```

For a coding-agent handoff with an artifact manifest and work order:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --out-dir .handoff
```

When review or proof artifacts exist, link them instead of pasting logs:

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

Give the agent `.handoff/work-order.md` first, then `.handoff/manifest.json`.
The handoff bundle points at review and proof evidence; it does not verify those
external receipts itself.

More detail:

- [Handoff bundles](handoff.md).
- [Handoff schema](handoff-schema.md).
- [Artifact glossary](artifacts.md) for handoff, review, and proof links.
- [Agent source-of-truth workflow](agent-workflows/source-of-truth.md).

## Browser Or No-Install Evaluation

Use the browser runner when the job is safe, rootless inspection:

- language summary;
- module summary;
- file export;
- browser-safe analysis presets over uploaded or GitHub-loaded inputs.

Native filesystem flows, git-history enrichers, `cockpit`, `gate`, `sensor`,
`baseline`, `context`, and `handoff` remain native-first.

More detail:

- [Browser runner](browser.md).
- [Browser/WASM contract](architecture.md#wasm--browser-runner).
- [Browser capability matrix](capabilities/wasm.json).
