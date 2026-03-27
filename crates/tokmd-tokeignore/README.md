# tokmd-tokeignore

`.tokeignore` template generation for tokmd.

## Problem
Scans stay noisy when build outputs and vendored blobs are not excluded in a predictable way.

## What it gives you
- Template generation for `Default`, `Rust`, `Node`, `Mono`, `Python`, `Go`, and `Cpp` profiles
- `init_tokeignore(&InitArgs) -> Result<Option<PathBuf>>`
- Print, overwrite, and directory-validation behavior for creating a `.tokeignore`

## API / usage notes
- Use `--print` to preview a template and `--force` to overwrite an existing file.
- The templates are profile-based and tuned for common build artifacts and vendored code.
- `src/lib.rs` contains the exact template text and profile mapping.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [CLI Reference](../../docs/reference-cli.md)
- Reference: [src/lib.rs](src/lib.rs)
- Explanation: [Troubleshooting](../../docs/troubleshooting.md)
