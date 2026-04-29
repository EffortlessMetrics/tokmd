# Spec: Path trust model

Status: Draft

## Scope
Native and in-memory path validation, root bounding, traversal rejection, and diagnostics contract.

## Examples
- "" and "." => root/rootless
- "src" => bounded subtree
- "../x", "/src", "/" => reject
