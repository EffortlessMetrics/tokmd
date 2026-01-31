# tokmd-tokeignore

## Purpose

Template generation for `.tokeignore` files. This is a **Tier 1** utility crate for initializing ignore patterns.

## Responsibility

- Generate `.tokeignore` templates by profile
- Write templates to disk or stdout
- Handle force overwrite logic
- **NOT** for parsing or applying ignore patterns

## Public API

```rust
pub fn init_tokeignore(args: &InitArgs) -> Result<()>
```

## Implementation Details

### Available Profiles

| Profile | Patterns |
|---------|----------|
| `Default` | All common build artifacts |
| `Rust` | Cargo target, backups, coverage |
| `Node` | node_modules, dist, build, coverage |
| `Mono` | Monorepo-safe conservative defaults |
| `Python` | `__pycache__`, .venv, .tox, .pytest_cache |
| `Go` | vendor, bin |
| `Cpp` | build, cmake-build-*, out, .cache |

### CLI Options

- `--profile <PROFILE>` - Select template profile
- `--print` - Print template to stdout instead of writing
- `--force` - Overwrite existing `.tokeignore`
- `--path <PATH>` - Target directory (default: current)

### Behavior

1. Validates target directory exists
2. Checks for existing `.tokeignore` (fails without `--force`)
3. Generates template based on profile
4. Writes to file or stdout

## Dependencies

- `anyhow` (error handling)
- `tokmd-config` (InitArgs, Profile enum)

## Testing

```bash
cargo test -p tokmd-tokeignore
```

Uses `tempfile` for file creation tests.

## Do NOT

- Parse or apply ignore patterns (tokei handles this)
- Add scanning logic
- Modify existing `.tokeignore` files (only create/overwrite)
