# tokmd-analysis

Analysis logic and enrichers for tokmd receipts.

## Overview

This is a **Tier 3** orchestration crate that computes derived metrics and optional enrichments from code inventories. It coordinates multiple analysis modules based on preset configuration.

## Installation

```toml
[dependencies]
tokmd-analysis = "1.4"

# Enable optional features
[dependencies.tokmd-analysis]
version = "1.4"
features = ["git", "walk", "content", "fun", "topics", "archetype"]
```

## Usage

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use tokmd_analysis::{analyze, AnalysisRequest, AnalysisContext, AnalysisLimits, AnalysisPreset, ImportGranularity};
use tokmd_types::{ExportData, ChildIncludeMode};
use tokmd_analysis_types::{AnalysisSource, AnalysisArgsMeta, NearDupScope};
use std::path::PathBuf;

let export_data = ExportData {
    rows: vec![],
    module_roots: vec![],
    module_depth: 0,
    children: ChildIncludeMode::ParentsOnly,
};

let request = AnalysisRequest {
    preset: AnalysisPreset::Risk,
    args: AnalysisArgsMeta {
        preset: "risk".to_string(),
        format: "md".to_string(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".to_string(),
    },
    limits: AnalysisLimits::default(),
    window_tokens: None,
    git: None,
    import_granularity: ImportGranularity::Module,
    detail_functions: false,
    near_dup: false,
    near_dup_threshold: 0.8,
    near_dup_max_files: 1000,
    near_dup_scope: NearDupScope::Lang,
    near_dup_max_pairs: None,
    near_dup_exclude: vec![],
};

let context = AnalysisContext {
    export: export_data,
    root: PathBuf::from("."),
    source: AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec![],
        module_depth: 0,
        children: "parents-only".to_string(),
    },
};

let receipt = analyze(context, request)?;
# Ok(())
# }
```

## Analysis Presets

| Preset | Includes |
|--------|----------|
| `Receipt` | Core derived metrics (density, distribution, COCOMO) |
| `Health` | + TODO density |
| `Risk` | + Git hotspots, coupling, freshness |
| `Supply` | + Assets, dependency lockfiles |
| `Architecture` | + Import graph |
| `Topics` | Semantic topic clouds (TF-IDF) |
| `Security` | License radar, entropy profiling |
| `Identity` | Archetype detection, corporate fingerprint |
| `Git` | Predictive churn, advanced git metrics |
| `Deep` | Everything (except fun) |
| `Fun` | Eco-label, novelty outputs |

## Analysis Modules

| Module | Feature | Purpose |
|--------|---------|---------|
| `archetype` | archetype | Project kind detection |
| `derived` | - | Core metrics |
| `topics` | topics | Semantic keyword extraction |
| `entropy` | content+walk | High-entropy file detection |
| `license` | content+walk | License radar scanning |
| `fingerprint` | git | Corporate domain analysis |
| `churn` | git | Git-based change prediction |
| `assets` | walk | Asset categorization |
| `git` | git | Hotspots, bus factor, freshness |
| `content` | content | TODOs, duplicates, imports |

## Feature Flags

```toml
[features]
git = ["tokmd-git"]       # Git history analysis
walk = ["tokmd-walk"]     # Asset discovery
content = ["tokmd-content"]  # Content scanning
topics = ["tokmd-analysis-topics"] # Topic-cloud extraction
archetype = ["tokmd-analysis-archetype"] # Archetype detection
fun = ["tokmd-analysis-fun"]  # Fun/novelty report enrichers
```

## Key Types

```rust
pub struct AnalysisLimits {
    pub max_files: Option<usize>,
    pub max_bytes: Option<u64>,
    pub max_commits: Option<usize>,
    pub max_commit_files: Option<usize>,
    pub max_file_bytes: Option<u64>,
}
```

## License

MIT OR Apache-2.0
