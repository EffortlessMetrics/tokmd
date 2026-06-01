# Security Scan Report

**Generated:** 2026-06-01
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

## Scan Scope

- **Commits Scanned:** 0 (no new commits since 2026-05-24)
- **Last Commit:** 8fca183 (2026-05-24) - Merge pull request #2505
- **Files Analyzed:** Architecture review of critical security surfaces

## Security Analysis Summary

### Threat Model Coverage
A new threat model was generated covering 5 trust boundaries:
1. CLI → Process boundary
2. FFI → tokmd-core boundary (hardened JSON interface)
3. tokmd-git shell-out boundary
4. tokmd-scan → tokei filesystem boundary
5. Cross-language FFI boundary (Python/Node/WASM)

### Critical Components Reviewed

| Component | Status | Notes |
|-----------|--------|-------|
| FFI run_json() | SECURE | JSON envelope validation, no panics |
| Git command execution | SECURE | Uses Command::args(), env stripping, ref validation |
| Path validation | SECURE | BoundedPath, canonicalization, symlink checks |
| Python bindings | SECURE | GIL release patterns, ? operator usage |
| WASM bindings | SECURE | Correct capability advertisement |

### Existing Mitigations Verified

1. **FFI path validation**: 4096-byte limit, control char rejection, no `..` traversal
2. **run_json envelope**: Always returns valid JSON, never panics
3. **Rust Command::args()**: Git arguments passed safely without shell expansion
4. **Python ? operator**: No .expect() in production code
5. **WASM capability advert**: Correctly advertises rootless capability surface

### Identified Gaps (Below Severity Threshold)

The following were identified but are LOW severity or mitigate by design:

1. **No per-input byte limit on decoded base64 content** - No exploitation path identified
2. **Git refs not validated for `-` prefix in all paths** - `env_base_ref_is_safe()` provides partial coverage
3. **Error messages may leak path fragments** - Informational only, no data exfiltration

## Conclusion

No security vulnerabilities at or above MEDIUM severity were identified in this scan.

## Appendix

### Threat Model
- **Version:** 2026-06-01
- **Location:** .factory/threat-model.md
- **Status:** Newly generated

### Scan Metadata
- **Commits Scanned:** 0
- **Files Reviewed:** 12 critical files across FFI, git, path validation, and bindings
- **Scan Duration:** ~15 minutes
- **Skills Used:** threat-model-generation, commit-security-scan, vulnerability-validation

### References
- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [tokmd Security Policy](../SECURITY.md)
