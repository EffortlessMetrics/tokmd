# ub-review ↔ tokmd Packet Integration

Status: consumer-side integration guide. This document describes how an
`ub-review` lane **calls and consumes** a tokmd evidence packet. The
**producer** recipe (how to run tokmd to write the packet) lives in
[ub-review tokmd sensor recipe](integrations/ub-review.md). General packet
reading rules live in [Packet consumption guide](packet-consumption.md).

This doc does not introduce a new command or schema version. It indexes the
contract that already exists in [Evidence packet](evidence-packet.md) and
[Evidence packet workflow spec](specs/evidence-packet-workflow.md), narrowed to
the ub-review consumer.

## Where this fits

`ub-review` runs `tokmd` as a bounded evidence sensor inside an **advisory**
lane (see [UB-Review single-tight CI gate](specs/ub-review-ci-gate.md)). The
relationship is one-directional:

```text
ub-review lane
  ├─ produces: sensors/tokmd/ packet   (tokmd packet generate / evidence-packet)
  └─ consumes: sensors/tokmd/manifest.json + receipts   (this doc)
```

The packet is review evidence, not a verdict. `tokmd` never decides whether
undefined behavior exists or is absent; ub-review must not present a green
packet as a UB-absence proof or a merge gate.

## How ub-review calls the packet

The advisory lane attaches the PR evidence packet family at `sensors/tokmd/`
(schema `tokmd.evidence-packet/v1`). Produce it with the orchestrator:

```bash
tokmd packet generate \
  --base "$BASE" \
  --head "$HEAD" \
  "$@"
```

`$@` is the changed review scope (native-boundary paths for Bun UB). See the
producer recipe for the `bun-ub` preset, the per-step form, and the Windows
PowerShell UTF-8 write paths. The lane then **consumes** the manifest using the
trust order below.

## Status taxonomy

ub-review consumers must read two independent axes and never collapse them: the
packet `status` (is the packet itself valid?) and the per-artifact advisory
state (was an optional signal produced?).

### Packet status (`manifest.json` `status`)

| `status` | Read as | ub-review action | Exit |
| --- | --- | --- | --- |
| `complete` | Required artifacts exist, refs resolved, scoped analysis parsed, `errors` empty. | Attach and review. Not a merge verdict. | `0` |
| `partial` | Required artifacts exist, but named `warnings` bound the evidence (e.g. optional `syntax.json` missing/degraded). | Attach with the warnings surfaced; treat limits as named, not as failure. | `0` |
| `failed` | A required artifact is missing, refs did not resolve, or `analyze.json` could not be parsed. | Do **not** attach a valid-looking packet. Regenerate or omit. | non-zero (manifest still written for inspection) |

Required artifacts: `analyze_md`, `analyze_json`, `context_md`. Optional
advisory artifact: `syntax_json`.

### Advisory artifact state

For optional signals (today: `syntax_json` / `review_priority`), distinguish:

| State | Meaning | ub-review action |
| --- | --- | --- |
| **skipped** | Optional artifact was not requested (e.g. binary built without the `ast` feature, syntax step omitted). | Expected. Packet stays `complete`/`partial`; do not call it a failure. |
| **advisory-missing** | Optional artifact was requested for this run but is absent. | Packet degrades to `partial` with a named warning. Read it as a bounded limit, not a required-proof failure, unless the repo explicitly promoted that signal to required. |

Neither state is a UB finding and neither is a merge blocker. An advisory gap is
a visible limit, not a red gate. (See the shared evidence-state vocabulary in
[Packet consumption guide](packet-consumption.md#evidence-state-glossary).)

## Claim boundary: what to trust first in `manifest.json`

Read the manifest top-down in this order; stop and reject early if an upstream
field invalidates the packet.

1. **`status` + `errors`** — gate validity. `failed` (or any `errors`) means
   invalid evidence; stop here.
2. **`schema` + `tokmd_version`** — confirm `tokmd.evidence-packet/v1` and a
   known binary version before interpreting other fields.
3. **`base` + `head` + `paths`** — confirm the packet matches the PR's diff
   window and review scope. A mismatch is **stale** evidence: re-run the
   `reproduce` commands against the current head before trusting it.
4. **`warnings` + `non_claims`** — the named limits and the claims the packet
   explicitly does not make. Surface these in any advisory note.
5. **`artifacts`** — the indexed receipts. `analyze.json` is the machine
   receipt for bots/ledgers; `analyze.md` is the first-read human summary;
   `context.md` shows which source was included, truncated, or skipped.
6. **`review_priority`** — advisory reading order only. Open the referenced
   `syntax_json` entries before making any review claim; never treat a priority
   rank as a finding or verdict.

Fail closed: if a required field is missing, treat the packet as invalid rather
than guessing. Ignore unknown fields (producers may add non-breaking fields).

## Cache and receipt guidance

- **Receipts are the evidence; the manifest is the index.** Trust `analyze.json`
  (and `syntax.json` when present) as the receipts; `manifest.json` only points
  at them and records status/scope.
- **Cache by identity, not by presence.** A reusable packet is keyed by
  `(schema, tokmd_version, base, head, paths, preset)`. Reuse a prior
  `sensors/tokmd/` packet only when all of those match the current run;
  otherwise it is stale and must be regenerated.
- **Regenerate from `reproduce`.** The `reproduce` array records copy-ready
  commands scoped to the same paths. Use it (or `tokmd packet generate` with the
  same base/head/preset/paths) to refresh evidence rather than hand-editing
  artifacts.
- **Never cache a `failed` packet as a pass.** A `failed` manifest is written to
  disk for inspection, not for reuse as a green signal.
- **Advisory lane posture.** Because the ub-review tokmd step is
  `continue-on-error` / advisory, a missing or degraded packet must surface as a
  what/why/fix note in the job summary, never as a bare red required check. The
  required floor remains `Tokmd Rust Result`.

## Claim boundary (lane level)

Consuming this packet establishes:

- what changed, whether the requested refs resolved, where review risk
  concentrates, and what source context was included/skipped;
- a reproducible, scoped evidence index for the bot, reviewer, and next agent.

It does **not** establish:

- that undefined behavior exists or is absent, or memory safety;
- merge readiness beyond the deterministic core floor;
- that LLM review ran (advisory, same-repo only; fork PRs skip it);
- CI proof, coverage, mutation, fuzz, release, signing, or publish results.

## Related docs

- [ub-review tokmd sensor recipe](integrations/ub-review.md) — producer recipe
  (how to run tokmd to write the packet)
- [Packet consumption guide](packet-consumption.md) — reading order, evidence-state
  glossary, worked examples
- [Evidence packet](evidence-packet.md) — manifest field reference and status rules
- [Evidence packet workflow spec](specs/evidence-packet-workflow.md) — normative
  producer/verifier contract
- [UB-Review single-tight CI gate](specs/ub-review-ci-gate.md) — advisory lane shape
- [ADR-0015: ub-review partial packet consumption](adr/0015-ub-review-partial-packet-consumption.md) — durable decision for `partial` vs `failed`
- [Bun UB analysis preset](analyze/bun-ub.md) — the `bun-ub` preset signals
