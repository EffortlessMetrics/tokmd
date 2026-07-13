# Security Scan Report

**Generated:** 2026-07-13
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Branch:** `droid/security-report-2026-07-13`
**Severity Threshold:** medium
**Scope:** Commits from 2026-07-06T00:00:00Z through 2026-07-13

## Executive Summary

| Severity | Count | Auto-fixed | Manual Required |
|----------|------:|-----------:|----------------:|
| CRITICAL | 0 | 0 | 0 |
| HIGH | 0 | 0 | 0 |
| MEDIUM | 0 | 0 | 0 |
| LOW | 0 | 0 | 0 |

**Total Findings:** 0
**Auto-fixed:** 0
**Manual Review Required:** 0

No vulnerabilities at or above the `medium` severity threshold were identified. The seven-day change set contained no commits and no files, so no commit-level findings were possible. An independent review of the current high-risk security surfaces found no reachable medium-or-higher vulnerabilities. No patches were needed, and no auto-fix commits were created.

## Scope

Local `git log --since="7 days ago"` and `git rev-list --count --since="7 days ago" HEAD` each found zero commits. Because the checkout is shallow, the result was independently confirmed with the GitHub API for commits since `2026-07-06T00:00:00Z`, which also returned zero commits. The latest remote commit is `5e8edc6a606b82eee9488cbe348e5b596bae1a96`, authored `2026-07-04T23:24:57Z`, and it matches local `HEAD`.

- **Commits in scope:** 0
- **Files in scope:** 0
- **Commit-level findings at or above medium:** 0

## Scan Coverage

The commit security scan correctly operated on an empty recent-change set. As an independent validation, the current high-risk surfaces were reviewed for reachable vulnerabilities in git subprocess handling, path and FFI validation, archive admission, redaction and content limits, proof execution, and CI actions. Candidate concerns were either below the configured threshold or not exploitable, so they are not reported as findings. This supplemental review does not expand or overstate the zero-file commit-level scope.

## Critical Findings

*None.*

## High Findings

*None.*

## Medium Findings

*None.*

## Low Findings

*None.*

## Appendix

### Threat Model

- **Status:** Current
- **Location:** `.factory/threat-model.md`
- **Generated:** 2026-07-13
- **Last Reviewed:** 2026-07-13
- **Methodology:** STRIDE

The configured threat model was missing at the start of the scan and was generated at the canonical path before security validation.

### Scan Metadata

- **Scan window start:** 2026-07-06T00:00:00Z
- **Scan date:** 2026-07-13
- **Commits scanned:** 0
- **Files in scope:** 0
- **Latest verified commit:** `5e8edc6a606b82eee9488cbe348e5b596bae1a96`
- **Latest commit authored:** 2026-07-04T23:24:57Z
- **Scan duration:** ~8m
- **Severity threshold:** medium
- **Skills Used:** threat-model-generation; commit-security-scan (empty recent-change set); vulnerability-validation (no candidate findings); security-review (independent high-risk-surface validation, no patches)
- **Patches generated:** None
- **Auto-fix commits created:** None
- **Previous scan:** `.factory/security/reports/security-report-2026-06-29.md`

### References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://learn.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model.md`
- Previous weekly report: `.factory/security/reports/security-report-2026-06-29.md`
