# Handoff Bundles

`tokmd handoff` creates a **self-contained source/context bundle** for LLM
review and automation. It is intended to be pasted or uploaded as a stable,
deterministic artifact when a coding agent needs the right source slice without
the whole repository.

When you pass review, proof, or evidence-packet inputs, the bundle also writes
link artifacts that point at adjacent evidence. Those links are handles, not
copied proof. The review-packet verifier, proof receipts, and evidence packet
manifest remain the sources of evidence truth.

Use handoff when the job is:

```text
Give my coding agent the right context and proof expectations.
```

## CLI

```bash
# Default output to .handoff/
tokmd handoff

# Custom output directory
tokmd handoff --out-dir ./artifacts/handoff

# Control token budget and strategy
tokmd handoff --budget 128k --strategy spread

# Disable git enrichment
tokmd handoff --no-git
```

## Agent Workflow

For a plain repository handoff, start with the current risk preset and a bounded
budget:

```bash
tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --out-dir .handoff
```

Give the agent these files in order:

1. `.handoff/work-order.md` for the agent task map, best-effort linked
   evidence summary, evidence handles, and guardrails.
2. `.handoff/manifest.json` for the authoritative artifact index, token budget,
   exclusions, included-file list, and packet-local hashes.
3. `.handoff/intelligence.json` for tree, hotspot, complexity, and derived
   signals.
4. `.handoff/code.txt` for the selected source bundle.
5. `.handoff/map.jsonl` when the agent needs full inventory or path lookup.

For PR repair or review work in this repository, pair the handoff with cockpit
and proof receipts:

```bash
cargo xtask ci-plan \
  --base origin/main \
  --head HEAD \
  --labels-json "[]" \
  --json-out target/ci/ci-plan.json \
  --route-json-out target/ci/proof-pack-route.json \
  --no-budget-annotations

cargo xtask affected \
  --base origin/main \
  --head HEAD \
  --json-output target/proof/affected.json

cargo xtask proof --profile affected --base origin/main --head HEAD --plan \
  --plan-json target/proof/proof-plan.json

cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json

tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --doc-artifacts-check target/docs/doc-artifacts-check.json \
  --proof-route target/ci/proof-pack-route.json \
  --review-packet-dir .tokmd/review

cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json

tokmd handoff \
  --preset risk \
  --budget 128k \
  --strategy spread \
  --review-packet-dir .tokmd/review \
  --review-packet-check target/tokmd/review-packet-check.json \
  --affected target/proof/affected.json \
  --proof-plan target/proof/proof-plan.json \
  --out-dir .handoff
```

When `--proof-route` is omitted, `tokmd handoff` looks for the packet-local
`.tokmd/review/proof/proof-pack-route.json` copied by `tokmd cockpit
--proof-route ... --review-packet-dir ...`. Pass `--proof-route <path>` to
override that packet-local route with another receipt.

Then give the agent the handoff plus the linked review evidence:

- `.handoff/work-order.md` for the ordered agent work map, compact linked
  evidence summary, and evidence guardrails.
- `.tokmd/review/comment.md` for the short review summary.
- `.tokmd/review/review-map.md` for what to inspect first and reproduction
  commands.
- `.tokmd/review/evidence.json` for available, missing, stale, degraded,
  skipped, and unavailable evidence.
- `target/ci/proof-pack-route.json` or the packet-local
  `.tokmd/review/proof/proof-pack-route.json` for changed-file proof-pack
  routing, unmatched files, skipped-by-policy lanes, and static or learned
  skipped-lane estimate telemetry.
- `target/proof/affected.json` for changed files and matched proof scopes.
- `target/proof/proof-plan.json` for expected proof commands.
- `target/tokmd/review-packet-check.json` for packet verification.
- `sensors/tokmd/manifest.json` for a tokmd evidence packet that can provide
  status, warnings/errors, scoped paths, reproduction commands, and
  `review_priority`.
- `.handoff/review-links.json` for packet-local pointers to the cockpit review
  packet and verifier receipt.
- `.handoff/proof-links.json` for packet-local pointers to proof-route,
  affected-proof, and proof-plan receipts.
- `.handoff/evidence-packet-links.json` for a packet-local pointer to the
  evidence packet manifest.

## Consuming Linked Evidence

Use linked evidence as a review map, not as a hidden assertion that the handoff
has already verified everything:

