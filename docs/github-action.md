# GitHub Action Reference

The root `EffortlessMetrics/tokmd` composite Action installs a released `tokmd` binary, runs one workflow mode, and optionally uploads generated files or posts a pull request comment.

For a shorter adoption path with copy-ready receipt and review-packet workflows,
start with [GitHub Action quickstart](action-quickstart.md). This page is the
complete reference for inputs, outputs, modes, artifacts, and failure behavior.

## Quick Start

```yaml
name: tokmd receipt

on:
  pull_request:

permissions:
  contents: read
  pull-requests: write

jobs:
  receipt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: EffortlessMetrics/tokmd@v1
        with:
          version: '1.11.0'
          paths: .
          artifact: 'true'
          comment: 'true'
```

## Versioning Model

There are two version choices in every workflow:

| Setting | Meaning | Example |
| :------ | :------ | :------ |
| Action ref | Which repository ref GitHub uses for `action.yml` | `EffortlessMetrics/tokmd@v1` |
| `version` input | Which released `tokmd` binary the Action downloads | `version: '1.11.0'` |

Use stable workflows like this:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    paths: .
```

For release-candidate smoke tests, pin both the Action ref and the downloaded binary:

```yaml
- uses: EffortlessMetrics/tokmd@v1.11.0-rc.1
  with:
    version: '1.11.0-rc.1'
    paths: .
