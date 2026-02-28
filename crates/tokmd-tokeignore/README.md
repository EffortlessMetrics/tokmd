# tokmd-tokeignore

Template generation for `.tokeignore` files.

## Overview

This is a **Tier 1** utility crate for generating `.tokeignore` templates. It provides language-specific and monorepo-aware patterns for excluding build artifacts and vendored code from tokmd scans.

## Installation

```toml
[dependencies]
tokmd-tokeignore = "1.3"
```

## Usage

```rust
use tokmd_tokeignore::init_tokeignore;
use tokmd_config::{InitArgs, InitProfile};
use std::path::PathBuf;

let args = InitArgs {
    dir: PathBuf::from("."),
    template: InitProfile::Rust,
    force: false,
    print: false,
    non_interactive: true,
};

let path = init_tokeignore(&args)?;
// path is Some(PathBuf) if file was written, None if --print mode
```

## Available Profiles

| Profile | Patterns |
|---------|----------|
| `Default` | All common build artifacts |
| `Rust` | `target/`, `*.rs.bk`, coverage |
| `Node` | `node_modules/`, `dist/`, `build/` |
| `Mono` | Conservative monorepo defaults |
| `Python` | `__pycache__/`, `.venv/`, `.tox/` |
| `Go` | `vendor/`, `bin/` |
| `Cpp` | `build/`, `cmake-build-*/` |

## CLI Options

- `--template <PROFILE>` - Select template profile
- `--print` - Print template to stdout
- `--force` - Overwrite existing file
- `--dir <PATH>` - Target directory

## Behavior

1. Validates target directory exists
2. Checks for existing `.tokeignore` (fails without `--force`)
3. Generates template based on profile
4. Writes to file or stdout

## License

MIT OR Apache-2.0
