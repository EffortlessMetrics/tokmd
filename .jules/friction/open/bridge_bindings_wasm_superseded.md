---
id: bridge_bindings_wasm_superseded
persona: bridge
style: explorer
shard: bindings-targets
status: learning
---
# Redundant Fix Superseded

The intended patch updating the `web/runner` message validation logic to support `paths` and `scan` structures in addition to `inputs` was found to be superseded by #1594 during execution.

As per standard workflow, we gracefully abort the redundant product fix and document the workflow edge case via this learning PR.