```

If `version` does not start with `v`, the Action prepends it before downloading release assets.

| `version` input | Release tag |
| :-------------- | :---------- |
| `1.11.0` | `v1.11.0` |
| `v1.11.0` | `v1.11.0` |
| `1.11.0-rc.1` | `v1.11.0-rc.1` |

## Inputs

| Input | Required | Default | Purpose |
| :---- | :------- | :------ | :------ |
| `mode` | no | `(omitted)` | `tokmd` mode to run: `module`, `export`, `gate`, `cockpit`, `sensor`, `baseline`, or `packet`. Omit it for the default module plus export flow. |
| `version` | no | `latest` | `tokmd` release to install. Use an explicit version when you want the Action ref and binary version to stay aligned. |
| `paths` | no | `.` | Paths to scan. Values are split on whitespace and passed as separate path arguments. |
| `module-roots` | no | `crates,packages` | Module root prefixes for `module`, `export`, and the default flow. |
| `top` | no | `20` | Number of rows shown in Markdown summaries. |
| `format` | no | `json` | Export receipt format for `export` and the default flow: `json`, `jsonl`, or `csv`. |
| `base` | no | `(inferred)` | Base git ref for `cockpit` and `sensor`. Explicit values are used as provided. When omitted, pull request runs use `origin/$GITHUB_BASE_REF`; other runs use `origin/HEAD` when available. |
| `head` | no | `HEAD` | Head git ref for `cockpit` and `sensor`. |
| `artifact` | no | `true` | Upload generated tokmd files as workflow artifacts. |
| `comment` | no | `true` | Post the generated Markdown summary as a pull request comment when running on `pull_request` events. |
| `review-packet` | no | `false` | For `mode: cockpit`, also emit the cockpit review packet directory. The packet-local `comment.md` remains the `summary` output; hosted pull request comments use a copied summary when metadata is added. |
| `preset` | no | `bun-ub` | For `mode: packet`, the analysis preset for `analyze.md` and `analyze.json`. |
| `output-dir` | no | `sensors/tokmd` | For `mode: packet`, the packet output directory. |
| `syntax` | no | `true` | For `mode: packet`, whether to request optional `syntax.json` evidence. |
| `context-budget` | no | `64000` | For `mode: packet`, the token budget for `context.md`. |
| `fail-on` | no | `failed` | For `mode: packet`, the packet status failure policy: `failed`, `partial`, or `never`. |
| `runtime` | no | `binary` | Runtime used to obtain `tokmd`: `binary` (default) or `container`. `container` anonymously pulls the publication GHCR image and runs it against the mounted workspace; it requires a Linux runner with Docker and only accepts verification-gated tags (currently `1.14.0`). Pin `version` to a verified tag — mutable tags such as `latest` are rejected. See `docs/specs/packet-ghcr-runtime.md`. |
| `image` | no | `ghcr.io/effortlessmetrics/tokmd` | Container image reference (without tag) for `runtime: container`. The tag is derived from `version`. Only verification-gated tags are accepted (see `docs/specs/packet-ghcr-runtime.md`). |

## Outputs

| Output | Description |
| :----- | :---------- |
| `receipt` | Path to the generated receipt file when one is produced. |
| `summary` | Path to `tokmd-summary.md`, `comment.md`, or another mode-specific Markdown summary when one is produced. |
| `gate-verdict` | Path to `tokmd-gate-verdict.json` when `mode: gate` is used. |
| `cockpit-report` | Path to `tokmd-cockpit-report.json` when `mode: cockpit` is used. |
| `review-packet` | Path to `.tokmd/review` when `mode: cockpit` and `review-packet: 'true'` are used. |
| `sensor-report` | Path to `tokmd-sensor-report.json` when `mode: sensor` is used. |
| `baseline-report` | Path to `tokmd-baseline.json` when `mode: baseline` is used. |
| `packet-status` | Evidence packet status (`complete`, `partial`, or `failed`) when `mode: packet` is used. |
| `packet-manifest` | Path to the packet `manifest.json` when `mode: packet` is used. |
| `packet-dir` | Packet output directory when `mode: packet` is used. |
| `review-priority-count` | Count of manifest `review_priority` entries when `mode: packet` is used. |
| `warnings-count` | Count of manifest warnings when `mode: packet` is used. |
| `errors-count` | Count of manifest errors when `mode: packet` is used. |
| `artifact-name` | Uploaded workflow artifact name when artifact upload is enabled. |
| `tokmd-version` | Version reported by the resolved `tokmd` runtime binary. |

## Modes

### Omitted Mode

When `mode` is omitted, the Action preserves the original workflow behavior:

- runs `tokmd module --format md`
- writes `tokmd-summary.md`
- runs `tokmd export --format <format>`
- writes `tokmd-receipt.<format>`

### `module`

Runs `tokmd module --format md` and writes `tokmd-summary.md`.

### `export`

Runs `tokmd export --format <format>` and writes `tokmd-receipt.<format>`.

Supported `format` values are `json`, `jsonl`, and `csv`.

### `gate`

Runs `tokmd gate --format json` and writes `tokmd-gate-verdict.json`.

`gate` expects policy or ratchet rules from `tokmd.toml` in the checkout. It accepts exactly one path. A failing gate still writes `tokmd-gate-verdict.json`, then the Action fails after exposing the verdict file.

### `cockpit`

Runs `tokmd cockpit --format json` and writes `tokmd-cockpit-report.json`.

If `base` is omitted, the Action infers a repository-aware base ref. Set `base` only when you want to override that inference.

When `review-packet: 'true'`, cockpit mode also runs with
`--review-packet-dir .tokmd/review`. The `review-packet` output points to that
directory, and the `summary` output points to the packet-local
`.tokmd/review/comment.md`.

When artifact upload is enabled, the Action also prepares
`tokmd-review-packet-comment.md` from `.tokmd/review/comment.md` and appends
hosted packet metadata: the workflow run URL, the `tokmd-receipts` artifact
name, and the packet path. The packet's own `comment.md` remains unchanged so
`manifest.json` hashes stay valid, while pull request comments still point to
hosted artifacts.

The Action verifies the packet after preparing the hosted comment copy and
writes `target/tokmd/review-packet-check.json`. That verifier receipt records
the checked schemas, packet-local artifact paths, and BLAKE3 hash verification
counts, and is uploaded with `tokmd-receipts` when artifact upload is enabled.
The hosted comment copy also includes a compact verifier status and proof
evidence count summary. The packet-local `.tokmd/review/comment.md` is still
not mutated.

### `sensor`

Runs `tokmd sensor --format json` and writes:

- `tokmd-sensor-report.json`
- `comment.md`
- `extras/`

The `summary` output points to `comment.md`. `sensor` uses the same base inference behavior as `cockpit`.

### `baseline`

Runs `tokmd baseline --force` and writes `tokmd-baseline.json`.

`baseline` accepts exactly one path.

### `packet`

Runs `tokmd packet generate` and writes a complete `sensors/tokmd/` evidence
packet:

- `manifest.json`
- `analyze.md`
- `analyze.json`
- `context.md`
- `syntax.json` (when `syntax: 'true'` and syntax evidence is available)

`packet` coordinates the existing `analyze`, `context`, `syntax`, and
`evidence-packet` surfaces over one shared base/head ref and path scope. It
adds no new analysis model. Set `output-dir` to write the packet somewhere
other than `sensors/tokmd`.

Like `cockpit` and `sensor`, `packet` infers a repository-aware base ref when
`base` is omitted. Use `actions/checkout` with `fetch-depth: 0` so the base and
head commits are available.

The Action reads `manifest.json` and exposes `packet-status`,
`packet-manifest`, `packet-dir`, `review-priority-count`, `warnings-count`,
`errors-count`, `artifact-name`, and `tokmd-version` outputs. It also writes a
job summary with the packet status, top review priority, warnings, errors,
artifact paths, the reproduction command, and the packet non-claims.

#### Packet status and `fail-on`

`mode: packet` separates two failure sources: the **packet status** recorded in
`manifest.json`, and **runtime errors** that prevent a manifest from being
written. The `fail-on` input governs only the first; runtime errors always fail
the job.

`tokmd packet generate` assigns one of three statuses (see
[Evidence packet workflow spec](specs/evidence-packet-workflow.md#status-semantics)
for the authoritative rules):

| Status | What it means |
| :----- | :------------ |
| `complete` | All required artifacts exist, base/head refs resolved, and no warnings or errors were recorded. |
| `partial` | Required artifacts exist, but non-fatal warnings bound the evidence — for example optional `syntax.json` is missing, malformed, or degraded. The manifest still records named warnings. |
| `failed` | A required artifact is missing, refs did not resolve, `analyze.json` could not be parsed, or the preset/paths did not match the requested scope. The manifest is still written with named errors. |

The Action maps each status to a job outcome through `fail-on`. The table also
shows the GitHub log annotation the Action emits so you can recognize each case
in CI logs:

| Packet status | `fail-on: failed` (default) | `fail-on: partial` | `fail-on: never` |
| :------------ | :-------------------------- | :----------------- | :--------------- |
| `complete` | Pass | Pass | Pass |
| `partial` | Pass | **Fail** — `::error::...status is 'partial' and fail-on=partial` | Pass |
| `failed` | **Fail** — `::error::...status is 'failed'` | **Fail** — `::error::...status is 'failed'` | Pass with `::warning::...fail-on=never; not failing the job` |
| unexpected | Pass with `::warning::Unexpected evidence packet status` | Pass with `::warning::` | Pass with `::warning::` |

Read the three consumer-facing situations this way:

- **Advisory evidence missing (does not fail).** Optional syntax evidence is
  best-effort. When `syntax.json` cannot be produced, the packet degrades to
  `partial` with a named warning rather than failing. Under the default
  `fail-on: failed`, a `partial` packet still passes; the warning is a signal,
  not a gate. Set `fail-on: partial` only when your workflow treats degraded
  evidence as blocking.
- **Partial packet (passes by default).** Same as above for any non-fatal
  warning that bounds the evidence. The manifest, `warnings-count`, and
  `packet-status` outputs let downstream jobs decide whether to act on it.
- **Action fails.** A `failed` status fails the job under the default and
  `fail-on: partial`; only `fail-on: never` downgrades it to a warning. On top
  of that, a **runtime error** — an invalid `fail-on`/`syntax` value, an
  unresolved `base`/`head` ref, or `tokmd packet generate` exiting before it
  writes `manifest.json` — fails the job regardless of `fail-on`, because there
  is no trustworthy packet status to honor.

`fail-on: never` is the most permissive setting: it never fails the job for a
packet status, but it does not suppress runtime errors. Use it when you want the
packet purely as advisory evidence and prefer to gate on the `packet-status`,
`warnings-count`, or `errors-count` outputs in a later step.

For the full packet workflow model, including the GHCR container runtime, see
[PR evidence packet workflows](packet-workflows.md).

## Artifacts

When `artifact: 'true'`, generated files are uploaded as a workflow artifact.

Artifact candidates include:

- `tokmd-summary.md`
- `tokmd-receipt.*`
- `tokmd-gate-verdict.json`
- `tokmd-cockpit-report.json`
- `tokmd-review-packet-comment.md`
- `target/tokmd/review-packet-check.json`
- `.tokmd/review`
- `tokmd-sensor-report.json`
- `tokmd-baseline.json`
- `sensors/tokmd/` (or the configured `output-dir` for `mode: packet`)
- `comment.md`
- `extras/`

## PR Comments

Pull request comments require:

```yaml
permissions:
  contents: read
  pull-requests: write
