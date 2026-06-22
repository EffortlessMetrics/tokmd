# tokmd Evidence Packet Contract

Status: implemented. The first packet shape is intended for review bots,
high-risk local review, and coding-agent handoff. `tokmd analyze` and
`tokmd context` emit the underlying receipts; `tokmd evidence-packet` writes
the manifest beside them.

## Purpose

An evidence packet is the stable directory-level contract over a scoped review
run. It lets a bot, reviewer, ledger, or agent consume one packet instead of
scraping Markdown or guessing which receipt belongs to which command.

The packet answers:

- what diff window was requested;
- which paths were in scope;
- which artifacts were produced;
- whether the packet is complete, partial, or failed;
- what warnings, errors, and non-claims bound the evidence;
- how to reproduce the packet.

It is not a merge verdict, CI proof result, UB detector, safety proof, or
release gate.

## Default Layout

For Bun UB review, the packet lives under `sensors/tokmd/`:

```text
sensors/tokmd/
  manifest.json
  analyze.md
  analyze.json
  context.md
  syntax.json        # optional when syntax receipts are produced
```

The manifest is the packet index. The receipts remain the source evidence.

## Manifest Schema

Use this schema identifier for the first packet contract:

```text
tokmd.evidence-packet/v1
```

Required fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `schema` | string | Must be `tokmd.evidence-packet/v1`. |
| `tokmd_version` | string | Version of the `tokmd` binary used for the receipts. |
| `preset` | string | Analysis preset, for example `bun-ub`. |
| `base` | string | Requested base ref. |
| `head` | string | Requested head ref. |
| `paths` | array of strings | Requested changed paths or review scope. |
| `status` | string | Packet status: `complete`, `partial`, or `failed`. |
| `artifacts` | object | Relative packet artifact paths. |
| `review_priority` | array | Optional first-read items derived from packet artifacts. |
| `warnings` | array | Non-fatal packet warnings. |
| `errors` | array | Fatal packet or artifact errors. |
| `non_claims` | array of strings | Claims this packet explicitly does not make. |
| `reproduce` | array of strings | Commands to regenerate the artifacts. |

Recommended artifact keys:

| Key | Path | Meaning |
| --- | --- | --- |
| `analyze_md` | `sensors/tokmd/analyze.md` | Human-first scoped analysis summary. |
| `analyze_json` | `sensors/tokmd/analyze.json` | Machine-readable analysis receipt. |
| `context_md` | `sensors/tokmd/context.md` | Context budget audit for reviewer or agent handoff. |
| `syntax_json` | `sensors/tokmd/syntax.json` | Optional syntax receipt packet for parser-backed review signals. |

Producers may add fields when they do not change the meaning of required
fields. Consumers should ignore unknown fields and fail closed when required
fields are missing.

When `syntax_json` includes `review_signals`, `tokmd evidence-packet` may add a
`review_priority` array. These items are sorted first by syntax signal score,
then severity and path. They are advisory first-read hints for reviewers and
agents. They do not prove reachability, bug presence, safety, or merge
readiness.

## Example

```json
{
  "schema": "tokmd.evidence-packet/v1",
  "tokmd_version": "1.13.1",
  "preset": "bun-ub",
  "base": "origin/main",
  "head": "HEAD",
  "paths": ["src/runtime/api"],
  "status": "complete",
  "artifacts": {
    "analyze_md": "sensors/tokmd/analyze.md",
    "analyze_json": "sensors/tokmd/analyze.json",
    "context_md": "sensors/tokmd/context.md",
    "syntax_json": "sensors/tokmd/syntax.json"
  },
  "review_priority": [
    {
      "rank": 1,
      "path": "src/runtime/api/MarkdownObject.rs",
      "category": "panic_seam",
      "severity": "high",
      "score": 95,
      "kind": "expect_call",
      "reason": "panic-like seam near review scope",
      "evidence": "expect",
      "refs": ["sensors/tokmd/syntax.json#/receipts/0/review_signals/1"]
    }
  ],
  "warnings": [],
  "errors": [],
  "non_claims": [
    "bun-ub packages review evidence; it does not prove UB exists or is absent"
  ],
  "reproduce": [
    "tokmd analyze --preset bun-ub --format md --effort-base-ref origin/main --effort-head-ref HEAD --no-progress src/runtime/api > sensors/tokmd/analyze.md",
    "tokmd analyze --preset bun-ub --format json --effort-base-ref origin/main --effort-head-ref HEAD --no-progress src/runtime/api > sensors/tokmd/analyze.json",
    "tokmd context --budget 64000 src/runtime/api > sensors/tokmd/context.md",
    "tokmd syntax --no-progress src/runtime/api > sensors/tokmd/syntax.json",
    "tokmd evidence-packet --base origin/main --head HEAD src/runtime/api"
  ]
}
```

