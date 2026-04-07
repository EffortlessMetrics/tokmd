#!/bin/bash
DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)
UUID=$(python3 -c "import uuid; print(uuid.uuid4())")
DATE_ONLY=$(date -u +%Y-%m-%d)
cat << ENVELOPE > .jules/bolt/envelopes/$UUID.json
{
  "run_id": "$UUID",
  "timestamp_utc": "$DATE",
  "lane": "scout",
  "target": "tokmd-content complexity allocations",
  "proof_method": "structural",
  "commands": [],
  "results_summary": ""
}
ENVELOPE

cat << RUNLOG > .jules/bolt/runs/$DATE_ONLY.md
# Bolt Run $DATE_ONLY

**Read Context:**
- CI config
- Docs

**Lane:** Scout
**Target:** tokmd-content complexity formatting (avoid .lines().collect::<Vec<&str>>())
**Proof:** structural (O(N) allocations eliminated)

**Options considered:**
### Option A
Refactor \`tokmd-content/src/complexity.rs\` to iterate over \`.lines()\` directly instead of collecting to \`Vec<&str>\`.
- **Trade-offs**: Simple, eliminates allocations.

### Option B
Wait for a larger refactor of \`tokmd-content\`.
- **Trade-offs**: Leaves performance win on the table.

**Decision:** Option A. Easy win to reduce heap allocations on hot path.

**Receipts:**
(Will be filled after checks)
RUNLOG
