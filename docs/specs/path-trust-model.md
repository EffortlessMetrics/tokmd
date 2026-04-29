# Spec: Path Trust Model

## Status
Draft.

## Contract
Define root bounding, Git-listed path constraints, in-memory path handling, absolute/parent traversal rejection, and rootless semantics.

## Examples
- `""` -> root/rootless
- `"."` -> root/rootless
- `"src"` -> scoped subtree
- `"../x"` -> reject
- `"/src"` -> reject
- `"/"` -> reject
