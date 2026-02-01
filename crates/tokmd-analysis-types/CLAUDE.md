# tokmd-analysis-types

## Purpose

Pure data structures for analysis receipts. This is a **Tier 0** crate defining the analysis contract.

## Responsibility

- Analysis-specific receipt types and findings
- No I/O, no business logic
- Defines schema for all analysis outputs

## Public API

### Core Receipt
- `AnalysisReceipt` - Top-level analysis output with all optional sections
- `AnalysisSource` - Source receipt reference (lang/module/export)
- `AnalysisArgsMeta` - Captured analysis arguments

### Analysis Result Types

| Type | Purpose |
|------|---------|
| `Archetype` | Project kind detection with evidence |
| `Topics` | Semantic topic clouds with TF scores |
| `EntropyFinding` / `EntropyClass` | High-entropy file detection |
| `ChurnTrend` / `TrendClass` | Predictive churn by module |
| `CorporateFingerprint` | Domain statistics from commits |
| `LicenseRadar` / `LicenseFinding` | License detection and analysis |
| `Derived` | Core metrics (density, distribution, COCOMO) |
| `Assets` | Asset discovery and categorization |
| `Dependencies` | Lockfile reporting |
| `Hotspots` / `BusFactor` / `Freshness` / `Coupling` | Git-derived metrics |
| `ImportGraph` | Import/dependency edges |
| `Duplicates` | Content duplication detection |
| `EcoLabel` | Fun eco-label scoring |

### Enums
- `EntropyClass` - Low, Medium, High, Suspicious
- `TrendClass` - Stable, Rising, Falling, Volatile
- `EffectiveLicense` - Detected license type

## Implementation Details

### Schema Version
```rust
pub const ANALYSIS_SCHEMA_VERSION: u32 = 4;
```
v4 added cognitive complexity, nesting depth, and function-level details to `ComplexityReport`.

### Optional Fields
All analysis sections are `Option<T>` to support preset-based inclusion:
```rust
pub struct AnalysisReceipt {
    pub archetype: Option<Archetype>,
    pub topics: Option<Topics>,
    pub entropy: Option<Vec<EntropyFinding>>,
    // ...
}
```

### Sorted Collections
Use `BTreeMap` for deterministic key ordering in maps.

## Dependencies

- `serde` with derive feature
- `tokmd-types` (base types)

## Testing

- Serde roundtrip tests
- Run: `cargo test -p tokmd-analysis-types`

## Do NOT

- Add analysis computation logic (belongs in tokmd-analysis)
- Add formatting logic (belongs in tokmd-analysis-format)
- Add I/O operations
