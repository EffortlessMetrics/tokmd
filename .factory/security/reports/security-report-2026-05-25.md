# Security Scan Report

**Generated:** 2026-05-25
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium

## Executive Summary

| Severity | Count | Auto-fixed | Manual Required |
|----------|-------|------------|-----------------|
| CRITICAL | 0 | 0 | 0 |
| HIGH | 0 | 0 | 0 |
| MEDIUM | 0 | 0 | 0 |
| LOW | 0 | 0 | 0 |

**Total Findings:** 0
**Auto-fixed:** 0
**Manual Review Required:** 0

## Scan Details

### Commits Scanned (Last 7 Days)
- `8fca183` - Merge pull request #2505 from EffortlessMetrics/publish/swarm-publication-import-template-2026-05-24

### Files Changed
No Rust source code changes in the scanned period. The merge commit contains publication template changes and documentation updates.

### Analysis Scope
Security review focused on:
- FFI layer (`crates/tokmd-core/src/ffi/`) - JSON parsing and input validation
- Path processing (`crates/tokmd-scan/src/path/`) - Path canonicalization and validation
- Git command execution (`crates/tokmd-git/src/command.rs`) - Environment isolation
- CLI argument parsing (`crates/tokmd/src/`) - Clap-based parsing

### Security Controls Verified

| Component | Control | Status |
|-----------|---------|--------|
| Path canonicalization | `BoundedPath::canonicalize_existing()` | VERIFIED |
| Path traversal prevention | `normalize_bounded_relative_path()` | VERIFIED |
| Git environment isolation | `git_cmd()` strips 14 dangerous env vars | VERIFIED |
| FFI input validation | Strict JSON type checking | VERIFIED |
| In-memory path validation | `validate_in_memory_input_path()` | VERIFIED |
| Symlink escape prevention | `ensure_under_root()` | VERIFIED |

## Threat Model

- **Version:** 2026-05-25 (newly generated)
- **Location:** .factory/threat-model.md
- **Overall Risk Level:** LOW

## Findings Summary

No security vulnerabilities meeting the medium severity threshold were identified in this scan.

The codebase demonstrates strong security discipline:
- Defense in depth through multiple validation layers
- Explicit rejection of dangerous patterns (no silent fallbacks)
- Environment isolation for subprocess execution
- Path canonicalization to prevent escape attacks

### Key Security Features

1. **Path Validation**: All paths are validated and canonicalized before use, preventing directory traversal attacks.

2. **Git Environment Isolation**: Git commands strip dangerous environment variables that could be used for hook execution or repo manipulation.

3. **FFI Boundary Protection**: JSON inputs are strictly parsed with explicit error messages, preventing injection attacks.

4. **Redaction Support**: Built-in path hashing via BLAKE3 for sensitive output.

## Appendix

### Scan Metadata
- Commits Scanned: 1
- Scan Duration: ~5 minutes
- Skills Used: security-review (manual code analysis)

### References
- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [tokmd Security Policy](../../SECURITY.md)
