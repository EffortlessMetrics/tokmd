# tokmd-export-tree

Single-responsibility microcrate for deterministic tree rendering from
`tokmd_types::ExportData`.

It provides two renderers:

- `render_analysis_tree`: full tree with file leaves and `(lines, tokens)` payloads
- `render_handoff_tree`: directory-only tree with `(files, lines, tokens)` payloads and depth limit

Both outputs are stable across input order for reproducible receipts and tests.
