# tokmd and evidencebus integration

Status: planned integration contract. This document maps tokmd review packet
artifacts to the evidencebus producer/packet boundary. It does not add a new
CLI command or make evidencebus a required dependency.

## Purpose

tokmd is a code-evidence producer. evidencebus is the cross-tool evidence
backplane. The integration boundary should let evidencebus validate, inventory,
bundle, and export tokmd evidence without making tokmd responsible for the
whole evidence system.

The first integration target is the proof-aware cockpit review packet:

```text
tokmd cockpit --review-packet-dir .tokmd/review
  -> cargo xtask review-packet-check --dir .tokmd/review
  -> evidencebus packet wrapping .tokmd/review
```

tokmd remains useful standalone. evidencebus can carry the same packet with
evidence from mergecode, CI sensors, gates, performance tools, and other
producers.

## Artifact Mapping

| tokmd artifact | evidencebus role |
| --- | --- |
| `.tokmd/review/manifest.json` | Packet inventory. Lists review artifacts, schemas, BLAKE3 hashes, base/head refs, and verdict metadata. |
| `.tokmd/review/cockpit.json` | Source code-review receipt. Contains the full cockpit receipt and review plan source data. |
| `.tokmd/review/evidence.json` | Evidence state. Records gate availability, imported proof evidence, required/advisory classification, and freshness. |
| `.tokmd/review/review-map.json` | Review routing. Gives tools an ordered list of review items with evidence refs, proof refs, and reproduction commands. |
| `.tokmd/review/review-map.md` | Human review view. Lets a reviewer start from the same ordered review map without parsing JSON. |
| `.tokmd/review/comment.md` | Human summary. Concise packet-local summary suitable for PR comments or bundle previews. |
| `.tokmd/review/proof/*.json` | Source proof artifacts. Packet-local copies of explicitly imported proof evidence, hash-listed in `manifest.json`. |
| `target/tokmd/review-packet-check.json` | Verification receipt. Proves the packet manifest, schemas, paths, and hashes were checked after generation. |
| `target/docs/doc-artifacts-check.json` | Documentation-control evidence. Optional future review-packet input proving source-of-truth artifact shape, links, active-goal state, and policy routing were checked. |

The verifier receipt is intentionally outside `.tokmd/review/manifest.json`.
It verifies the final packet rather than being part of the packet it verifies.
An evidencebus wrapper may include it as a sibling evidence item.

## Producer Metadata

An evidencebus wrapper for a tokmd review packet should preserve:

- tokmd version and command arguments from `manifest.json`;
- packet schema names and schema versions;
- base and head refs or commits;
- artifact paths exactly as packet-local relative paths;
- BLAKE3 hashes from `manifest.json`;
- verifier receipt schema and result;
- proof evidence freshness values: `exact`, `partial`, `stale`, or `unknown`;
- evidence availability values: `available`, `missing`, `skipped`, `stale`,
  `degraded`, or `unavailable`;
- documentation artifact checker result and checked counts when a review packet
  imports `tokmd.doc_artifacts_check.v1` evidence;
- required/advisory proof classification;
- reproduction commands from `review-map.json`.

The wrapper should not rewrite packet-local artifact paths into host-specific
absolute paths. If evidencebus stores the packet in a bundle, it should retain
the tokmd packet path as bundle-internal provenance.

## Trust Boundary

The evidencebus consumer can rely on a tokmd review packet only when both are
true:

1. `.tokmd/review/manifest.json` lists all packet-local artifacts that are part
   of the packet contract.
2. `cargo xtask review-packet-check --dir .tokmd/review --json <path>` produced
   a successful `tokmd.review_packet_check.v1` receipt for that packet.

Passing verification means the packet shape, manifest paths, schemas, and
hashes were consistent at verification time. It does not mean all evidence
inside the packet passed. Consumers must still read `evidence.json` and preserve
missing, stale, degraded, skipped, and unavailable states.

## Future Command Shape

The first implementation should stay outside the user-facing tokmd CLI until
the evidencebus packet schema is stable. A narrow `xtask` prototype is enough:

```bash
cargo xtask evidencebus-review-packet \
  --packet .tokmd/review \
  --verifier target/tokmd/review-packet-check.json \
  --out target/evidencebus/tokmd-review-packet.json
```

Possible later CLI shape:

```bash
tokmd export --evidencebus review-packet \
  --input .tokmd/review \
  --verifier target/tokmd/review-packet-check.json \
  --out tokmd-review-packet.eb.json
```

Do not add either command until there is an evidencebus-side schema and
validation path to target.

## Non-Goals

- Making evidencebus a tokmd runtime dependency.
- Making tokmd the evidencebus backplane.
- Adding a merge verdict or global readiness decision.
- Promoting advisory coverage, mutation, Codecov, or fast proof into required
  gates.
- Introducing a separate `tokmd review` command.
- Copying large proof payloads into every mapped artifact.
- Treating missing or stale proof as passing proof.

## Implementation Checklist

- Keep tokmd review packet generation and verification working standalone.
- Define the evidencebus packet schema on the evidencebus side first.
- Prototype wrapping in `xtask`, not the public CLI.
- Preserve packet-local paths and hashes from `manifest.json`.
- Include `review-packet-check.json` as a verifier evidence item.
- Include `doc-artifacts-check.json` as documentation-control evidence only
  after cockpit has a documented packet import path.
- Validate the wrapper with evidencebus before adding user-facing docs beyond
  this contract.
