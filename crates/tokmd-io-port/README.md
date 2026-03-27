# tokmd-io-port

Read-only file access ports for tokmd.

## Problem
Scan and model code should work against disk and in-memory inputs, including WASM, without changing call sites.

## What it gives you
- `ReadFs`
- `HostFs`
- `MemFs`
- `MemFsError`

## API / usage notes
- Use `HostFs` for production filesystem access.
- Use `MemFs` in tests and browser/WASM code.
- This crate stays read-only by design.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [tokmd-scan](../tokmd-scan/README.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