To produce the whole packet (artifacts plus manifest) in one step, use the
[`tokmd packet generate`](packet-workflows.md) orchestrator:

```bash
tokmd packet generate \
  --base origin/main \
  --head HEAD \
  src/runtime/api
```

To write just the manifest after the analysis and context artifacts already
exist, use `evidence-packet` directly:

```bash
tokmd evidence-packet \
  --base origin/main \
  --head HEAD \
  src/runtime/api
```

The default output is `sensors/tokmd/manifest.json`. The command exits
nonzero for `failed` packets while still writing the manifest so a bot or human
can inspect the named errors.

## Status Rules

Use `complete` only when:

- every required artifact listed in `artifacts` exists;
- explicit base and head refs resolved before analysis;
- `analyze.json` reports successful scoped analysis;
- context generation completed for the requested paths;
- `errors` is empty.

Use `partial` when:

- one or more non-fatal warnings bound the evidence;
- optional artifacts are missing;
- context was generated but a listed source path was skipped by policy;
- analysis completed but some enrichment was unavailable.

Use `failed` when:

- explicit refs do not resolve;
- a required artifact is missing;
- `analyze.json` cannot be parsed;
- the producer cannot determine whether the packet matches the requested paths.

Do not attach a packet marked `complete` when the real state is `partial` or
`failed`.

## Producer Rules

1. Generate `analyze.md`, `analyze.json`, and `context.md` from the same base,
   head, and paths recorded in `manifest.json`.
2. Write paths relative to the repository root or packet root, not absolute
   machine-local paths.
3. Record ref-resolution failures as `failed` packets, or do not attach a
   packet at all.
4. Preserve warnings from `analyze.json`; do not hide them in Markdown-only
   output.
5. Include non-claims that bound the preset. For `bun-ub`, the key non-claim is
   that the packet packages review evidence and does not prove UB exists or is
   absent.
6. Keep reproduction commands copy-ready and scoped to the same paths.
7. When `sensors/tokmd/syntax.json` exists, include it as `syntax_json`; when
   syntax evidence is explicitly requested but missing, keep the packet
   `partial` and name the missing optional artifact.
8. When syntax `review_signals` exist, surface them in `review_priority` with
   refs back to `syntax_json`.
9. Prefer `tokmd evidence-packet` over hand-written manifest glue so preset,
   path, artifact, warning, and status checks stay consistent.

## Consumer Rules

1. Open `manifest.json` first to identify the packet status and artifact list.
2. Treat `status=failed` as invalid evidence.
3. Treat `status=partial` as evidence with named limits, not a pass.
4. Use `analyze.md` for first-read review context.
5. Use `analyze.json` for bot, ledger, and agent ingestion.
6. Use `context.md` to check which source files were included, truncated, or
   skipped before handing the packet to an agent.
7. Use `syntax_json` only as advisory parser evidence; missing or degraded
   syntax evidence is not a proof failure unless your workflow requires it.
8. Use `review_priority` as a reading order, not as a verdict. Open the
   referenced receipt entries before making a review claim.
9. Do not infer CI proof, safety, or whole-repo coverage from this packet.

## Bun UB Non-Claims

A `bun-ub` evidence packet does not:

- detect undefined behavior;
- prove memory safety;
- prove that a change is safe to merge;
- analyze the whole repository unless `.` was supplied as a path;
- replace cockpit review packets;
- execute CI proof;
- promote coverage, mutation, fuzz, release, signing, or publish checks.

## Related Docs

- [PR evidence packet workflows](packet-workflows.md)
- [Bun UB analysis preset](analyze/bun-ub.md)
- [ub-review tokmd sensor recipe](integrations/ub-review.md)
- [Artifact glossary](artifacts.md)
- [Review packet contract](review-packet.md)
- [Handoff bundles](handoff.md)