```

Commenting only runs on `pull_request` events. Set `comment: 'false'` for scheduled jobs, push jobs, private smoke tests, or workflows where comments are not desired.

### Fork pull requests

On fork pull requests the default `GITHUB_TOKEN` is read-only for security, even
when the base workflow declares `pull-requests: write`. Hosted comment posting
can therefore be rejected for a fork PR while receipt, packet, and artifact
generation still succeed. A missing hosted comment on a fork PR is not a packet
failure: treat the uploaded `tokmd-receipts` artifact (and, for cockpit review
packets, the packet-local `.tokmd/review/` directory) as the source of truth.

If a repository takes many fork contributions, prefer one of these fork-safe
patterns over relying on the hosted comment:

- keep `comment: 'false'` and let reviewers read the uploaded artifact, or
- run generation on the `pull_request` event with `artifact: 'true'`, then post
  the summary from a separate `workflow_run` job that has write access.

Do not switch the generating job to `pull_request_target` only to expose a write
token to fork code; that reintroduces the fork-secret exposure the read-only
token is designed to prevent.

The default flow comments with `tokmd-summary.md`. `sensor` comments with `comment.md`. JSON-only modes such as `gate`, `cockpit`, and `baseline` normally leave the `summary` output empty. `cockpit` with `review-packet: 'true'` comments with the packet summary, using `tokmd-review-packet-comment.md` when hosted packet metadata is added.

For cockpit review packets, the Action copies `.tokmd/review/comment.md` to
`tokmd-review-packet-comment.md` and appends a short hosted-packet block before
posting the pull request comment. With `artifact: 'true'`, the block points
reviewers to the workflow run and `tokmd-receipts` artifact that contains the
full `.tokmd/review/` directory. With artifact upload disabled, the comment
states that the packet was generated locally in the workflow workspace but not
uploaded. After packet verification, the hosted copy also shows whether the
packet was verified, whether manifest hashes were valid, and compact proof
evidence counts. The packet-local `comment.md` is not mutated after generation.

## Checkout Guidance

The default, `module`, `export`, `gate`, and `baseline` modes can usually use a normal checkout:

```yaml
- uses: actions/checkout@v6
```

For `cockpit`, `sensor`, and `packet` in external pull request workflows, prefer full history so compare refs are available:

```yaml
- uses: actions/checkout@v6
  with:
    fetch-depth: 0

- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    mode: cockpit
    head: HEAD
    artifact: 'true'
    comment: 'false'
```

## Multi-Path Behavior

Multiple scan paths can be passed on one line:

```yaml
paths: "src crates"
```

or as a multiline value:

```yaml
paths: |
  src
  packages
```

`gate` and `baseline` accept exactly one path. Same-line or multiline multi-path inputs fail before `tokmd` runs.

## Base And Head Inference

`cockpit` and `sensor` compare a base ref and a head ref.

When `base` is omitted:

- pull request runs use `origin/$GITHUB_BASE_REF`
- other runs use `origin/HEAD` when available

Set `base` explicitly only when you want to override inference:

```yaml
with:
  mode: sensor
  base: origin/main
  head: HEAD
```

The default `head` is `HEAD`.

## Failure Behavior

The Action fails early for:

- unsupported modes
- unsupported runner architectures
- unresolved release assets
- checksum mismatches
- invalid `gate` or `baseline` path counts
- unresolved `cockpit`, `sensor`, or `packet` base refs
- an invalid `mode: packet` `fail-on` or `syntax` value
- `runtime: container` with a non-verified or mutable tag (such as `latest`), on a non-Linux runner, or without Docker — these fail with a spec-aligned error naming the resolved image reference (see `docs/specs/packet-ghcr-runtime.md`)

`mode: gate` preserves `tokmd-gate-verdict.json` before failing when the policy verdict fails.

`mode: packet` maps packet status to failure through `fail-on` (`failed` by
default). A packet that fails before writing `manifest.json` is a runtime error
and fails regardless of `fail-on`.

## Release Assets And Checksums

The Action installs `tokmd` from GitHub Release assets.

Supported binary assets:

- `tokmd-linux-amd64`
- `tokmd-linux-arm64`
- `tokmd-macos-amd64`
- `tokmd-macos-arm64`
- `tokmd-windows-amd64.exe`

When `checksums.txt` exists on the release, the Action verifies the downloaded binary before running it.

Stable release tags update the `v1` major tag. Release-candidate tags such as `v1.11.0-rc.1` are prereleases, do not become the latest release, and do not move `v1`.

## Examples

### Default Receipt

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    paths: .
    artifact: 'true'
    comment: 'true'
```

### Gate

```yaml
- name: Write gate policy
  run: |
    cat > tokmd.toml <<'TOML'
    [[gate.rules]]
    name = "has_files"
    pointer = "/derived/totals/files"
    op = "gte"
    value = 1
    TOML

- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    mode: gate
    paths: .
    artifact: 'true'
    comment: 'false'
```

### Cockpit

```yaml
- uses: actions/checkout@v6
  with:
    fetch-depth: 0

- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    mode: cockpit
    head: HEAD
    artifact: 'true'
    comment: 'false'
```

### Sensor

```yaml
- uses: actions/checkout@v6
  with:
    fetch-depth: 0

- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    mode: sensor
    head: HEAD
    artifact: 'true'
    comment: 'false'
```

### Baseline

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.11.0'
    mode: baseline
    paths: .
    artifact: 'true'
    comment: 'false'
```

### Packet

```yaml
- uses: actions/checkout@v6
  with:
    fetch-depth: 0

- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.14.0'
    mode: packet
    preset: bun-ub
    base: origin/main
    head: HEAD
    paths: |
      src/runtime/api
      src/bun.js/bindings
    fail-on: failed
    artifact: 'true'
    comment: 'false'
```
