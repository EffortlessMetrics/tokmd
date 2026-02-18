# tokmd-sensor

## Purpose

Sensor contract and substrate builder. This is a **Tier 1** crate.

## Responsibility

- Define the `EffortlessSensor` trait for multi-sensor integration
- Build `RepoSubstrate` from a single tokei scan (+ optional git diff)
- **NOT** for sensor implementations (those go in their respective crates)
- **NOT** for CLI parsing

## Public API

```rust
/// Trait for effortless code quality sensors.
pub trait EffortlessSensor {
    type Settings: Serialize + DeserializeOwned;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn run(&self, settings: &Self::Settings, substrate: &RepoSubstrate) -> Result<SensorReport>;
}

/// Build a RepoSubstrate from a scan of the given repo root.
pub fn substrate_builder::build_substrate(
    repo_root: &str,
    scan_options: &ScanOptions,
    module_roots: &[String],
    module_depth: usize,
    diff_range: Option<DiffRange>,
) -> Result<RepoSubstrate>
```

## Implementation Details

### Settings / Substrate / Report Separation

1. **Settings** (`Self::Settings`): Each sensor defines its own configuration type
2. **Substrate** (`RepoSubstrate`): Shared context built once, shared across sensors
3. **Report** (`SensorReport`): Standardized envelope from `tokmd-envelope`

### Substrate Builder

- Runs tokei scan once via `tokmd-scan`
- Builds file rows via `tokmd-model`
- Marks files appearing in the diff range as `in_diff`
- Aggregates per-language summary into `BTreeMap` for determinism

### Feature Flags

- `git`: Enables `tokmd-git` dependency for git diff context

## Dependencies

- `tokmd-envelope` (SensorReport contract)
- `tokmd-substrate` (RepoSubstrate types)
- `tokmd-settings` (ScanOptions)
- `tokmd-scan` (tokei wrapper)
- `tokmd-model` (file row aggregation)
- `tokmd-types` (ChildIncludeMode)
- Optional: `tokmd-git` (behind `git` feature)

## Testing

```bash
cargo test -p tokmd-sensor
```

Tests cover:
- Trait implementation with a dummy sensor
- Substrate building from real crate source
- Diff range marking (in_diff selectivity)
- Error on missing repo root

## Do NOT

- Add sensor implementations here (this crate defines the contract only)
- Add CLI argument parsing (belongs in tokmd CLI)
- Skip path normalization in the substrate builder
