# tokmd-scan

Source code scanning adapter for tokmd.

## Overview

This is a **Tier 1** crate that wraps the [tokei](https://github.com/XAMPPRocky/tokei) library, isolating this dependency to a single location. It handles configuration mapping and scan execution.

## Installation

```toml
[dependencies]
tokmd-scan = "1.3"
```

## Usage

```rust
use tokmd_scan::scan;
use tokmd_config::GlobalArgs;
use std::path::PathBuf;

let args = GlobalArgs::default();
let paths = vec![PathBuf::from(".")];

let languages = scan(&paths, &args)?;
// Returns tokei::Languages with code statistics
```

## Configuration Mapping

Maps `GlobalArgs` fields to tokei configuration:

| Arg | Effect |
|-----|--------|
| `hidden` | Include hidden files/directories |
| `no_ignore` | Skip all ignore files |
| `no_ignore_dot` | Skip .ignore files |
| `no_ignore_parent` | Skip parent ignore files |
| `no_ignore_vcs` | Skip .gitignore |
| `treat_doc_strings_as_comments` | Count doc strings as comments |
| `config` | Config file loading strategy |

## Error Handling

- Returns error for non-existent paths (as of v1.3.0)
- Propagates tokei configuration errors
- Empty `Languages` for valid but empty directories

## Dependencies

- `tokei` - Core line counting
- `tokmd-config` - GlobalArgs definition
- `tokmd-types` - ConfigMode enum

## License

MIT OR Apache-2.0
