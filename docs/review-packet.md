# Review Packet Contract

Status: implemented. `tokmd cockpit --review-packet-dir <dir>` emits packet
artifacts without changing the existing default `tokmd cockpit` stdout behavior
or the shipped `--artifacts-dir` contract.

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
  review-map.json
  review-map.md
  proof/
    proof-run-summary.json
    proof-run-observation.json
    proof-executor-observation.json
    coverage-receipt.json
```

`review-map.json` and `review-map.md` are derived from the existing cockpit
`review_plan`. They do not add a new scoring model.

The `proof/` directory is present only when explicit proof evidence artifacts
are supplied. Missing optional proof artifacts are represented in evidence
state instead of being silently assumed to have passed.

## Artifacts

| Artifact | Contract |
| --- | --- |
| `manifest.json` | Packet index with schema name, generated-by metadata, base/head refs, artifact paths, hashes, and verdict metadata. |
| `cockpit.json` | Full `CockpitReceipt` JSON. This is the same receipt produced by `tokmd cockpit --format json`. |
| `evidence.json` | Evidence availability and gate status. It distinguishes passed evidence from missing, skipped, stale, degraded, or unavailable evidence. |
| `comment.md` | PR-comment-ready summary. It stays concise, summarizes evidence/proof availability, and points readers to packet artifacts when hosted by CI. |
| `review-map.json` | Machine-readable prioritized review plan with files, reasons, compact evidence status, evidence references, item-level proof references where imported proof directly matches the item path, and reproduction commands derived from `cockpit.json#/review_plan`. |
| `review-map.md` | Human-readable review plan for artifact browsing and local review, including what to review first, which evidence is present or missing, and matching proof evidence lines when imported proof directly names the item path. |
| `proof/*.json` | Optional packet-local copies of explicitly imported proof artifacts, listed and hash-verified through `manifest.json`. |

Formal JSON Schemas are published with the docs and embedded in the CLI test
package:

- [`review-packet-manifest.schema.json`](review-packet-manifest.schema.json)
- [`review-packet-evidence.schema.json`](review-packet-evidence.schema.json)
- [`review-map.schema.json`](review-map.schema.json)

## Packet Verification

`cargo xtask review-packet-check --dir <dir>` validates the packet manifest,
`evidence.json`, and `review-map.json` against their embedded schemas, verifies
that manifest artifact paths are packet-local, rejects hosted comment copies in
the manifest, and recomputes BLAKE3 hashes for listed artifacts.

Use `--json <path>` to write a machine-readable verifier receipt:

```bash
cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/review-packet-check.json
```

The receipt uses schema `tokmd.review_packet_check.v1` and records the verified
schemas, artifact count, hash count, packet-local artifact paths, and verifier
errors. It is a verifier artifact for CI and downstream systems; it is not
listed in the packet manifest.

## Evidence Semantics

Packet consumers must not treat unavailable evidence as passing evidence.

`evidence.json` records the existing cockpit gate status (`pass`, `fail`,
`warn`, `skipped`, or `pending`) plus a separate availability value. Optional
gates that are not present in the cockpit receipt are represented with
`status: "unavailable"` and `availability: "unavailable"` so consumers cannot
mistake absent evidence for a passing gate.

Recommended evidence availability values:

| Availability | Meaning |
| --- | --- |
| `available` | Evidence ran for the requested commit/scope and can be interpreted with the gate status. |
| `missing` | Evidence was expected for a relevant scope, but no tested scope or usable result was found. |
| `skipped` | Evidence was intentionally not requested for this run. |
| `stale` | Evidence exists but does not match the requested commit or scope. |
| `degraded` | Evidence exists but is partial, incomplete, or lower confidence than the normal policy requires. |
| `unavailable` | The runtime or checkout cannot support the evidence source. |

Missing, stale, degraded, and unavailable evidence should be visible in
`comment.md`, `evidence.json`, and `manifest.json` verdict metadata.
When explicit proof artifacts are imported, `comment.md` also summarizes
required proof, advisory proof, and freshness counts without listing raw
commands.

