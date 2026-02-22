# tokmd-analysis

## Purpose

Analysis logic and enrichers for tokmd receipts. This is a **Tier 3** orchestration crate that computes derived metrics.

## Responsibility

- Orchestrate optional analysis modules
- Compute derived metrics
- Support analysis presets
- **NOT** for formatting (see tokmd-analysis-format)

## Public API

```rust
pub fn analyze(request: AnalysisRequest) -> Result<AnalysisReceipt>

pub struct AnalysisRequest {
    pub context: AnalysisContext,
    pub limits: AnalysisLimits,
    pub preset: AnalysisPreset,
}

pub struct AnalysisContext {
    pub paths: Vec<PathBuf>,
    pub export: ExportData,
    pub base_receipt: Option<LangReceipt>,
}

pub fn normalize_root(path: &Path) -> PathBuf
```

## Implementation Details

### Analysis Presets

| Preset | Includes |
|--------|----------|
| `Receipt` | Core derived metrics (density, distribution, COCOMO) |
| `Health` | + TODO density, complexity, Halstead metrics |
| `Risk` | + Git hotspots, coupling, freshness, complexity, Halstead metrics |
| `Supply` | + Assets, dependency lockfiles |
| `Architecture` | + Import graph |
| `Topics` | Semantic topic clouds |
| `Security` | License radar, entropy profiling |
| `Identity` | Archetype detection, corporate fingerprint |
| `Git` | Predictive churn, advanced git metrics |
| `Deep` | Everything (except fun) |
| `Fun` | Eco-label, novelty outputs |

### Analysis Modules

| Module | Purpose |
|--------|---------|
| `archetype` | Project kind detection (CLI, library, web app, etc.) |
| `derived` | Core metrics (density, distribution, COCOMO) |
| `topics` | Semantic keyword extraction |
| `entropy` | High-entropy file detection |
| `license` | License radar scanning |
| `fingerprint` | Corporate domain analysis from git |
| `churn` | Git-based change trend prediction |
| `assets` | Asset categorization |
| `git` | Hotspots, bus factor, freshness, coupling |
| `content` | TODOs, duplicates, imports |

### Feature Flags

```toml
[features]
git = ["tokmd-git"]      # Git history analysis
walk = ["tokmd-walk"]    # Asset discovery
content = ["tokmd-content"]  # Content scanning
topics = ["tokmd-analysis-topics"] # Topic-cloud extraction
archetype = ["tokmd-analysis-archetype"] # Archetype detection
fun = ["tokmd-analysis-fun"]  # Fun report + novelty outputs
```

## Dependencies

- `blake3`, `serde_json`
- `tokmd-analysis-types`, `tokmd-types`
- Optional: `tokmd-git`, `tokmd-walk`, `tokmd-content`

## Testing

```bash
cargo test -p tokmd-analysis
cargo test -p tokmd-analysis --all-features
```

## Do NOT

- Format output (use tokmd-analysis-format)
- Add CLI parsing logic
- Modify files