1. Read `.handoff/review-links.json`, `.handoff/proof-links.json`, and
   `.handoff/evidence-packet-links.json` when present to find the adjacent
   receipt paths. If a linked path has `exists: false`, treat that evidence as
   missing until it is regenerated.
2. Open `target/tokmd/review-packet-check.json` before trusting a linked review
   packet. If the verifier receipt is absent, rerun:

   ```bash
   cargo xtask review-packet-check --dir .tokmd/review --json target/tokmd/review-packet-check.json
   ```

3. Use the `Linked Evidence Summary` section in `.handoff/work-order.md` as a
   quick triage view of readable review/proof/evidence-packet receipts. The
   `Changed Surfaces`, `Review Evidence`, `Proof Expectations`,
   `Missing / Stale / Degraded Evidence`, and `Agent Stop Conditions` sections
   turn those handles into the agent work order. Open the linked receipts for
   source-of-truth details. Missing, stale, degraded, skipped, unavailable,
   partial, or failed evidence is a task for the agent, not passing proof.
4. If the review-packet verifier summary lists verified packet-local
   `proof/*.json` artifacts, treat that as hash-verified packet inventory. It
   identifies copied route/proof receipts but does not mean those receipts were
   executed or promoted. Handoff requires the artifact schema and media type to
   avoid treating arbitrary packet-local files under `proof/` as proof receipts.
5. Use `.tokmd/review/review-map.md` for review order and reproduction
   commands.
6. Use the proof route linked from `.handoff/proof-links.json` to see
   changed-file route ownership, unmatched files, and skipped-by-policy lanes.
   By default this can be the packet-local
   `.tokmd/review/proof/proof-pack-route.json`; pass `--proof-route` to handoff
   when a different route receipt should own the link. A proof route is routing
   and skip-policy evidence; it is not an execution result and skipped lanes
   are not passing proof.
7. Use `target/proof/affected.json` to see which proof scopes matched the
   change and `target/proof/proof-plan.json` to see expected commands. A proof
   plan is planned evidence; it is not an execution result.
8. If `.handoff/evidence-packet-links.json` is present, open the linked
   `sensors/tokmd/manifest.json`. Use `review_priority` as first-read order,
   not as a verdict, and resolve or explicitly acknowledge `partial`/`failed`
   packet status before claiming complete review evidence.
9. Keep the regenerated receipts with the repair so reviewers can follow the
   same handles from handoff to review packet to proof artifacts.

The link artifacts do not copy, normalize, or verify external receipts. They
make the handoff bundle point at adjacent review/proof/evidence-packet evidence
while preserving the review-packet verifier, proof artifacts, and evidence
packet manifest as their own evidence sources.

## Output Tree

```
<out-dir>/
├── manifest.json      # authoritative index (schema v5)
├── work-order.md      # agent work map, evidence summary, and stop conditions
├── map.jsonl          # full file inventory (JSONL)
├── intelligence.json  # summary signals (payload-only)
├── code.txt           # token-budgeted code bundle
├── review-links.json  # optional linked cockpit review packet artifacts
├── proof-links.json   # optional linked proof-route/affected/proof-plan artifacts
└── evidence-packet-links.json # optional linked tokmd evidence packet manifest
```

## Consumption Pattern

1. **Read `work-order.md` first.**
   It is the agent-facing task map, changed surfaces,
   best-effort linked evidence summary, proof expectations, missing evidence,
   stop conditions, and guardrails.
2. **Use `manifest.json` as the authoritative index.**
   It lists artifacts, included files, exclusions, token-budget state, and
   packet-local hashes.
3. **Use `map.jsonl`** for full inventory or downstream tooling.
4. **Use `intelligence.json`** as a warning label (tree, hotspots, derived).
5. **Use `code.txt`** as the LLM bundle content.

## Agent Guardrails

When using handoff output as an agent work order:

- Treat missing, stale, degraded, skipped, or unavailable evidence as work to
  resolve, not as passing proof.
- Run reproduction commands from `.tokmd/review/review-map.md` before claiming a
  repair is proven.
- Keep generated receipts with the work when they explain review or proof state.
- Do not promote advisory proof, enable default Codecov upload, or turn cockpit
  into a merge verdict from a handoff bundle.

## Determinism Notes

- Output directory is excluded from scans by construction.
- All selection strategies and ordering are deterministic.

## Schema

See `docs/handoff.schema.json` and `docs/handoff-schema.md`.
