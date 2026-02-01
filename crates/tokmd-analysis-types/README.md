# tokmd-analysis-types

Analysis receipt contracts for tokmd.

## Overview

This is a **Tier 0** crate defining pure data structures for analysis receipts. It contains no I/O or business logic - only type definitions and serialization.

## Installation

```toml
[dependencies]
tokmd-analysis-types = "1.3"
```

## Key Types

### Core Receipt
```rust
pub struct AnalysisReceipt {
    pub schema_version: u32,
    pub archetype: Option<Archetype>,
    pub topics: Option<TopicClouds>,
    pub entropy: Option<EntropyReport>,
    pub derived: Option<DerivedReport>,
    pub git: Option<GitReport>,
    // ... and more optional sections
}
```

### Analysis Result Types

| Type | Purpose |
|------|---------|
| `Archetype` | Project kind detection (CLI, library, web app) |
| `TopicClouds` | Semantic topic extraction with TF-IDF scores |
| `EntropyReport` | High-entropy file detection |
| `PredictiveChurnReport` | Git-based change trend prediction |
| `CorporateFingerprint` | Author domain statistics |
| `LicenseReport` | SPDX license detection |
| `DerivedReport` | Core metrics (density, distribution, COCOMO) |
| `AssetReport` | Non-code file categorization |
| `GitReport` | Hotspots, bus factor, freshness, coupling |
| `ImportReport` | Module dependency graph |
| `DuplicateReport` | Content duplication detection |
| `FunReport` | Eco-label and novelty outputs |

## Schema Version

```rust
pub const ANALYSIS_SCHEMA_VERSION: u32 = 4;
```

v4 added cognitive complexity, nesting depth, and function-level details.

## Design Principles

- All analysis sections are `Option<T>` to support preset-based inclusion
- Uses `BTreeMap` for deterministic key ordering
- No I/O operations - pure data definitions

## License

MIT OR Apache-2.0
