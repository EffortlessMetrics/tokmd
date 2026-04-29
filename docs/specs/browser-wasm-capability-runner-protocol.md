# Spec: Browser/WASM Capability + Runner Protocol

Status: Draft

## Scope

Defines capability matrix semantics and worker protocol expectations.

## Required coverage

- `browser_safe`/`rootless_safe` meanings.
- Allowed modes/payload shapes.
- Runner message schema (`protocolVersion`, `mode`, `args`, error shape).
- Change-control invariant for payload widening.
