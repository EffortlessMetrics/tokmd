---
id: superseded_pr
persona: Bridge
style: Explorer
shard: bindings-targets
status: open
---
# Description
A pull request fixing `scan.inputs` parity for browser runner inputs was superseded by #1594 which merged a similar implementation alongside strict validation and worker coverage.
# Impact
Workflow interruption, duplicate effort during independent async generation.
# Resolution
Gracefully aborting and generating a learning PR.
