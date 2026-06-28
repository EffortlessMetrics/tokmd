# Packet Consumption Guide

Status: user-facing consumer guide for tokmd review evidence.

This guide explains how to read tokmd packets without mixing artifact families
or mistaking advisory gaps for required failures. It does not introduce a new
command or schema version.

## Two Packet Families

tokmd ships two distinct review-evidence directories. They answer different
questions and must not be treated as interchangeable.

| Family | Typical path | Manifest schema | Primary question |
| --- | --- | --- | --- |
| PR evidence packet | `sensors/tokmd/` | `tokmd.evidence-packet/v1` | What scoped `tokmd` receipts were produced for this base/head/preset? |
| Cockpit review packet | `.tokmd/review/` | `tokmd.review_packet_manifest.v1` | What should a maintainer review first, and which cockpit/proof gates apply? |

Use [Evidence packet](evidence-packet.md) and
[Evidence packet workflow spec](specs/evidence-packet-workflow.md) for PR
evidence packets. Use [Review packet contract](review-packet.md) for cockpit
review packets.

## Reading Order

### PR evidence packet (`sensors/tokmd/`)

1. `manifest.json` — packet status, artifact list, warnings, errors, and
   reproduction commands.
2. `analyze.md` — human-first review context for the scoped paths.
3. `analyze.json` — machine-readable analysis receipt for bots and ledgers.
4. `context.md` — which source files were included, truncated, or skipped.
5. `syntax.json` — optional advisory parser signals; open referenced entries
   before acting on `review_priority`.

Re-run verification with the `reproduce` commands recorded in `manifest.json`,
or invoke `tokmd evidence-packet` with the same base, head, preset, and paths.

### Cockpit review packet (`.tokmd/review/`)

1. `comment.md` — compact first screen; safe for hosted PR comments.
2. `review-map.md` — review-first ordering, reasons, and reproduction commands.
3. `evidence.json` — exact gate status, availability, and imported proof
   metadata.
4. `manifest.json` — packet-local artifact paths and hashes.
5. `target/tokmd/review-packet-check.json` — verifier receipt from
   `cargo xtask review-packet-check`.

Verify the packet before trusting it:

