# PR Evidence Packet Workflows

Status: planned workflow contract. `tokmd` already has the underlying
`analyze`, `context`, `syntax`, and `evidence-packet` surfaces. This page
defines the one-command CLI and GitHub Action user paths for the next workflow
lane without claiming that the orchestration command or dedicated Action already
exist.

## Purpose

`tokmd` should be easy to run inside pull request workflows and from a local
checkout to produce one bounded, reproducible evidence packet:

```text
sensors/tokmd/
  manifest.json
  analyze.md
  analyze.json
  context.md
  syntax.json
```

The packet should answer:

- what changed;
- what paths were in scope;
- what evidence was produced;
- what evidence degraded or failed;
- what to inspect first;
- what context was included, truncated, or skipped;
- how to reproduce the packet;
- what `tokmd` explicitly does not claim.

The packet is a review optic. It is not a verifier, UB detector, CI
replacement, or merge verdict.

## Support Model

For non-local usage, prefer the hosted workflow path. Users should not need to
build `tokmd` in every repository.

| Path | Role |
| --- | --- |
| GitHub Action | Default pull request workflow UX. |
| Prebuilt binary | Fast default runtime for the Action. |
| GHCR image | Optional pinned Linux/container runtime. |
| Cargo install | Local and development fallback, not the default CI path. |

GHCR is useful when a workflow needs a pinned Linux container runtime, but the
normal user-facing entrypoint should be an Action step, not `docker run`.

## Target Local CLI

The planned CLI orchestration should be thin:

```bash
tokmd packet generate \
  --preset bun-ub \
  --base origin/main \
  --head HEAD \
  --out sensors/tokmd \
  --syntax \
  src/runtime/api src/bun.js/bindings
```

It should coordinate the existing receipt-producing commands and write:

- `sensors/tokmd/analyze.md`;
- `sensors/tokmd/analyze.json`;
- `sensors/tokmd/context.md`;
- `sensors/tokmd/syntax.json` when syntax is requested and available;
- `sensors/tokmd/manifest.json`.

The command should not add a new analysis model. It should keep the same
base/head refs and path scope across every generated artifact, then use the
existing evidence packet status rules for `complete`, `partial`, and `failed`.

### Current Manual Equivalent

Until the orchestration command exists, use the manual recipe:

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
  src/runtime/api src/bun.js/bindings \
  > sensors/tokmd/analyze.md

tokmd analyze \
  --preset bun-ub \
  --format json \
  --effort-base-ref "$BASE" \
  --effort-head-ref "$HEAD" \
  --no-progress \
  src/runtime/api src/bun.js/bindings \
  > sensors/tokmd/analyze.json

tokmd context \
  --budget 64000 \
  src/runtime/api src/bun.js/bindings \
  > sensors/tokmd/context.md

tokmd syntax \
  --no-progress \
  src/runtime/api src/bun.js/bindings \
  > sensors/tokmd/syntax.json

tokmd evidence-packet \
  --preset bun-ub \
  --base "$BASE" \
  --head "$HEAD" \
  src/runtime/api src/bun.js/bindings
```

## Target GitHub Action

The planned Action path should look like this:

```yaml
- uses: actions/checkout@v6
  with:
    fetch-depth: 0

- uses: EffortlessMetrics/tokmd-action@v1
  with:
    version: "1.13.1"
    preset: bun-ub
    base: origin/main
    head: HEAD
    paths: |
      src/runtime/api
      src/bun.js/bindings
```

The Action should:

- download and cache the requested prebuilt `tokmd` binary by version, OS, and
  architecture;
- run the packet generation command from the checkout root;
- upload `sensors/tokmd/` as a workflow artifact when requested;
- write a job summary with packet status, top review priority, warnings,
  errors, artifact paths, reproduction command, and non-claims;
- expose stable outputs for downstream jobs.

### Inputs

| Input | Default | Meaning |
| --- | --- | --- |
| `version` | required for stable workflows | `tokmd` version to download or run. |
| `preset` | `bun-ub` | Packet preset. |
| `base` | workflow-defined | Base ref for effort delta and packet metadata. |
| `head` | `HEAD` | Head ref for effort delta and packet metadata. |
| `paths` | required | Newline or whitespace separated packet scope. |
| `output-dir` | `sensors/tokmd` | Packet directory. |
| `syntax` | `true` | Whether to request optional syntax evidence. |
| `context-budget` | `64000` | Token budget for `context.md`. |
| `upload-artifact` | `true` | Upload the packet directory. |
| `fail-on` | `failed` | Failure policy: `failed`, `partial`, or `never`. |
| `runtime` | `binary` | Runtime mode: `binary` or `container`. |

### Outputs

| Output | Meaning |
| --- | --- |
| `status` | Packet status from `manifest.json`. |
| `manifest-path` | Path to `sensors/tokmd/manifest.json`. |
| `artifact-name` | Uploaded artifact name when artifact upload is enabled. |
| `review-priority-count` | Count of manifest `review_priority` entries. |
| `warnings-count` | Count of manifest warnings. |
| `errors-count` | Count of manifest errors. |
| `tokmd-version` | Version reported by the runtime binary. |

## Failure Policy

The Action should make packet status explicit and map it to workflow failure
through `fail-on`:

| `fail-on` | Behavior |
| --- | --- |
| `failed` | Fail only when packet status is `failed`. |
| `partial` | Fail when packet status is `partial` or `failed`. |
| `never` | Never fail only because of packet status; still fail on Action/runtime errors. |

Bad explicit refs should produce a failed packet or nonzero command. Missing
required artifacts should fail. Optional syntax degradation should produce a
partial packet with named warnings unless the workflow explicitly makes syntax
required in a later contract.

## GHCR Runtime

Container runtime should be optional:

```yaml
with:
  runtime: container
  image: ghcr.io/effortlessmetrics/tokmd:1.13.1
```

The image should include:

- `tokmd`;
- `git`;
- CA certificates;
- sensible working-directory behavior;
- `ENTRYPOINT ["tokmd"]`;
- OCI source, description, license, and version labels.

Release verification for GHCR must distinguish push success from public
consumer visibility. A release gate should verify:

- the image was pushed;
- expected tags exist;
- the package is public;
- anonymous pull works;
- the container reports the expected `tokmd --version`;
- the container can generate a packet against a mounted repository.

## Non-Claims

A packet workflow does not:

- prove undefined behavior exists or is absent;
- prove public reachability;
- prove memory safety;
- replace human review;
- replace CI, fuzzing, Miri, mutation, coverage, or release proof;
- decide merge readiness;
- promote advisory proof or Codecov upload by default.

## Implementation Order

1. Document this support model before implementation grows.
2. Add the thin CLI orchestration command over existing receipts.
3. Lock packet generation status behavior with integration tests.
4. Build the Action with binary runtime as the default.
5. Add Action examples and job-summary behavior.
6. Harden GHCR as a secondary runtime with public-pull verification.
7. Wire downstream `ub-review` consumption after the Action path is stable.

## Related Docs

- [Evidence packet contract](evidence-packet.md)
- [Bun UB analysis preset](analyze/bun-ub.md)
- [ub-review tokmd sensor recipe](integrations/ub-review.md)
- [GitHub Action quickstart](action-quickstart.md)
- [GitHub Action reference](github-action.md)
