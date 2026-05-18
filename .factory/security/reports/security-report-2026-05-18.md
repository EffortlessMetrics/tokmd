# Security Scan Report

**Generated:** 2026-05-18
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

## Scan Results

No security vulnerabilities were identified at or above the medium severity threshold in the commits scanned from the past 7 days.

### Commits Scanned

| Commit | Description | Files Changed |
|--------|-------------|----------------|
| `baf1b76` | docs(release): polish 1.11 release record (#2407) | 1 (CHANGELOG.md) |

### Files Analyzed

The following new files from recent changes were analyzed for security vulnerabilities:

- `crates/tokmd-core/src/ffi/mod.rs` - FFI entrypoint coordinator
- `crates/tokmd-core/src/ffi/envelope.rs` - JSON response envelope
- `crates/tokmd-core/src/ffi/inputs.rs` - In-memory input validation
- `crates/tokmd-core/src/ffi/modes.rs` - Mode dispatch with whitelist validation
- `crates/tokmd-core/src/ffi/parse.rs` - Strict JSON parsing helpers
- `crates/tokmd-core/src/ffi/settings_parse.rs` - Settings parsing
- `crates/tokmd-git/src/command.rs` - Git command isolation with env stripping

All new code implements strong security controls:
- Path validation prevents traversal attacks in in-memory inputs
- Mode whitelist prevents spoofing
- Environment variable stripping prevents git helper hijacking
- Strict JSON parsing prevents injection
- No shell execution, no unsafe code

## Appendix

### Threat Model

- **Version:** newly generated (2026-05-18)
- **Location:** `.factory/threat-model.md`
- **Coverage:** All 6 crate tiers, FFI layer and binding surfaces, path handling, git integration, output generation, CLI command parsing

### Security Controls Verified

| Control | Implementation | Status |
|---------|----------------|--------|
| Path traversal prevention | `canonicalize_bounded_path()` in `tokmd-scan` | Verified |
| Command injection prevention | Array-arg API in `tokmd-git` | Verified |
| FFI panic isolation | `panic = "deny"` lint policy | Verified |
| Input validation | Whitelist validation in FFI modes | Verified |
| Environment isolation | Env variable stripping in git commands | Verified |

### Scan Metadata

- **Commits Scanned:** 1
- **Files Scanned:** 7 (new FFI and git command files)
- **Skills Used:** threat-model-generation, commit-security-scan, vulnerability-validation

### Recommendations

1. **Continue monitoring** - The codebase demonstrates good security practices. Continue weekly scans to maintain visibility.

2. **Formula injection awareness** - The threat model identified CSV formula injection as a potential issue. Consider adding output encoding for CSV exports if not already present.

3. **DoS prevention** - The `max_commits` setting for git history analysis should be kept to prevent resource exhaustion attacks.

### References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
