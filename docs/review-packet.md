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

## Reviewer Quickstart

For a local PR checkout, start with the review packet and verify its manifest
before treating it as review evidence:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review

cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

Open the packet in this order:

1. `.tokmd/review/comment.md` for the compact summary.
2. `.tokmd/review/review-map.md` for review-first ordering and reproduction
   commands.
3. `.tokmd/review/evidence.json` for exact available, missing, stale,
   degraded, skipped, or unavailable evidence.
4. `.tokmd/review/manifest.json` for packet-local artifact paths and hashes.
5. `target/tokmd/review-packet-check.json` for the verifier receipt.

The first two artifacts have different jobs. `comment.md` is the best first
screen because it compresses the packet into a hosted-comment-sized status.
`review-map.md` is the working artifact for the actual review: it names the
review-first files, the reason each item is in front, any matching proof lines,
and the commands to reproduce or repair evidence. Use `evidence.json` when a
summary line needs exact availability, freshness, required/advisory status, or
source-artifact detail.

Interpret common states this way:

| State seen in packet | Read as | Next action |
| --- | --- | --- |
| Required proof available and fresh | The named required evidence exists for this commit or scope. | Continue reviewing the changed files; do not treat the packet itself as merge approval. |
| Advisory proof missing or skipped | Optional evidence did not run or was intentionally not requested. | Not a required failure. Use the reproduction command only if the PR needs that optional signal. |
| Required proof missing, stale, degraded, or failed | The packet lacks trustworthy required evidence for the relevant surface. | Regenerate or repair the named proof before claiming the packet is complete. |
| Evidence unavailable | The runtime, checkout, or packet inputs could not support that evidence source. | Treat it as an explicit gap, not as passing evidence. |

For extended glossary entries, proof metadata fields, and worked examples across
both packet families, see [Packet consumption guide](packet-consumption.md).

If the PR changes `.tokmd-spec/**`, source-of-truth docs, the swarm routing
topology, agent workflow rails, plans, ADRs, templates, `.jules/goals/**`, or
doc-artifact policy, generate and import the documentation-control receipt
first:

```bash
cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json

tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --doc-artifacts-check target/docs/doc-artifacts-check.json \
  --review-packet-dir .tokmd/review
```

This imports documentation-control evidence into the packet. It is not a merge
verdict and does not promote advisory proof, coverage, mutation, or Codecov
upload into a required gate. For proof imports and hosted Action artifacts,
see [tokmd in Cockpit](tokmd-in-cockpit.md).

### Worked Examples by Evidence State

The reading order above stays the same in every state; what changes is which
line stops you and what you do next. The three packets below are illustrative,
not literal output. Field names match the `evidence.json` availability values in
[Evidence Semantics](#evidence-semantics) (`available`, `missing`, `skipped`,
`stale`, `degraded`, `unavailable`) and the cockpit gate statuses (`pass`,
`fail`, `warn`, `skipped`, `pending`). Open the named artifact to confirm before
making a review claim.

#### Passed: required evidence present and fresh

`comment.md` opens with available required evidence and no missing-required
line:

```text
Review packet: 7 files to review
Evidence: 4 available, 1 advisory skipped, 0 missing
Doc artifacts: verified
```

`evidence.json` shows the required gates as available for the reviewed commit:

```json
{
  "gates": [
    { "id": "complexity", "status": "pass", "availability": "available" },
    { "id": "doc_artifacts", "status": "pass", "availability": "available" }
  ]
}
```

Reading order in this state: skim `comment.md` to confirm nothing is missing,
then work `review-map.md` top-to-bottom on the changed files. You only need
`evidence.json` if a summary line needs an exact freshness or source detail.
Do not read a clean packet as merge approval; it means the named required
evidence existed for this commit, not that the change is correct.

#### Advisory-missing: optional evidence did not run

`comment.md` distinguishes advisory gaps from required ones:

```text
Review packet: 5 files to review
Evidence: 2 available, 2 advisory missing, 0 required missing
```

`evidence.json` marks the optional gate as not-passing without inventing a
result:

```json
{
  "gates": [
    { "id": "complexity", "status": "pass", "availability": "available" },
    { "id": "coverage", "status": "skipped", "availability": "skipped" },
    { "id": "mutation", "status": "unavailable", "availability": "unavailable" }
  ]
}
```

