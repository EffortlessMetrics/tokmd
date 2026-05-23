# Cockpit Proof Evidence Import Contract

Status: implemented for explicit proof imports. `tokmd cockpit` can validate
explicitly supplied proof artifacts, copy them into the review packet under
`proof/`, and attach normalized imported proof items to `evidence.json` when
`--review-packet-dir` is used. `review-map.json` can link direct changed-file
matches to packet-local proof refs, and `review-map.md` now renders both a
packet-level proof evidence overview and matching item-level proof lines.
`comment.md` includes compact proof evidence totals for required/advisory proof
and freshness.

## Purpose

Cockpit is the review evidence surface. The proof control plane is the proof
router. Future cockpit proof imports should connect those systems without
making cockpit a CI decision engine.

The intended flow is:

```text
proof-plan / proof-run / executor artifacts
  -> cockpit proof evidence import
  -> evidence.json
  -> proof/*.json packet-local artifact copies
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

## Local Review Workflow

Use this workflow when you want a proof-aware packet for a local branch or a
pull request checkout. The first command shows what proof would run without
executing anything:

```bash
cargo xtask proof \
  --profile affected \
  --base origin/main \
  --head HEAD \
  --plan
```

When you intentionally want to execute required proof locally, use the explicit
guard and then derive the compact observation artifact:

```bash
cargo xtask proof \
  --profile fast \
  --base origin/main \
  --head HEAD \
  --run-required \
  --allow-local-required-execution \
  --proof-run-summary target/proof-run/proof-run-summary.json \
  --summary-md target/proof-run/proof-run-summary.md

cargo xtask proof-run-observation \
  --proof-run-summary target/proof-run/proof-run-summary.json \
  --output target/proof-run/proof-run-observation.json
```

Then import the proof artifacts while writing the review packet:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --proof-run-summary target/proof-run/proof-run-summary.json \
  --proof-observation target/proof-run/proof-run-observation.json \
  --review-packet-dir .tokmd/review

cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

When advisory executor or coverage receipts exist, pass them too:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --executor-observation target/proof/proof-executor-observation.json \
  --coverage-receipt target/coverage/coverage-receipt.json \
  --review-packet-dir .tokmd/review
```

The packet remains advisory by default. Imported proof changes evidence
visibility; it does not promote coverage, mutation, Codecov, or fast proof into
required gates.

## Normalized Evidence Model

Cockpit should normalize imported proof artifacts into a small internal evidence
model before rendering packet artifacts.

Each imported evidence item should preserve:

- packet-local source artifact path;
- source schema;
- source run metadata, such as GitHub run ID and attempt, or workflow URL when
  available;
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

Coverage receipts that include GitHub run metadata preserve non-empty
`run_id` and `run_attempt` values on their normalized `evidence.json` proof
entry. That lets packet consumers distinguish reruns while keeping the copied
`proof/coverage-receipt.json` artifact as the full source payload.

Packet renderers should refer back to packet-local source artifacts using
stable refs rather than copying large proof payloads into every packet file.

Example future reference shape:

```json
{
  "proof_refs": [
    "proof/proof-run-observation.json#/entries/0",
    "proof/proof-executor-observation.json#/entries/3"
  ]
}
```

When proof artifacts are supplied with `--review-packet-dir`, cockpit copies
the validated JSON input into canonical packet-local names:

| Kind | Packet path |
| --- | --- |
| proof run summary | `proof/proof-run-summary.json` |
| proof run observation | `proof/proof-run-observation.json` |
| proof executor observation | `proof/proof-executor-observation.json` |
| coverage receipt | `proof/coverage-receipt.json` |

These copied artifacts are listed in `manifest.json` with BLAKE3 hashes. The
review packet verifier treats them like any other packet artifact.

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

When proof evidence is imported, packet outputs show it in the same evidence
vocabulary used by the current review packet:

- `available`
- `missing`
- `skipped`
- `stale`
- `degraded`
- `unavailable`

The information should be visible in:

- `evidence.json` gate entries and imported proof entries;
- `review-map.json` item evidence status and `proof_refs`;
- `review-map.md` packet-level proof overview;
- `review-map.md` item proof lines for review-first direct changed-file matches;
- `comment.md` compact evidence availability and proof evidence totals.

Item-level proof refs are intentionally conservative. Cockpit may attach proof
to a review-map item when the imported proof artifact has unambiguous
changed-file ownership, such as a single normalized proof item or multiple
entries for one scope. Multi-scope artifacts with only a top-level
`changed_files` list remain packet-level evidence until a policy-backed scope
matcher can prove which files belong to which scope.

Review map output should answer a reviewer-facing question:

```text
This PR touched tokmd_cockpit.
Required proof passed: cockpit integration, cockpit workflow.
Advisory coverage produced tokmd_cockpit.lcov.
Mutation was planned but not executed.
Diff coverage evidence is missing.
```

## Error Handling

Explicit import behavior distinguishes:

- absent optional proof artifact;
- explicitly requested artifact that is missing;
- malformed artifact;
- unknown schema;
- stale commit.

Packet verification distinguishes:

- manifest artifact paths that are not relative or are outside the packet root;
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
- Normalize required/advisory status before rendering. (done)
- Classify commit match as exact, partial, stale, or unknown. (done)
- Attach imported proof entries to `evidence.json`. (done)
- Copy supplied proof artifacts into packet-local `proof/*.json` files. (done)
- Attach proof refs to review-map items without duplicating large artifacts. (done for direct changed-file matches)
- Render packet-level and matching proof evidence in `review-map.md` without changing JSON schemas. (done; item-level links currently use direct changed-file matches)
- Render compact proof evidence totals in `comment.md` without listing raw commands. (done)
- Keep review packet schemas versioned if output shape changes.
- Keep proof-control-plane promotion decisions outside cockpit.
