# Review Packet Contract

Status: initial implementation. `tokmd cockpit --review-packet-dir <dir>`
emits the first packet slice without changing the existing default
`tokmd cockpit` stdout behavior or the shipped `--artifacts-dir` contract.

## Purpose

The review packet is a stable artifact directory for PR review evidence. It
lets a maintainer inspect what changed, what evidence is available, what is
missing or degraded, and which files deserve attention first.

`tokmd cockpit` remains the current PR-review evidence surface. A future
`tokmd review` command should not be introduced unless it becomes a distinct
orchestrator over this packet instead of duplicating cockpit computation.

## Existing Cockpit Artifacts

`tokmd cockpit --artifacts-dir <dir>` writes:

```text
<dir>/
  cockpit.json
  report.json
  comment.md
```

Those artifacts remain the shipped cockpit-director contract. Sensor mode
continues to use the `sensor.report.v1` envelope and its documented sidecars.

`tokmd cockpit --review-packet-dir <dir>` writes the packet-shaped PR-review
artifacts documented below. It is an additive output option.

## Target Layout

The review packet directory is:

```text
.tokmd/review/
  manifest.json
  cockpit.json
  evidence.json
  comment.md
  review-map.json   # future
  review-map.md     # future
```

The initial implementation emits `manifest.json`, `cockpit.json`,
`evidence.json`, and `comment.md`. `review-map.json` and `review-map.md` should
land once the priority model is stable enough to treat as a contract.

## Artifacts

| Artifact | Contract |
| --- | --- |
| `manifest.json` | Packet index with schema name, generated-by metadata, base/head refs, artifact paths, hashes, and verdict metadata. |
| `cockpit.json` | Full `CockpitReceipt` JSON. This is the same receipt produced by `tokmd cockpit --format json`. |
| `evidence.json` | Evidence availability and gate status. It distinguishes passed evidence from missing, skipped, stale, degraded, or unavailable evidence. |
| `comment.md` | PR-comment-ready summary. It stays concise and links readers to packet artifacts when hosted by CI. |
| `review-map.json` | Machine-readable prioritized review plan with files, reasons, evidence references, and reproduction commands. |
| `review-map.md` | Human-readable review plan for artifact browsing and local review. |

## Evidence Semantics

Packet consumers must not treat unavailable evidence as passing evidence.

Recommended evidence status values:

| Status | Meaning |
| --- | --- |
| `passed` | Evidence ran for the requested commit/scope and met the configured policy. |
| `failed` | Evidence ran and violated the configured policy. |
| `missing` | Evidence was expected but no usable result was found. |
| `skipped` | Evidence was intentionally not requested for this run. |
| `stale` | Evidence exists but does not match the requested commit or scope. |
| `degraded` | Evidence exists but is partial, incomplete, or lower confidence than the normal policy requires. |
| `unavailable` | The runtime or checkout cannot support the evidence source. |

Missing, stale, degraded, and unavailable evidence should be visible in
`comment.md`, `evidence.json`, and `manifest.json` verdict metadata.

## Manifest Requirements

`manifest.json` should use schema `tokmd.review_packet_manifest.v1` and include:

- `schema`
- `generated_by` with `name`, `version`, and command arguments
- `generated_at_ms`
- `base_ref` and `head_ref`
- `artifacts` with `id`, `path`, `schema`, `media_type`, and `hash`
- `verdict` with `status`, `blocking`, and `reason`
- `capabilities` or links to capability details when checks are unavailable

Artifact paths in the manifest are relative to the packet directory. Hashes use
the repo-standard BLAKE3 digest and are computed from the final bytes written
to disk.

## Exit Codes

Packet emission success means the requested artifacts were written and are
internally consistent. Evidence verdicts are data inside the packet.

Future gate modes may map evidence verdicts to exit codes:

| Mode | Behavior |
| --- | --- |
| `off` | Never fail because of evidence verdicts. |
| `advisory` | Write failing or missing evidence into the packet but exit successfully when artifacts are valid. |
| `blocking` | Exit non-zero when configured blocking evidence fails or is missing. |

The default should remain advisory unless a repo explicitly chooses blocking
behavior.

## GitHub Action Behavior

The Action should upload the packet as an artifact before any optional comment
step. Comment posting must remain fork-safe and must not be required for packet
generation.

Future Action inputs should build on the cockpit surface first:

```yaml
mode: cockpit
review-packet: true
post-comment: true
upload-artifacts: true
```

Do not add `mode: review` until there is a distinct review orchestrator contract
that uses this packet.

## Non-Goals

- Replacing tests, coverage, mutation testing, SAST, or human review.
- Treating missing evidence as a successful check.
- Introducing an external review service or secret requirement.
- Adding AI-written recommendations without deterministic evidence references.

## Implementation Checklist

- `tokmd cockpit --review-packet-dir <dir>` can emit packet artifacts without
  changing default stdout.
- `manifest.json` hashes every artifact it lists.
- `cockpit.json` remains a valid cockpit receipt with the current cockpit schema.
- `evidence.json` records unavailable and degraded evidence explicitly.
- `comment.md` remains concise enough for PR comments.
- Existing `--format comment` and `--artifacts-dir` behavior remains compatible.
- Action artifact upload works even when comments are disabled or unavailable.