Reading order in this state: confirm in `comment.md` that the gaps are advisory,
then use `review-map.md` normally. Do not call an advisory gap a failure unless
policy made that proof required. If the PR genuinely needs the missing signal,
run the reproduction command from the matching `review-map.md` item or
`cockpit-proof-evidence.md`; importing it later refreshes the packet rather than
flipping a gate.

#### Failed: required evidence missing, stale, degraded, or failing

`comment.md` surfaces the required gap first so it is not mistaken for advisory
noise:

```text
Review packet: 9 files to review
Evidence: 1 available, 3 required missing/stale, 1 degraded
Required evidence is incomplete; do not treat this packet as complete.
```

`evidence.json` names exactly what is wrong:

```json
{
  "gates": [
    { "id": "complexity", "status": "fail", "availability": "available" },
    { "id": "doc_artifacts", "status": "pending", "availability": "stale" },
    { "id": "tests", "status": "pending", "availability": "missing" }
  ]
}
```

Reading order in this state: start from `evidence.json` (or the `comment.md`
required-gap line) to identify the untrustworthy gates, then regenerate or
repair the named proof before reviewing further. A `fail` gate is a real signal
to act on; `stale`, `degraded`, and `missing` mean the evidence cannot be
trusted for this commit, not that the surface passed. Re-run
`cargo xtask review-packet-check` after repairing so the manifest hashes and the
verifier receipt reflect the repaired packet.

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
    proof-pack-route.json
  docs/
    doc-artifacts-check.json
```

`review-map.json` and `review-map.md` are derived from the existing cockpit
`review_plan`. The packet keeps the original receipt order in `cockpit.json`,
but the review map may reorder items for review-first use: source-of-truth
items stay first, then missing/stale/degraded evidence, high-complexity items,
contract paths, existing cockpit priority, changed lines, and path. Each item
keeps evidence refs back to its original `cockpit.json#/review_plan/<index>`
source.

The `proof/` directory is present only when explicit proof evidence artifacts
are supplied. Missing optional proof artifacts are represented in evidence
state instead of being silently assumed to have passed.
The `docs/` directory is present only when explicit documentation-control
evidence is supplied with `--doc-artifacts-check`.

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
| `<packet>/docs/doc-artifacts-check.json` | Optional packet-local copy of explicitly imported documentation-control evidence, listed and hash-verified through `manifest.json`. |

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
schemas, artifact count, hash count, packet-local artifact paths, artifact
schemas, media types, and verifier errors. When packet-local `proof/*.json`
artifacts are present, downstream handoff output may list their verified path
and schema as inventory evidence. Handoff treats entries as proof artifacts only
when they are packet-local `proof/*.json` files with a recognized proof or
coverage receipt schema and JSON media type. That still does not make route
receipts executed proof or promote imported proof gates. The verifier receipt is
a CI and downstream-system artifact; it is not listed in the packet manifest.

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
required proof, advisory proof, proof routing, and freshness counts without
listing raw commands.

Cockpit proof imports should follow
[`cockpit-proof-evidence.md`](cockpit-proof-evidence.md). When proof artifacts
are supplied with `--review-packet-dir`, cockpit validates them, copies them
into canonical packet-local `proof/*.json` paths, and records normalized proof
items in `evidence.json`. Packet imports preserve required/advisory
classification and commit freshness. Coverage proof entries also preserve
non-empty GitHub `run_id`, `run_attempt`, `workflow`, `event_name`, and
`ref_name` values when the source receipt includes them, plus a derived
`run_url` for safe GitHub repository/run ID pairs. Packet imports must not
promote advisory proof into blocking evidence.

Proof-pack route receipts imported with `--proof-route` are routing evidence,
not execution proof. Cockpit copies them to
`.tokmd/review/proof/proof-pack-route.json`, records a `proof_pack_route`
entry in `evidence.json`, and may link matching changed files in the review
map. The entry is rendered with planned execution status so skipped-by-policy
lanes and selected proof packs remain visible without becoming passing proof.

