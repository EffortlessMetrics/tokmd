# Cockpit Proof Evidence Import Contract

Status: partially implemented. `tokmd cockpit` can validate explicitly supplied
proof artifacts through validation-only flags, but it does not yet attach
imported proof evidence to review packet outputs. This document defines the
contract future implementation should follow when attaching proof-control-plane
evidence to cockpit review packets.

## Purpose

Cockpit is the review evidence surface. The proof control plane is the proof
router. Future cockpit proof imports should connect those systems without
making cockpit a CI decision engine.

The intended flow is:

```text
proof-plan / proof-run / executor artifacts
  -> cockpit proof evidence import
  -> evidence.json
  -> review-map.json / review-map.md
  -> comment.md
```

Imported proof evidence should help a reviewer answer:

- which proof was planned for the touched scopes;
- which proof actually ran;
- whether the proof matched the reviewed commit;
- whether each proof signal was required or advisory;
- what evidence is missing, stale, unavailable, skipped, or degraded;
- which command or artifact reproduces the claim.

## Accepted Inputs

Cockpit accepts these artifact families for validation. Future import support
may normalize the same artifacts into review packet evidence:

| Artifact | CLI flag | Role |
| --- | --- |
| `proof-run-summary.json` | `--proof-run-summary <PATH>` | Summary from `cargo xtask proof --run-required ...`; represents required proof commands that were executed under an explicit guard. |
| `proof-run-observation.json` | `--proof-observation <PATH>` | Compact observation derived from a verified required proof-run summary; useful for routine fast-proof visibility. |
| `proof-executor-observation.json` | `--executor-observation <PATH>` | Observation from scoped advisory executor artifacts, such as non-required coverage runs. |
| `coverage-receipt.json` | `--coverage-receipt <PATH>` | Coverage artifact inventory and byte-count receipt; useful for proving a coverage artifact exists without treating coverage as required. |

Inputs are optional. A missing artifact is not an error unless the caller
explicitly requested that artifact. When an explicitly supplied artifact is
missing, malformed, or does not match the flag's expected artifact family,
cockpit fails before rendering. Passive discovery can later record degraded
evidence state instead of failing.

## Normalized Evidence Model

Cockpit should normalize imported proof artifacts into a small internal evidence
model before rendering packet artifacts.

Each imported evidence item should preserve:

- source artifact path;
- source schema;
- source run or workflow URL when available;
- base ref or commit when available;
- head ref or commit when available;
- proof profile, such as `fast`, `affected`, `release`, or `deep`;
- proof scope name;
- command text or command id;
- execution status;
- exit code when available;
- required/advisory classification;
- artifact references, such as LCOV paths;
- generated timestamp when available.

Packet renderers should refer back to source artifacts using stable refs rather
than copying large proof payloads into every packet file.

Example future reference shape:

```json
{
  "proof_refs": [
    "proof-run-observation.json#/entries/0",
    "proof-executor-observation.json#/entries/3"
  ]
}
```

## Commit Matching

Imported proof is only strong evidence when it matches the change being
reviewed. Cockpit should classify imported proof with a commit match status:

| Match | Meaning |
| --- | --- |
| `exact` | The proof artifact's head commit equals the cockpit head commit. |
| `partial` | The proof artifact covers a matching scope or command, but base/head metadata is incomplete or only one side matches. |
| `stale` | The proof artifact is from a different head commit or an older source-run window. |
| `unknown` | The proof artifact does not contain enough commit metadata to compare. |

Stale or unknown proof must not be rendered as passing evidence. It may still
be useful context, but packet outputs should show it as stale or degraded.

## Required vs Advisory

Cockpit should preserve the proof policy classification supplied by the proof
artifacts.

| Classification | Review-packet treatment |
| --- | --- |
| Required proof passed | Display as available evidence for the matching scope. |
| Required proof failed | Display as failing evidence with the reproducing command. |
| Required proof missing | Display as missing evidence when the touched scope expected it. |
| Advisory proof passed | Display as available advisory evidence, not as a merge requirement. |
| Advisory proof failed | Display as advisory failure or degraded evidence, depending on policy. |
| Advisory proof not executed | Display as planned or skipped, not as passing. |

The review packet can show required/advisory status, but cockpit must not decide
to promote advisory proof into required proof.

## Rendering Requirements

When proof evidence is imported, future packet outputs should show it in the
same evidence vocabulary used by the current review packet:

- `available`
- `missing`
- `skipped`
- `stale`
- `degraded`
- `unavailable`

The information should be visible in:

- `evidence.json` gate entries;
- `review-map.json` item evidence status and `proof_refs`;
- `review-map.md` proof lines for review-first items;
- `comment.md` compact evidence availability text.

Review map output should answer a reviewer-facing question:

```text
This PR touched tokmd_cockpit.
Required proof passed: cockpit integration, cockpit workflow.
Advisory coverage produced tokmd_cockpit.lcov.
Mutation was planned but not executed.
Diff coverage evidence is missing.
```

## Error Handling

Future import behavior should distinguish:

- absent optional proof artifact;
- explicitly requested artifact that is missing;
- malformed artifact;
- unknown schema;
- stale commit;
- artifact paths that are not relative or are outside the packet root;
- evidence artifact listed but missing on disk.

Malformed or unsafe inputs should fail fast when the caller explicitly provided
the input path. Passive discovery can instead record degraded or unavailable
evidence.

## Non-Goals

- Promoting fast proof, scoped coverage, mutation, or Codecov into required
  gates.
- Enabling default Codecov upload.
- Producing a merge verdict.
- Replacing existing CI jobs or proof-policy checks.
- Treating planned-but-not-executed proof as passing proof.
- Treating stale or unknown-commit proof as passing proof.
- Adding a separate `tokmd review` command.

## Implementation Checklist

- Add deserializable DTOs for accepted proof artifacts. (done)
- Validate explicit proof artifact inputs without changing receipt output. (done)
- Keep absent imports compatible with current cockpit behavior. (done)
- Normalize required/advisory status before rendering.
- Classify commit match as exact, partial, stale, or unknown.
- Attach proof refs to review-map items without duplicating large artifacts.
- Keep review packet schemas versioned if output shape changes.
- Keep proof-control-plane promotion decisions outside cockpit.
