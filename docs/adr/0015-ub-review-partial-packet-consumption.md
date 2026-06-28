# ADR-0015: ub-review partial packet consumption

- Status: accepted
- Date: 2026-06-28
- Related specs:
  - `docs/specs/evidence-packet-workflow.md`
  - `docs/specs/ub-review-ci-gate.md`
- Related docs:
  - `docs/ub-review-integration.md`
  - `docs/packet-consumption.md`
  - `docs/evidence-packet.md`
- Related ADRs:
  - `docs/adr/0014-schema-identity-idioms.md`

## Context

tokmd evidence packets (`tokmd.evidence-packet/v1`) expose a manifest `status`
with three values: `complete`, `partial`, and `failed`. Required artifacts are
`analyze_md`, `analyze_json`, and `context_md`. Optional advisory artifact:
`syntax_json` (from `tokmd syntax`).

When optional syntax is skipped (`--no-syntax`, binary without `ast`, or syntax
step omitted), or when syntax was requested but degraded, the producer may mark
the packet `complete` or `partial` with named warnings. The normative producer
contract already exits `0` for `partial` packets
(`docs/specs/evidence-packet-workflow.md`).

The ub-review lane is **advisory** (`continue-on-error: true`; merge floor is
`Tokmd Rust Result` only). Consumers must not collapse two independent axes:

1. **Packet status** — is the packet itself valid evidence?
2. **Advisory artifact state** — was an optional signal produced?

`docs/ub-review-integration.md` and `docs/packet-consumption.md` already
document the intended consumer posture, but
`docs/specs/evidence-packet-workflow.md` left an open question: whether
downstream ub-review should treat missing optional syntax as a workflow failure
independent of packet `partial` status.

Without a durable ADR, operators may wire the tokmd Action with `fail-on:
partial`, treat advisory syntax gaps as merge blockers, or fail closed on
`partial` packets that still carry usable analyze/context evidence.

## Decision

**ub-review consumes `partial` packets; it does not fail the workflow for
optional-syntax gaps unless a repo explicitly promotes syntax to required.**

### Packet status → ub-review action

| `manifest.status` | ub-review action | Workflow exit |
| --- | --- | --- |
| `complete` | Attach and review. Not a merge verdict. | `0` |
| `partial` | Attach with `warnings` and `non_claims` surfaced. Treat limits as named, not as failure. | `0` |
| `failed` | Do **not** attach as valid evidence. Emit advisory what/why/fix note. | `0` (advisory lane; must not fail merge gate) |

Required-artifact absence or ref-resolution failure yields `failed`. That is
invalid evidence, not a UB finding.

### Optional syntax → advisory artifact state

| State | Packet effect | ub-review action |
| --- | --- | --- |
| **skipped** | Syntax not requested (`--no-syntax`, no `ast` feature). May remain `complete`. | Expected. Do not call it a failure. |
| **advisory-missing** | Syntax requested but absent or degraded. Degrades to `partial` with a named warning. | Consume as bounded evidence. Surface the warning. Do not fail the workflow. |

Neither state proves or disproves undefined behavior.

### When a workflow **may** fail on partial

Failure on `partial` is **opt-in**, not the ub-review default:

- **tokmd Action** with `fail-on: partial` — operator choice for strict packet
  gates (`action.yml`, default `fail-on: failed`).
- **Repo policy** that explicitly promotes `syntax_json` to required — outside
  the default evidence-packet contract; must be documented in that repo's policy
  ledger, not assumed by ub-review.

The default ub-review integration uses `fail-on: failed` (or equivalent: only
`failed` packet status may fail a dedicated tokmd Action step). A `partial`
packet from optional syntax degradation must **not** fail the ub-review job or
present as a bare red required check.

### Claim boundary

Consuming a `partial` packet establishes scoped analyze/context evidence with
named limits. It does **not** establish syntax-backed `review_priority` coverage,
UB absence, merge readiness, or that optional parser signals ran successfully.

## Consequences

- ub-review operators attach `partial` packets and surface warnings in advisory
  notes instead of treating syntax gaps as merge blockers.
- `docs/specs/evidence-packet-workflow.md` resolves its open question by
  reference to this ADR.
- Strict workflows that need syntax as required proof must say so explicitly
  (`fail-on: partial` or a promoted required-artifact policy), not inherit it
  from the default contract.
- Integration tests and Action defaults remain aligned: `partial` exits `0` from
  `tokmd evidence-packet` / `tokmd packet generate`; Action fails only on
  `partial` when `fail-on: partial`.

## Alternatives

### A. Fail ub-review workflow on any `partial` packet

Rejected. Most `partial` cases are optional-syntax degradation while required
analyze/context evidence is valid. Failing the advisory lane would noise the
merge gate posture and contradict `docs/specs/ub-review-ci-gate.md`.

### B. Treat skipped syntax as `failed`

Rejected. Skipped optional evidence is expected when syntax is not requested.
Marking the packet or workflow failed would confuse advisory-missing with
required-missing (see `docs/packet-consumption.md` evidence-state glossary).

### C. Documentation only (no ADR)

Rejected. The open question in the workflow spec needed a durable decision owner
before more consumers wire packet status into CI.

## Enforcement

- Consumer guidance: `docs/ub-review-integration.md` (status taxonomy and cache
  rules).
- Producer/verifier contract: `docs/specs/evidence-packet-workflow.md`.
- Advisory lane shape: `docs/specs/ub-review-ci-gate.md`.
- Action failure policy: `action.yml` `fail-on` input (`failed` default).
- Evidence-state vocabulary: `docs/packet-consumption.md`.

No new schema version or command surface is introduced by this ADR.

## Related specs

- `docs/specs/evidence-packet-workflow.md` — packet status semantics and
  producer exit codes
- `docs/specs/ub-review-ci-gate.md` — advisory lane must not fail merge