For a complete local workflow that plans proof, optionally executes guarded
required proof, imports proof artifacts, and verifies the packet, see the
[`cockpit-proof-evidence.md` local review workflow](cockpit-proof-evidence.md#local-review-workflow).
For the planned stack boundary where evidencebus carries a verified tokmd
packet, see [tokmd and evidencebus integration](evidencebus-integration.md).

## Documentation Artifact Evidence

Source-of-truth changes are review evidence too. When a PR changes
`.tokmd-spec/**`, source-of-truth docs, the swarm routing topology, agent
workflow rails, plans, specs, ADRs, templates, `.jules/goals/**`, or
doc-artifact policy, cockpit packets can import the docs checker receipt:

```text
target/docs/doc-artifacts-check.json
```

That receipt uses schema `tokmd.doc_artifacts_check.v1` and is produced by:

```bash
cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json
```

The review packet should treat this as documentation-control evidence, not as a
merge verdict. A successful receipt means the source-of-truth artifact shape,
links, `.tokmd-spec` index entries, active-goal state, and policy routing
checked by the doc-artifacts contract were valid at verification time. It does
not prove the prose is
correct or that a PR should merge.

Packet treatment:

- `evidence.json` records the receipt schema, source path, `ok` result, checked
  counts, and any checker errors.
- `review-map.json` links source-of-truth review items to the imported
  doc-artifacts evidence when paths match the checked families or active-goal
  policy.
- `review-map.md` shows whether documentation-control evidence is available,
  missing, stale, or degraded for source-of-truth changes.
- `comment.md` may include a compact line such as `Doc artifacts: verified` or
  `Doc artifacts: missing for source-of-truth changes`.
- Source-of-truth review-map items include the `cargo xtask doc-artifacts
  --check --json target/docs/doc-artifacts-check.json` command in their
  reproduction commands, so reviewers can regenerate the imported receipt.

Absent doc-artifacts evidence is `missing` only when source-of-truth paths are
changed and the packet has enough context to know the receipt was expected.
Otherwise it is `unavailable` or omitted. Cockpit must not promote docs
evidence into a required gate by itself.

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
`review-map.md` is a Markdown rendering of the same ordered items. It starts
with a short work-order note that the packet is not a merge verdict, then lists
the "Review First" items with a review-first signal, the cockpit reason,
evidence present/missing lines where applicable, matching proof evidence, proof
references, evidence references, and reproduction commands for artifact browsing
and local review.

### Merge-blocking vs context-only review-map items

The review map orders items into risk tiers, and each `review-map.md`
"Review First" item names the tier through its review-first signal. Use the tier
to decide whether an item must be cleared before you treat the packet as
complete, or is only a reading-efficiency hint. This is a reading lens over the
existing gate and proof classification, not a new gate: the packet is still not
a merge verdict, and "merge-blocking" here means a maintainer must resolve or
explicitly acknowledge the item before sign-off, not that `tokmd` decides the
merge.

Treat these tiers as must-resolve before sign-off:

- source-of-truth artifact changed (a governing contract, spec, ADR, plan,
  routing topology, policy doc, or `.tokmd-spec/**`);
- required evidence missing, stale, or degraded for the item;
- high review complexity (4/5 or 5/5);
- contract or policy path changed (schemas, `policy/**`,
  `.github/workflows/**`, the CLI command surface, or public API).

Treat these tiers as context-only ordering (read for efficiency, not as a gate):

- highest or medium cockpit priority from the source review plan;
- items with available evidence and attached references;
- items whose only gap is skipped advisory evidence;
- lower-priority source review items.

To find blocking signal first, read the packet in this order: the `comment.md`
required-evidence line, then `evidence.json#/gates` for any `fail` status or
`missing`/`stale`/`degraded` availability on a required gate, then the
`review-map.md` "Review First" items whose signal names a must-resolve tier
above. Read the remaining `review-map.md` ordering and any advisory or skipped
lines as context-only.

These tiers reuse, and must not duplicate, the shared taxonomies: the
evidence-state glossary and per-family reading order in
[Packet consumption guide](packet-consumption.md#evidence-state-glossary), and
the manifest trust order and advisory-versus-required split for the ub-review
lane in
[ub-review ↔ tokmd packet integration](ub-review-integration.md#claim-boundary-what-to-trust-first-in-manifestjson).
Required-versus-advisory classification still comes from the gate and proof
metadata described in [Evidence Semantics](#evidence-semantics).

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
- Local proof-aware review workflow is documented with packet verification.
- Planned doc-artifact evidence import behavior is documented before cockpit
  implementation.
- Evidencebus packet mapping is documented without adding a tokmd CLI command
  or evidencebus runtime dependency.
