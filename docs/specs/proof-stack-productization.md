# Spec: Proof-stack productization (deferred)

- Status: draft
- Schema family: none (no tokmd proof-stack profile schema yet)
- Related ADRs: `docs/adr/0013-proof-stack-productization-boundary.md`
- Related proof scopes: n/a until a profile is proposed
- Tracking: issue #223; ripr-swarm `RIPR-PROP-0015`, `RIPR-SPEC-0060`

## Contract

This document records the **deferred** ripr→tokmd proof-stack contract so the
deferral is tracked with an explicit next action instead of dead-ending in ripr
docs alone.

**Current tokmd posture (accepted):** no proof-stack CLI or profile exists.
ripr's repo-local traceability stack remains the prototype.

### Deferred ripr command surface

These commands were named in ripr closeout artifacts and are **not** implemented
in tokmd:

| Command | Intended role (ripr prototype) |
| --- | --- |
| `tokmd init proof-stack` | Bootstrap profile / traceability.toml scaffold |
| `tokmd check --profile proof-stack` | Verify spec→test→code→output→metric chain |
| `tokmd graph` | Emit traceability graph for changed scope |
| `tokmd next` | Suggest next proof/traceability work item |
| `tokmd explain` | Explain a traceability edge or gap |

### ripr prototype (source of truth today)

| Artifact | Owner repo | Role |
| --- | --- | --- |
| `xtask check-traceability` | ripr-swarm | Machine-checked traceability verifier |
| `traceability.toml` | ripr-swarm | Profile and edge declarations |
| Policy ledgers under `policy/` | ripr-swarm | Exception and suppression routing |

tokmd's adjacent surfaces (`cargo xtask affected`, `cargo xtask proof`,
`ci/proof.toml`, proof-observation receipts) are **not** drop-in replacements
for this profile until a portability spec exists.

## Inputs

N/A — no tokmd command accepts a proof-stack profile today.

## Outputs

N/A — no tokmd receipt family is defined for proof-stack.

## Compatibility

- ripr may continue to reference these deferred commands in its docs as
  **future tokmd productization**, not as current tokmd behavior.
- tokmd docs must not imply the commands exist.
- If productization proceeds, the first deliverable is a **spec proposal** (not
  CLI stubs) covering schema owner, verifier entrypoints, and overlap with
  existing `xtask` proof planning.

## Next action gate

Reopen productization only when **all** of the following are true:

1. A maintainer accepts a written proposal that addresses RIPR-PROP-0015 overfit
   risk (portable contracts vs ripr-local selectors).
2. ripr's use-case spec layer (RIPR-SPEC-0065 family) or lifecycle dashboard
   (#1040) defines concrete consumer requirements tokmd `xtask` cannot satisfy
   without a named profile.
3. The proposal names: profile TOML schema owner, verifier command(s), proof
   scope routing in `ci/proof.toml`, and rollback if ripr-local checks remain
   sufficient.

Until then, status stays **deferred**. File a new issue when the gate is met;
do not silently add CLI stubs.

## Proof Requirements

None for the deferred state. When a profile is proposed, proof must include:

```bash
cargo xtask doc-artifacts --check
cargo xtask proof-policy --check
```

plus targeted tests for any new verifier introduced by the proposal.

## Open questions

- Whether proof-stack generalizes as a tokmd **profile** on existing `xtask
  check` infrastructure vs a separate command family.
- How ripr repair-packet and use-case-spec lifecycles map to tokmd proof scopes
  without duplicating `cargo xtask proof` planning.
- Whether cross-repo profile schemas live in tokmd-types, a shared spec repo, or
  remain ripr-owned with tokmd as optional renderer/verifier.
