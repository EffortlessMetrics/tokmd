# tokmd-scan

## Purpose

Source code scanning adapter. This is the **Tier 1** tokei wrapper for tokmd, isolating the tokei dependency to a single location.

## Responsibility

- Map tokmd args to tokei configuration
- Execute tokei scans
- Return raw `Languages` results
- **NOT** for aggregation, formatting, or business logic

## Public API

```rust
pub fn scan(paths: &[PathBuf], args: &GlobalArgs) -> Result<Languages>
```

Maps `GlobalArgs` fields to tokei `Config`:
- `hidden` → include hidden files
- `no_ignore` → skip all ignore files
- `no_ignore_dot` → skip .ignore files
- `no_ignore_parent` → skip parent ignore files
- `no_ignore_vcs` → skip .gitignore
- `treat_doc_strings_as_comments` → count doc strings as comments
- `config` → custom tokei config path

## Configuration Handling

```rust
pub enum ConfigMode {
    Auto,  // Search for .tokeignore, tokei.toml, etc.
    None,  // Skip config file loading
}
```

With `ConfigMode::Auto`, tokei searches for config files in the scanned directory.

## Implementation Details

### Best-Effort Error Handling
Tokei logs errors to stderr but doesn't return them. The scan function:
- Returns empty `Languages` for nonexistent paths
- Logs warnings but continues scanning
- Only fails on actual configuration errors

### Flag Implication
`no_ignore` implies all `no_ignore_*` variants are true.

## Dependencies

- `tokei` (14.0.0, no default features)
- `anyhow` (error handling)
- `tokmd-config` (GlobalArgs)

## Testing

```bash
cargo test -p tokmd-scan
```

Tests cover:
- Scan success with various flag combinations
- Hidden files handling
- Ignore pattern behavior
- Doc string handling
- Nonexistent paths (returns empty)

## Do NOT

- Aggregate or transform tokei results (use tokmd-model)
- Format output (use tokmd-format)
- Add CLI parsing logic
- Import higher-tier crates
