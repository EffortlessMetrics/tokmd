# Architecture Decision Records

This directory stores accepted architecture, packaging, and product-contract
decisions for tokmd. ADRs explain why a durable decision exists; exact
behavioral contracts belong in specs such as `docs/specification.md` or
surface-specific schema documents.

## Index

| ADR | Status | Title |
|-----|--------|-------|
| [0000](0000-adr-process.md) | accepted | ADR and specification governance |
| [0001](0001-production-package-publishability.md) | accepted | Production package publishability |
| [0002](0002-crate-vs-module-boundaries.md) | accepted | Crate vs module boundaries |
| [0003](0003-publish-surface-taxonomy.md) | accepted | Publish-surface taxonomy |
| [0004](0004-binding-surfaces.md) | accepted | Binding surfaces (Node, Python, WASM) |
| [0005](0005-release-train-and-rc-semantics.md) | accepted | Release train and RC semantics |
| [0006](0006-deterministic-receipts.md) | accepted | Deterministic receipts and renderers |
| [0007](0007-schema-family-versioning.md) | accepted | Independent schema-family versioning |

## House Style

Use the structure defined by [ADR-0000](0000-adr-process.md): context,
decision, consequences, alternatives, enforcement, and related specs.
