---
id: run-palette-prover-dx-wasm-contract
persona: palette
style: prover
shard: bindings-targets
status: open
---

# WASM Capability Matrix Blocks `scan` and `paths` in Web Runner
The `web/runner` intentionally strictly validates `inputs` rather than supporting `paths` or `scan` because the WASM capability matrix (`docs/capabilities/wasm.json`) marks those native payloads as unsupported in browser runners. Attempts to relax payload validation in `isRunMessage` must be avoided to keep contracts in sync.
