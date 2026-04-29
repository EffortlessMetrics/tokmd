# Spec: Path Trust Model

Status: Draft

## Scope

Defines accepted/rejected path forms and trust boundaries for native, MemFs, and FFI inputs.

## Required rules

- Reject absolute and parent-traversal paths.
- Define root/rootless semantics for empty and dot paths.
- Define diagnostics for missing and dirty-index path states.
