# Review Packet Tree

Use this when your job is:

```text
Review this PR.
```

Run first:

```bash
tokmd cockpit \
  --base origin/main \
  --head HEAD \
  --review-packet-dir .tokmd/review

cargo xtask review-packet-check \
  --dir .tokmd/review \
  --json target/tokmd/review-packet-check.json
```

Sample layout:

```text
.tokmd/review/
  manifest.json
  cockpit.json
  evidence.json
  comment.md
  review-map.json
  review-map.md
  docs/
    doc-artifacts-check.json
  proof/
    proof-run-summary.json
    proof-run-observation.json
    proof-executor-observation.json
    coverage-receipt.json

target/tokmd/
  review-packet-check.json
```

Open first:

1. `.tokmd/review/review-map.md`
2. `.tokmd/review/comment.md`
3. `.tokmd/review/evidence.json`
4. `target/tokmd/review-packet-check.json`

What each file owns:

| File | Owns |
| --- | --- |
| `review-map.md` | Human review order, reasons, evidence state, and reproduction commands. |
| `comment.md` | Compact PR-comment-ready summary. |
| `evidence.json` | Exact evidence availability: present, missing, stale, degraded, skipped, or unavailable. |
| `manifest.json` | Packet-local inventory and hashes. |
| `cockpit.json` | Full machine-readable cockpit receipt. |
| `review-packet-check.json` | Verifier receipt for packet-local schemas, paths, and hashes. |

What not to infer:

- The packet is not a merge verdict.
- Missing evidence is not passing proof.
- Advisory proof does not become required because it appears in the packet.
- The verifier receipt checks packet-local artifacts, not external CI state.

Next action:

- Run reproduction commands from `review-map.md`.
- Repair missing, stale, or degraded evidence before claiming the packet is
  complete.
- Keep the verifier receipt with the review artifacts when sharing the packet.