```bash
cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

## Evidence State Glossary

Consumers must not treat **missing advisory evidence** as a required failure,
and must not treat **unavailable** evidence as passing proof.

### Shared vocabulary

| State | Meaning | Typical next action |
| --- | --- | --- |
| **Required and available** | Evidence the workflow or policy treats as blocking exists for the relevant commit/scope and can be interpreted with its gate status. | Continue review; the packet is not a merge verdict. |
| **Advisory and missing or skipped** | Optional evidence was not produced, not requested, or was skipped by policy. | Do not call it a failure unless your repo explicitly promoted that proof to required. Use the reproduction command if the PR needs that signal. |
| **Required and missing** | Blocking evidence was expected for the relevant scope but no usable result was found. | Regenerate or repair the named proof before claiming the packet is complete. |
| **Stale** | Evidence exists but does not match the requested commit, ref, or scoped paths. | Re-run the recorded reproduction command against the current head. |
| **Degraded** | Evidence exists but is partial, incomplete, or lower confidence than normal policy requires. | Read the warning or availability reason; do not treat degraded proof as fresh required proof. |
| **Unavailable** | The runtime, checkout, or packet inputs could not support that evidence source. | Treat it as an explicit gap, not as passing evidence. |

### PR evidence packet status (`manifest.json`)

| `status` | Read as | Command exit |
| --- | --- | --- |
| `complete` | Required artifacts exist; refs resolve; scoped analysis is parseable and consistent. | `0` |
| `partial` | Required artifacts exist, but named warnings bound the evidence (for example optional `syntax.json` missing or degraded). | `0` |
| `failed` | A required artifact is missing, refs do not resolve, or verification found a fatal mismatch. | non-zero after writing manifest |

Required manifest artifacts: `analyze_md`, `analyze_json`, `context_md`.
Optional advisory artifact: `syntax_json`. Missing optional syntax degrades to
`partial` with warnings; it is **not** a required proof failure unless a
downstream workflow explicitly requires syntax outside this contract.

### Cockpit review packet availability (`evidence.json`)

| `availability` | Meaning |
| --- | --- |
| `available` | Evidence ran for the requested commit/scope and can be interpreted with the gate status. |
| `missing` | Evidence was expected for a relevant scope, but no tested scope or usable result was found. |
| `skipped` | Evidence was intentionally not requested for this run. |
| `stale` | Evidence exists but does not match the requested commit or scope. |
| `degraded` | Evidence exists but is partial, incomplete, or lower confidence than normal policy requires. |
| `unavailable` | The runtime or checkout cannot support the evidence source. Optional gates absent from the cockpit receipt use `status: "unavailable"` so consumers cannot mistake absent evidence for a passing gate. |

Gate `status` (`pass`, `fail`, `warn`, `skipped`, `pending`) describes the
gate outcome when evidence exists. Availability describes whether the evidence
source can be interpreted at all. Read both fields together.

## Proof Metadata Fields

When cockpit review packets import proof artifacts, `evidence.json` may record:

| Field | Meaning |
| --- | --- |
| `run_id` | GitHub Actions run identifier when the source receipt includes it. |
| `run_attempt` | Attempt number for that workflow run. |
| `run_url` | Derived URL for safe GitHub repository/run ID pairs. |
| `workflow` | Workflow file name that produced the proof receipt. |
| `event_name` | GitHub event that triggered the run (`pull_request`, `push`, etc.). |
| `ref_name` | Branch or tag name associated with the run. |
| `required` | When `true`, missing or stale proof should block claims of packet completeness. When `false`, the proof is advisory. |
| `source` | Path or schema of the imported receipt copied into `proof/*.json`. |

Proof-pack route receipts (`proof/proof-pack-route.json`) are routing evidence,
not execution proof. They show which lanes were planned or skipped by policy;
they do not make skipped lanes pass.

## Worked Examples

### Required proof available

Review packet summary line:

```text
Required proof: 2/2 available (fresh)
```

Read as: imported required proof exists and matches the packet head. Continue
reviewing changed files; the packet still does not approve merge by itself.

### Advisory proof missing (not a failure)

Review packet summary line:

```text
Advisory proof: coverage receipt skipped by policy
```

Read as: coverage was not required for this run. Do not escalate to a required
failure unless branch policy says coverage is blocking for this change.

PR evidence packet manifest:

```json
"status": "partial",
"warnings": ["syntax_json: optional artifact missing"]
```

Read as: required analyze/context evidence is present; optional syntax is absent.
Treat warnings as named limits, not as invalid evidence.

### Required proof missing

Review packet `evidence.json` entry:

```json
"availability": "missing",
"status": "pending",
"required": true
```

Read as: the packet expected required proof for this scope and does not have
usable evidence. Regenerate proof before claiming review is complete.

PR evidence packet manifest:

```json
"status": "failed",
"errors": ["analyze_json: file not found"]
```

Read as: the packet is invalid evidence. Do not use it for merge or agent
handoff claims until regeneration succeeds.

## Hosted-comment troubleshooting

Cockpit review packets can be posted as hosted PR comments by the composite
Action (`mode: cockpit`, `review-packet: true`, `comment: true`). Comment
posting is optional and fork-safe: a missing comment does **not** mean packet
generation failed. The implementation surface remains `tokmd cockpit`; there is
no separate `tokmd review` command.

| Symptom | Likely cause | What to do |
| --- | --- | --- |
| Workflow succeeded but no PR comment | Event is not `pull_request`, or `comment: false` | Expected for `push`, `schedule`, and `workflow_dispatch`. Download the `tokmd-receipts` artifact or read `.tokmd/review/` from the job log path. |
| Fork PR has artifacts but no comment | Fork-safe comment posting is skipped | Treat the uploaded packet as the source of truth. Do not infer packet failure from the absent comment. |
| Comment says the packet was not uploaded | `artifact: false` while `comment: true` | Set `artifact: true` when reviewers need hosted links, or run `tokmd cockpit --review-packet-dir .tokmd/review` locally. |
| Hosted comment text differs from `.tokmd/review/comment.md` | Action copies to `tokmd-review-packet-comment.md` and appends run/artifact links | Normal. Packet-local `comment.md` stays unchanged so `manifest.json` hashes remain valid. |
| `review-packet-check` rejects a file in `.tokmd/review/` | A hosted comment copy was placed inside the packet directory | Keep hosted copies outside the packet tree. The verifier rejects hosted comment copies in the manifest path set. |
| Comment posting fails with permissions error | Workflow lacks `pull-requests: write` | Add the permissions block from [GitHub Action](github-action.md#permissions). |
| Comment shows verifier failure | Packet failed `cargo xtask review-packet-check` after the hosted copy was prepared | Read `target/tokmd/review-packet-check.json` and regenerate the packet before trusting the summary. |

Reproduce packet generation and verification locally without posting a comment:

```bash
tokmd cockpit --base "$BASE" --head "$HEAD" --review-packet-dir .tokmd/review
cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

See [Review packet contract](review-packet.md#github-action-behavior) and
[GitHub Action](github-action.md) for the hosted copy and artifact upload flow.

## Common Mistakes

| Mistake | Correct reading |
| --- | --- |
| "Advisory proof missing, therefore CI failed." | Advisory gaps are visible limits, not required failures, unless policy promoted that proof. |
| "Packet uploaded, therefore merge-ready." | Packets package evidence; they do not prove correctness, safety, or merge approval. |
| "Unavailable gate passed." | Unavailable means the evidence source could not run or was not present; it is not a pass. |
| "PR evidence packet replaces cockpit review packet." | They index different surfaces. Use the packet that matches your workflow. |

## Related Docs

- [Review packet contract](review-packet.md) — cockpit packet layout and verifier
- [Evidence packet](evidence-packet.md) — PR evidence packet field reference
- [Evidence packet workflow spec](specs/evidence-packet-workflow.md) — normative
  producer/verifier contract
- [Cockpit proof evidence](cockpit-proof-evidence.md) — importing and reading proof
- [Packet workflows](packet-workflows.md) — Action and CI adoption paths
- [ub-review ↔ tokmd packet integration](ub-review-integration.md) — consumer-side
  status taxonomy, cache/receipt, and manifest trust order for the ub-review lane
- [ADR-0015: ub-review partial packet consumption](adr/0015-ub-review-partial-packet-consumption.md) —
  durable decision: consume `partial`, fail workflow only when explicitly required
