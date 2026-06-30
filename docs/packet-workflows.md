# PR Evidence Packet Workflows

Status: CLI orchestration implemented; Action path implemented with the binary
runtime. `tokmd` has the underlying `analyze`, `context`, `syntax`, and
`evidence-packet` surfaces, and `tokmd packet generate` now coordinates them
into one `sensors/tokmd/` packet from a single command. The root
`EffortlessMetrics/tokmd` Action exposes that orchestration through
`mode: packet`, downloading a prebuilt `tokmd` binary by default. This page
defines the one-command CLI path and the `mode: packet` Action path. The GHCR
container runtime is implemented for verification-gated tags (currently
`1.14.0`); downstream `ub-review` consumption remains planned.

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
| GHCR image (`ghcr.io/effortlessmetrics/tokmd`) | Optional pinned Linux/container runtime from the publication repo only. |
| Cargo install | Local and development fallback, not the default CI path. |

GHCR is useful when a workflow needs a pinned Linux container runtime from the
publication image, but the normal user-facing entrypoint should be an Action
step, not `docker run`. Swarm workbench GHCR is verified-public for `:main` (issue #264 closed 2026-06-24) but remains a workbench/experimental runtime, not a supported consumer path.

## Local CLI

The CLI orchestration is thin:

```bash
tokmd packet generate \
  --preset bun-ub \
  --base origin/main \
  --head HEAD \
  --out sensors/tokmd \
  --syntax \
  src/runtime/api src/bun.js/bindings
```

It coordinates the existing receipt-producing commands and writes:

- `sensors/tokmd/analyze.md`;
- `sensors/tokmd/analyze.json`;
- `sensors/tokmd/context.md`;
- `sensors/tokmd/syntax.json` when syntax is requested and available;
- `sensors/tokmd/manifest.json`.

The command adds no new analysis model. It keeps the same base/head refs and
path scope across every generated artifact, runs one analysis pass rendered to
both the JSON and Markdown artifacts, then applies the existing evidence packet
status rules for `complete`, `partial`, and `failed`.

| Flag | Default | Meaning |
| --- | --- | --- |
| `--preset` | `bun-ub` | Analysis preset for `analyze.md`/`analyze.json`. |
| `--base` | `origin/main` | Base ref shared by every artifact. |
| `--head` | `HEAD` | Head ref shared by every artifact. |
| `--out` | `sensors/tokmd` | Packet output directory. |
| `--syntax` / `--no-syntax` | on | Request or skip optional `syntax.json`. |
| `--context-budget` | `64000` | Token budget for `context.md`. |

Optional syntax evidence is best-effort: when it cannot be produced the packet
degrades to `partial` with a named missing-artifact warning rather than failing.
Unresolved `--base`/`--head` refs fail the command before artifacts are written.

For UB/crash review triage using `review_signals` and manifest
`review_priority`, see [Syntax evidence guide](workflows/syntax-evidence-guide.md).

### Manual Equivalent

The orchestrator is equivalent to this manual recipe, which remains useful when
a workflow needs to customize individual steps:

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

## GitHub Action

The packet path is implemented as `mode: packet` on the root
`EffortlessMetrics/tokmd` Action:

```yaml
- uses: actions/checkout@v6
  with:
    fetch-depth: 0

- uses: EffortlessMetrics/tokmd@v1
  with:
    version: "1.14.0"
    mode: packet
    preset: bun-ub
    base: origin/main
    head: HEAD
    paths: |
      src/runtime/api
      src/bun.js/bindings
```

The Action:

- downloads the requested prebuilt `tokmd` binary by version, OS, and
  architecture, verifying release checksums when available;
- runs `tokmd packet generate` from the checkout root with the shared
  base/head refs and path scope;
- uploads the packet directory as a workflow artifact when requested;
- writes a job summary with packet status, top review priority, warnings,
  errors, artifact paths, reproduction command, and non-claims;
- exposes stable outputs for downstream jobs.

When `base` is omitted, the Action infers a repository-aware base ref the same
way `mode: cockpit` and `mode: sensor` do: pull request runs use
`origin/$GITHUB_BASE_REF`, other runs use `origin/HEAD` when available. Use
`fetch-depth: 0` so the base and head commits are present.

### Inputs

These are the `mode: packet` inputs. See the
[GitHub Action reference](github-action.md) for every shared input.

| Input | Default | Meaning |
| --- | --- | --- |
| `version` | `latest` | `tokmd` release to download. Pin it for stable workflows. |
| `preset` | `bun-ub` | Packet preset for `analyze.md` / `analyze.json`. |
| `base` | inferred | Base ref for effort delta and packet metadata. |
| `head` | `HEAD` | Head ref for effort delta and packet metadata. |
| `paths` | `.` | Newline or whitespace separated packet scope. |
| `output-dir` | `sensors/tokmd` | Packet directory. |
| `syntax` | `true` | Whether to request optional syntax evidence. |
| `context-budget` | `64000` | Token budget for `context.md`. |
| `artifact` | `true` | Upload the packet directory as a workflow artifact. |
| `fail-on` | `failed` | Failure policy: `failed`, `partial`, or `never`. |
| `runtime` | `binary` | Runtime mode: `binary` (default), or `container` (anonymous GHCR pull + mounted run; Linux+Docker only; verification-gated tags only). |
| `image` | `ghcr.io/effortlessmetrics/tokmd` | Container image reference (without tag) for `runtime: container`. The tag is derived from `version`. Only verification-gated tags are accepted; mutable tags such as `latest` are rejected. |

### Outputs

| Output | Meaning |
| --- | --- |
| `packet-status` | Packet status from `manifest.json`. |
| `packet-manifest` | Path to the packet `manifest.json`. |
| `packet-dir` | Packet output directory. |
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

GHCR is the intended secondary Linux/container runtime for the **publication
image** (`ghcr.io/effortlessmetrics/tokmd`), not the primary user experience.
The primary PR path should be a GitHub Action that downloads a prebuilt binary.
Cargo install remains the local/dev fallback. Swarm workbench GHCR is
verified-public for `:main` (issue #264 closed 2026-06-24) but remains a
workbench/experimental runtime, not a supported consumer path.

Current support status: publication GHCR is **verified-public** for `v1.13.1`
(2026-06-21) and for `v1.14.0` (2026-06-26). For `1.14.0`, the container-runtime
gate steps 6-7 in `docs/specs/packet-ghcr-runtime.md` also passed on 2026-06-26
via the `GHCR Container Smoke` lane (anonymous pull, `tokmd 1.14.0`, and a
`complete` mounted-repository packet); see `docs/releases/1.14-ledger.md` and
`docs/ci/ghcr-container-smoke.md`. The container runtime is gate-verified **and
wired** for `1.14.0`: the `runtime: container` Action path now anonymously pulls
that tag and runs it against the mounted workspace. New stable tags still need
post-release verification (gate steps 1-7) and must be added to the Action's
supported-tag set before their container runtime is called supported.

The Action's `runtime: container` path:

- requires a Linux runner with Docker available;
- accepts only verification-gated tags (currently `1.14.0`); any other tag,
  including mutable aliases such as `latest`, `1.14`, and `1`, is a hard error
  pointing at the spec;
- anonymously pulls `<image>:<normalized-version>` with an isolated docker config
  (no ambient ghcr.io credentials);
- runs `tokmd` via a wrapper that bind-mounts `$GITHUB_WORKSPACE` and runs as the
  host user so repo-relative scoped paths and base/head refs resolve identically
  to the binary runtime and written artifacts stay host-readable;
- never silently falls back to the binary runtime.

Action shape (for a verification-gated tag):

```yaml
with:
  runtime: container
  image: ghcr.io/effortlessmetrics/tokmd
  version: 1.14.0
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

If any public-pull check returns `denied` for a new stable tag, keep that tag's
GHCR marked pending and do not rewrite tags, rerun release mutation, or advertise
container runtime support as available. Fix package visibility or linkage first,
then rerun the verification checklist. This applies only to
`ghcr.io/effortlessmetrics/tokmd` from the publication repo, not swarm GHCR.

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

1. ~~Document this support model before implementation grows.~~ (done)
2. ~~Add the thin CLI orchestration command over existing receipts.~~ (done:
   `tokmd packet generate`)
3. ~~Lock packet generation status behavior with integration tests.~~ (done)
4. ~~Build the Action with binary runtime as the default.~~ (done:
   `mode: packet` on `EffortlessMetrics/tokmd`)
5. ~~Add Action examples and job-summary behavior.~~ (done: see
   [GitHub Action reference](github-action.md) and the packet job summary)
6. ~~Harden publication GHCR as a secondary runtime.~~ (done: `runtime:
   container` wired in `action.yml` for verification-gated tags, currently
   `1.14.0`.) Re-verify on each stable release and extend the supported-tag set.
7. Wire downstream `ub-review` consumption after the Action path is stable.

## Related Docs

- [Evidence packet workflow spec](specs/evidence-packet-workflow.md) — normative
  contract for layout, status semantics, support model, and verifier behavior
- [Evidence packet contract](evidence-packet.md)
- [Bun UB analysis preset](analyze/bun-ub.md)
- [ub-review tokmd sensor recipe](integrations/ub-review.md)
- [GitHub Action quickstart](action-quickstart.md)
- [GitHub Action reference](github-action.md)
