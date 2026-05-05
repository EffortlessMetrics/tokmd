---
id: bridge_bindings_wasm_superseded
persona: Bridge
style: Explorer
shard: bindings-targets
status: open
---

## Description
An intended patch to sync the `browser-runner` input location (`args.scan.inputs`) with the Rust core FFI was aborted.

## Impact
Wasted execution cycles.

## Workaround
Gracefully fell back to creating a learning PR.

## Evidence
- Received PR comment: "Superseded by #1594, which merged the current browser runner args.scan.inputs parity synthesis with strict validation and worker coverage."
