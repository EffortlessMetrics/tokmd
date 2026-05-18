# Handoff Tree

Use this when your job is:

```text
Prepare a coding-agent handoff.
```

Run first:

```bash
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

Sample layout:

```text
.handoff/
  manifest.json
  work-order.md
  code.txt
  map.jsonl
  intelligence.json
  review-links.json
  proof-links.json
```

Open first:

1. `.handoff/work-order.md`
2. `.handoff/manifest.json`
3. `.handoff/code.txt`
4. `.handoff/review-links.json`
5. `.handoff/proof-links.json`

What each file owns:

| File | Owns |
| --- | --- |
| `work-order.md` | Agent starting brief, linked evidence summary, and guardrails. |
| `manifest.json` | Bundle inventory, token budget, included files, excluded files, and hashes. |
| `code.txt` | Token-budgeted source bundle. |
| `map.jsonl` | Full path inventory for lookup. |
| `intelligence.json` | Repo shape, hotspot, and derived analysis signals. |
| `review-links.json` | Pointers to external review packet artifacts and verifier receipt. |
| `proof-links.json` | Pointers to external affected-proof and proof-plan artifacts. |

What not to infer:

- The handoff does not verify external review or proof receipts.
- The source bundle is not necessarily the whole repository.
- A proof plan is not executed proof.
- Linked missing, stale, degraded, skipped, or unavailable evidence is agent
  work, not a pass.

Next action:

- Give the agent `work-order.md` first.
- Tell the agent to use `code.txt` as the bounded source bundle.
- Verify linked review and proof receipts with their own checkers.
