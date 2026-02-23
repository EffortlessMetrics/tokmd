# tokmd-module-key

Single-responsibility microcrate for deterministic module-key derivation from paths.

## What it does

- Normalizes path separators for module grouping (`\` -> `/`).
- Handles common relative path prefixes (`./`).
- Derives stable module keys from a path, module roots, and depth.

## API

- `module_key(path: &str, module_roots: &[String], module_depth: usize) -> String`
- `module_key_from_normalized(path: &str, module_roots: &[String], module_depth: usize) -> String`
