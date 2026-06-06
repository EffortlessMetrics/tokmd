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
          version: '1.12.0'
          paths: .
          artifact: 'true'
          comment: 'true'
```

## Versioning Model

There are two version choices in every workflow:

| Setting | Meaning | Example |
| :------ | :------ | :------ |
| Action ref | Which repository ref GitHub uses for `action.yml` | `EffortlessMetrics/tokmd@v1` |
| `version` input | Which released `tokmd` binary the Action downloads | `version: '1.12.0'` |

Use stable workflows like this:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.12.0'
    paths: .
```

For release-candidate smoke tests, pin both the Action ref and the downloaded binary:

```yaml
- uses: EffortlessMetrics/tokmd@v1.12.0-rc.1
  with:
    version: '1.12.0-rc.1'
    paths: .
```

If `version` does not start with `v`, the Action prepends it before downloading release assets.

| `version` input | Release tag |
| :-------------- | :---------- |
| `1.12.0` | `v1.12.0` |
| `v1.12.0` | `v1.12.0` |
| `1.12.0-rc.1` | `v1.12.0-rc.1` |

## Inputs

| Input | Required | Default | Purpose |
| :---- | :------- | :------ | :------ |
| `mode` | no | `(omitted)` | `tokmd` mode to run: `module`, `export`, `gate`, `cockpit`, `sensor`, or `baseline`. Omit it for the default module plus export flow. |
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

For `cockpit` and `sensor` in external pull request workflows, prefer full history so compare refs are available:

```yaml
- uses: actions/checkout@v6
  with:
    fetch-depth: 0

- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.12.0'
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
- unresolved `cockpit` or `sensor` base refs

`mode: gate` preserves `tokmd-gate-verdict.json` before failing when the policy verdict fails.

## Release Assets And Checksums

The Action installs `tokmd` from GitHub Release assets.

Supported binary assets:

- `tokmd-linux-amd64`
- `tokmd-linux-arm64`
- `tokmd-macos-amd64`
- `tokmd-macos-arm64`
- `tokmd-windows-amd64.exe`

When `checksums.txt` exists on the release, the Action verifies the downloaded binary before running it.

Stable release tags update the `v1` major tag. Release-candidate tags such as `v1.12.0-rc.1` are prereleases, do not become the latest release, and do not move `v1`.

## Examples

### Default Receipt

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.12.0'
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
    version: '1.12.0'
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
    version: '1.12.0'
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
    version: '1.12.0'
    mode: sensor
    head: HEAD
    artifact: 'true'
    comment: 'false'
```

### Baseline

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.12.0'
    mode: baseline
    paths: .
    artifact: 'true'
    comment: 'false'
```
