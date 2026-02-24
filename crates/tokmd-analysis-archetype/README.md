# tokmd-analysis-archetype

Tiered microcrate for archetype inference used by analysis receipts.

## What it does

- Detects repository archetypes from normalized `ExportData` paths.
- Returns optional `Archetype` values for analysis consumers.

## API

- `detect_archetype(export: &ExportData) -> Option<Archetype>`

## Contract

- `Cargo` + `crates/*` + workspace directories => `Rust workspace`
- `Cargo.toml` + `next.config.*` + `package.json` => `Next.js app`
- `Dockerfile` + Kubernetes manifests => `Containerized service`
- Terraform inputs => `Infrastructure as code`
- `pyproject.toml` => `Python package`
