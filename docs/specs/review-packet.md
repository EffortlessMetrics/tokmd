# Spec: Review Packet

- Status: active
- Schema family, if any: `tokmd.review_packet_manifest.v1`,
  `tokmd.review_packet_evidence.v1`, `tokmd.review_map.v1`;
  verifier receipt `tokmd.review_packet_check.v1`
- Related ADRs:
- Related proof scopes: `tokmd_cockpit`, `review_packet_verifier`,
  `doc_artifacts_policy`, `project_truth_docs`

## Contract

A review packet is a packet-local PR review evidence directory produced by
`tokmd cockpit --review-packet-dir <DIR>`. It gives maintainers and agents a
stable way to inspect what changed, which evidence is available, which evidence
is missing or degraded, and which files should be reviewed first.

The review packet is an additive cockpit output surface. It must not change
default `tokmd cockpit` stdout behavior, the existing `--artifacts-dir`
contract, cockpit receipt semantics, public receipt schema versions, required
CI gates, proof promotion, Codecov defaults, release mutation, or AST defaults.

The packet is not a merge verdict, release approval, proof verifier, AI review
service, or external evidence bus runtime. It may summarize imported proof and
documentation-control receipts, but those source receipts and their verifiers
remain authoritative for their own domains.

`tokmd cockpit` remains the current PR-review evidence surface. A public
`tokmd review` command should not be introduced unless a future accepted spec
defines a distinct orchestrator over the packet instead of duplicating cockpit
computation.

## Inputs

The packet producer consumes explicit cockpit inputs:

- base and head refs from the `tokmd cockpit` invocation;
- the computed `CockpitReceipt`;
- optional explicit proof artifacts accepted by
  `docs/cockpit-proof-evidence.md`;
- optional documentation-control evidence from
  `cargo xtask doc-artifacts --check --json <PATH>`;
- optional Action-hosting context used only for the hosted PR comment copy.

The packet producer must not silently discover hidden proof state, call GitHub
APIs for missing evidence, upload coverage, mutate external receipts, post
comments, or decide that evidence should become required.

Imported proof inputs must be explicitly supplied by path. Missing optional
imports are represented as missing, skipped, unavailable, stale, or degraded
evidence when relevant; they are not rendered as passing proof. Malformed or
unsafe explicitly supplied imports should fail before packet rendering.

## Outputs

The review packet directory uses this stable layout:

```text
<DIR>/
  manifest.json
  cockpit.json
  evidence.json
  comment.md
  review-map.json
  review-map.md
  proof/
    *.json
  docs/
    doc-artifacts-check.json
```

`proof/` is present only when explicit proof artifacts are imported. Copied
proof artifacts must use canonical packet-local paths and be listed in
`manifest.json` with hashes.

`docs/` is present only when explicit documentation-control artifacts are
imported. The copied doc-artifacts check receipt uses
`<packet>/docs/doc-artifacts-check.json` and is listed in `manifest.json` with
hashes.

`manifest.json` is the packet-local artifact index. It records generated-by
metadata, base/head refs, packet-local artifact paths, schemas, media types,
BLAKE3 hashes, verdict metadata, evidence availability counts, and capability
summaries. Artifact paths must be relative to the packet directory and must not
escape the packet root.

`cockpit.json` is the full cockpit receipt. It remains the authoritative source
for cockpit review-plan items and cockpit gate statuses.

`evidence.json` records evidence availability and imported proof/doc evidence.
It must distinguish passed evidence from `missing`, `skipped`, `stale`,
`degraded`, and `unavailable` evidence. Consumers must not treat absent,
planned-only, stale, or unknown-commit evidence as passing evidence.

`review-map.json` and `review-map.md` render a review-first map derived from
`cockpit.json#/review_plan`. They may reorder items for review use while keeping
refs back to the original cockpit review-plan indexes. Item-level proof refs are
allowed only when imported proof directly names the changed file; scope-only or
global proof remains packet-level evidence until policy-backed matching exists.

`comment.md` is the packet-local compact summary. When the composite Action
needs hosted metadata, it must write that hosted block to a separate
`tokmd-review-packet-comment.md` copy after packet hashing. The packet-local
`comment.md` must not be mutated after `manifest.json` hashes are written.

The verifier receipt written by `cargo xtask review-packet-check --json <PATH>`
uses schema `tokmd.review_packet_check.v1`. It verifies packet schemas,
packet-local paths, listed artifacts, and manifest hashes. It is intentionally
outside the packet manifest because it verifies the final packet instead of
being part of the packet.

## Compatibility

This spec preserves the current shipped surfaces:

- `tokmd cockpit --format json|md|comment|sections`;
- `tokmd cockpit --artifacts-dir <DIR>`;
- `tokmd cockpit --review-packet-dir <DIR>`;
- review-packet schema names and packet artifact names;
- `cargo xtask review-packet-check --dir <DIR> --json <PATH>`;
- composite Action artifact upload and fork-safe comment behavior;
- optional explicit proof imports;
- optional documentation-control evidence import.

Existing consumers can keep reading the packet artifacts documented in
`docs/review-packet.md`. Future changes that alter required packet artifacts,
schema families, hash behavior, evidence availability semantics, imported proof
rules, or Action-hosted comment behavior should update this spec, the user
guide, schemas, and verifier tests in the same PR.

## Proof Requirements

For spec-only changes:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-review-packet-spec.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-review-packet-spec.json --evidence-json target/proof/proof-evidence-review-packet-spec.json
git diff --check
```

Implementation changes to packet emission, imported evidence, hosted comment
copies, schemas, or verifier behavior should also run focused cockpit and
verifier tests that prove:

- required packet artifacts are written and listed in `manifest.json`;
- packet-local BLAKE3 hashes remain valid;
- artifact paths are relative and cannot escape the packet root;
- unavailable, missing, skipped, stale, and degraded evidence are not rendered
  as passing evidence;
- explicit proof imports preserve required/advisory classification and commit
  freshness;
- source-of-truth changes can link documentation-control evidence without
  turning it into a merge verdict;
- hosted comment metadata does not mutate packet-local `comment.md`;
- `cargo xtask review-packet-check` accepts valid packets and rejects malformed
  schemas, missing listed artifacts, bad hashes, hosted comment copies in the
  manifest, and unsafe paths;
- proof promotion, Codecov defaults, public receipt schemas, public CLI default
  behavior, release mutation, AST defaults, and branch-protection semantics are
  unchanged.

## Open Questions

- Whether exact review-map ordering should become machine-checked beyond the
  current review-first invariants.
- Whether a future policy-backed scope matcher should allow item-level proof
  refs for scope-only proof artifacts.
- Whether a future public `tokmd review` command is warranted as a distinct
  orchestrator over the packet, or whether cockpit should remain the only
  review evidence command.