Cockpit proof imports should follow
[`cockpit-proof-evidence.md`](cockpit-proof-evidence.md). When proof artifacts
are supplied with `--review-packet-dir`, cockpit validates them, copies them
into canonical packet-local `proof/*.json` paths, and records normalized proof
items in `evidence.json`. Packet imports preserve required/advisory
classification and commit freshness, and must not promote advisory proof into
blocking evidence.

## Manifest Requirements

`manifest.json` should use schema `tokmd.review_packet_manifest.v1` and include:

- `schema`
- `generated_by` with `name`, `version`, and command arguments
- `generated_at_ms`
- `base_ref` and `head_ref`
- `artifacts` with `id`, `path`, `schema`, `media_type`, and `hash`
- `verdict` with `status`, `blocking`, and `reason`
- `verdict.evidence` with counts by evidence availability and a link to
  `evidence.json#/gates`
- `capabilities.evidence` with gate ids grouped by availability and a link to
  `evidence.json#/gates`

Artifact paths in the manifest are relative to the packet directory. Hashes use
the repo-standard BLAKE3 digest and are computed from the final bytes written
to disk. Optional copied proof artifacts must also be listed in the manifest
using packet-local relative paths such as `proof/proof-run-observation.json`.

## Review Map Requirements

`review-map.json` should use schema `tokmd.review_map.v1` and include:

- `schema`
- `base_ref` and `head_ref`
- `source` identifying `cockpit.review_plan`
- `item_count`
- `items` sorted in cockpit review-plan order

The map may also include a packet-level evidence summary copied from the same
availability buckets as `manifest.json`. Each item includes rank, path,
priority, priority label, reason, optional complexity, optional lines changed,
compact item-level evidence status, evidence references, optional `proof_refs`,
and reproduction commands. Proof refs are added only when imported proof
evidence directly lists the item path as a changed file; scope-only or global
proof stays packet-level until a policy-backed scope matcher exists.
`review-map.md` is a Markdown rendering of the same ordered items, including a
"Review First" section, evidence present/missing lines where applicable,
matching proof evidence, proof references, evidence references, and reproduction
commands for artifact browsing and local review.

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

The Action uploads the packet as an artifact when `artifact: 'true'` and
`review-packet: 'true'` are both set. Comment posting remains fork-safe and is
not required for packet generation.

When the composite Action generates a review packet, it copies
`.tokmd/review/comment.md` to `tokmd-review-packet-comment.md` and appends a
hosted-packet block to that comment copy before artifact upload and PR
commenting. With artifact upload enabled, that block points to the workflow run,
the `tokmd-receipts` artifact, and the packet path. With artifact upload
disabled, it states that the packet was not uploaded. The packet-local
`comment.md` is not mutated after `manifest.json` hashes are written.

After preparing the hosted comment copy, the Action runs
`cargo xtask review-packet-check --dir .tokmd/review --json
target/tokmd/review-packet-check.json`. The verifier receipt is uploaded with
`tokmd-receipts` when artifact upload is enabled. It is intentionally outside
the packet manifest because it verifies the final packet instead of being part
of the packet itself. The hosted comment copy includes the verification status,
manifest hash status, and compact proof evidence counts; packet-local
`comment.md` remains unchanged.

Action inputs build on the cockpit surface first:

```yaml
mode: cockpit
review-packet: true
comment: true
artifact: true
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
- `manifest.json` summarizes evidence availability and links to
  `evidence.json#/gates`.
- `cockpit.json` remains a valid cockpit receipt with the current cockpit schema.
- `evidence.json` records unavailable and degraded evidence explicitly.
- `comment.md` remains concise enough for PR comments.
- Existing `--format comment` and `--artifacts-dir` behavior remains compatible.
- Action artifact upload works even when comments are disabled or unavailable.
- Proof evidence imports preserve required/advisory status, mark stale or
  unknown-commit evidence explicitly, and list packet-local proof artifact
  copies in `manifest.json`.
