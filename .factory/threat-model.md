# Threat Model for tokmd

**Generated:** 2026-05-25
**Repository:** EffortlessMetrics/tokmd
**Type:** STRIDE-based security threat model

## Overview

tokmd is a Rust CLI tool and library that wraps the tokei library to generate code inventory receipts.

## STRIDE Analysis

### Spoofing
- FFI Entry (`run_json`): Strict JSON parsing with type validation
- In-memory inputs: Path validation blocks absolute paths, parent traversal, control chars
- CLI arguments: Clap-based parsing with typed arguments

### Tampering
- File paths: `BoundedPath` rejects parent traversal, canonicalization prevents symlink escapes
- Configuration: `ConfigMode::None` skips all config loading
- Path operations: Validation rejects control characters

### Repudiation
- Git operations: `git_cmd()` strips all repo-shaping and execution-helper environment variables
- Git hooks: Environment stripping prevents hook execution

### Information Disclosure
- File paths: `RedactMode` provides path hashing
- Output: User controls what is exposed

### Denial of Service
- Deep directory traversal: `ignore` crate handles cycles
- Pathological paths: `MAX_IN_MEMORY_INPUT_PATH_BYTES = 4096` limit
- Git history: `max_commits` and `max_commit_files` limits

### Elevation of Privilege
- FFI boundary: Strict parsing, type validation, no code evaluation
- Path operations: Bounded paths prevent escape
- Plugin loading: No plugin system; static linking

## Security Controls

| Control | Implementation | Effectiveness |
|---------|----------------|---------------|
| Path canonicalization | `canonicalize_existing()` in `bounded_path.rs` | HIGH |
| Path validation | `normalize_bounded_relative_path()` | HIGH |
| Git env isolation | `git_cmd()` strips 14 dangerous environment variables | HIGH |
| JSON parsing | Strict type checking | HIGH |
| Path length limits | `MAX_IN_MEMORY_INPUT_PATH_BYTES = 4096` | MEDIUM |
| Redaction | BLAKE3 hashing | HIGH |

## Conclusion

**Overall Risk Level:** LOW

No critical or high severity vulnerabilities identified.
