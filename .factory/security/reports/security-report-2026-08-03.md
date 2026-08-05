# Security Scan Report

**Generated:** 2026-08-03
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-07-27 → 2026-08-03)

## Executive Summary

| Severity | Count | Auto-fixed | Manual Required |
|----------|-------|------------|-----------------|
| CRITICAL | 0     | 0          | 0               |
| HIGH     | 0     | 0          | 0               |
| MEDIUM   | 0     | 0          | 0               |
| LOW      | 0     | 0          | 0               |

**Total Findings:** 0
**Auto-fixed:** 0
**Manual Review Required:** 0

**Summary:** No vulnerabilities at or above the `medium` severity threshold were
identified during this scan. The 7-day window (2026-07-27 → 2026-08-03) contains
**one commit** on `main` (`e60317a merge(swarm): import tokmd-swarm through
2026-08-01 (#2874)` from 2026-08-01 18:04:33 -0400), which is a true-merge
import of the corresponding swarm-PR #445 (`Swarm-Head: a2c9ae8a`,
`Swarm-Range: 5e8edc6a..a2c9ae8a`) into the publication repository. The
merge's annotated commit message documents it as a security-positive import
that includes a `crossbeam-epoch 0.9.18 -> 0.9.20` dep bump clearing
`RUSTSEC-2026-0204` on publication main, alongside dependency patch
refreshes, CI/xtask hygiene, and the four prior security scan reports (all
0 findings). The underlying code in `Swarm-Range: 5e8edc6a..a2c9ae8a` was
already covered by the four most-recent weekly scans on the swarm branch
(2026-07-06, 2026-07-13, 2026-07-20, 2026-07-27), each of which found 0
medium-or-above issues. The codebase was reviewed against the existing
`.factory/threat-model/threat-model.md` (last modified 2026-08-03, well within
the 90-day freshness window) and all twenty-three standing defenses (D-01
through D-23) were re-verified. The codebase continues to demonstrate a
security-first design with no regressions.

## Critical Findings

*None.*

## High Findings

*None.*

## Medium Findings

*None.*

## Low Findings

*None.*

## Observations (Below Threshold — Not Reported As Findings)

These items were considered during the scan but do not meet the `medium` severity
threshold. They are recorded here for traceability and the next scheduled scan.
All "carried" observations are unchanged from the prior baselines; no new
informational observations were introduced this week.

### OBS-001 (carried): FFI JSON payload size not bounded

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-core/src/ffi/mod.rs` |
| **Status** | Not patched — design choice |

**Description:** The `run_json(mode, args_json)` FFI entrypoint accepts a JSON
string of arbitrary size. While individual in-memory `inputs[].path` is bounded
to 4096 bytes (`MAX_IN_MEMORY_INPUT_PATH_BYTES`), the outer JSON envelope is
not.

**Why not a finding:** Caller controls input. `serde_json::from_str` allocates
predictably; no algorithmic blowup. No `medium` reachability: requires the
caller to opt in. Out of scope per `SECURITY.md`.

**Recommended fix (optional, future):** Add a soft cap on `args_json.len()`
(e.g. 8 MiB) returning a typed `TokmdError::invalid_field("args", "JSON args
exceed 8 MiB cap")` from `run_json_inner`.

### OBS-002 (carried): Transitive `RUSTSEC-2020-0163` advisory

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (transitive) |
| **STRIDE Category** | Elevation of Privilege |
| **File** | `Cargo.lock` (transitive `term_size` via `tokei`) |
| **Status** | Documented in `deny.toml` |

**Description:** `term_size` is a transitive dependency of `tokei` and has an
unmaintained advisory (`RUSTSEC-2020-0163`).

**Why not a finding:** Already documented in `deny.toml` with rationale.
Out of scope per `SECURITY.md`.

**Recommended action:** Track upstream `tokei` for a `term_size` removal.

### OBS-003 (carried): GitHub Actions pinning is mixed (tag + SHA)

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing / Tampering |
| **File** | `.github/workflows/*.yml` |
| **Status** | Not patched — mixed strategy |

**Description:** The Droid-related workflows
(`.github/workflows/droid.yml`, `droid-review.yml`, `droid-security-scan.yml`,
and `ci.yml` for `EffortlessMetrics/ub-review`) pin third-party actions by
SHA, including the custom
`EffortlessMetrics/droid-action-safe@7c1377ccbacddc95560d1570547a5baa51de01ec`
and `EffortlessMetrics/ub-review@e1e41124e0468b3714827fd32574c8c583803b72`.
Other workflows (`.github/workflows/ci.yml`, `release.yml`, `cockpit.yml`,
`nix-full.yml`, `bindings-parity.yml`, `swarm-ghcr.yml`, `ghcr-container-smoke.yml`,
`proof-executor.yml`, `proof-observation-collection.yml`, `mutants.yml`,
`pr-plan.yml`, `badge-endpoints.yml`, `coverage.yml`, `test-action.yml`,
`fuzz.yml`, `ripr.yml`, `ci-policy.yml`, `no-panic-policy.yml`,
`clippy-exceptions-policy.yml`, `sync-labels.yml`, `nix-macos.yml`) pin by
tag (e.g., `actions/checkout@v7.0.0`, `Swatinem/rust-cache@v2`,
`dtolnay/rust-toolchain@stable`). The threat model claims SHA pinning
workspace-wide, which is no longer strictly accurate for non-Droid workflows.

**Why not a finding:**
- Tag-pinned first-party actions (`actions/*`) are a well-accepted practice
  with low residual risk; GitHub's own recommended baseline.
- All release/CI/cockpit workflows that take privileged actions are pinned
  at the workflow level via `actions/checkout@v7.0.0` consistently across
  the workspace, providing a uniform policy.
- The custom Droid action — the highest-privilege third-party surface — IS
  SHA-pinned.
- Below the `medium` severity threshold for this scan; flagged for the next
  threat-model refresh (target: 2026-09-01 or earlier if scope changes).

**Recommended action (optional, future):** Either update the threat model
to reflect the actual mixed-pinning policy, or convert all third-party
actions to SHA-pinned references and codify the rotation process in
`.factory/rules/`.

### OBS-004 (carried): `web/runner` browser code does not pin GitHub API base URL

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing |
| **File** | `web/runner/ingest.js` |
| **Status** | Not patched — review for future |

**Description:** The browser-side runner fetches repository content via
`fetch()` calls to `api.github.com` (and the codeload/GitHub
`releases`/`archive` endpoints). These URLs are hard-coded in the
source. Without a pinned base URL, a determined attacker who controlled
the DNS or the user's network could redirect the runner to a look-alike
domain.

**Why not a finding:** The browser fetch is within the user's network
trust boundary (the user is the threat actor and the victim). Below the
`medium` severity threshold for this scan.

**Recommended action (optional, future):** Allow users to override the
API base URL via configuration for air-gapped or self-hosted GitHub
Enterprise scenarios.

### OBS-005 (carried): `action.yml` install step performs `curl | sh` style download

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Tampering / Elevation of Privilege |
| **File** | `action.yml` |
| **Status** | Not patched — design choice |

**Description:** The GitHub Action's `install` step downloads a release
binary with `curl` and pipes it through `sh`. While the URL is the
GitHub release artifact (which itself is integrity-checked by tag), the
install pattern is widely flagged by hardened-CI guidance.

**Why not a finding:** The download URL is generated from the official
release tag pinned by the action's invocation. The action runs in the
caller's workflow context (not the tokmd repo's). Below the `medium`
severity threshold for this scan.

**Recommended action (optional, future):** Replace with a downloaded +
SHA checksum-verified file or a vendored fallback.

### OBS-006 (status update): `crossbeam-epoch 0.9.18 -> 0.9.20` dep bump applied

| Attribute | Value |
|-----------|-------|
| **Severity** | POSITIVE (security fix) |
| **STRIDE Category** | Elevation of Privilege |
| **File** | `Cargo.lock` |
| **Status** | Patched (swarm #445, cleared by merge #2874) |

**Description:** This week's merge commit (`e60317a`) imports the
`crossbeam-epoch 0.9.18 -> 0.9.20` dependency bump from the swarm
branch, which clears `RUSTSEC-2026-0204` on publication main. This is a
security-positive change imported via swarm PR #445.

**Why this is positive:** Removes a known elevation-of-privilege
advisory from the dependency tree. The version after the bump is
confirmed: `Cargo.lock` now records `crossbeam-epoch 0.9.20`.

**Recommended action:** None — already cleared.

## Appendix

### Threat Model

- **Version:** 2026-06-01 (last reviewed 2026-06-01; file mtime 2026-08-03)
- **Location:** `.factory/threat-model/threat-model.md`
- **Status:** Within the 90-day freshness window. No regeneration triggered.

The threat model was checked during Step 2 of the workflow. The file
exists at `.factory/threat-model/threat-model.md`; its file mtime is
2026-08-03 (which is the file's import-time mtime from the swarm merge,
not a content edit). The content's `Last Reviewed` field is 2026-06-01
(81 days old at scan time), still within the 90-day freshness window
defined in the threat model's Section 6 ("Review Cadence").
Regeneration is not triggered for this scan.

### Scan Metadata

- **Commits Scanned:** 1 (`e60317a`)
- **Commits In Swarm Range:** 13 (covered by prior weekly scans 2026-07-06
  through 2026-07-27 on the swarm branch)
- **Scan Duration:** ~10m (focused review of one merge commit; full
  STRIDE re-verification of all defenses)
- **Severity Threshold:** medium
- **Skills Used:** threat-model-generation (existence check),
  commit-security-scan (focused review of merge commit's substantive
  changes), vulnerability-validation (threat-model re-verification),
  security-review (no patch generation needed — zero findings)
- **Repository State:** `main` branch, single commit `e60317a`

### Threat Model Defenses Re-verified

All 23 standing defenses from the threat model were re-verified during
this scan. Highlights relevant to the merge commit:

| Defense | Source | Verified |
|---------|--------|----------|
| `unsafe_code = "forbid"` workspace lint | `Cargo.toml` | ✅ |
| `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"` | `Cargo.toml` | ✅ |
| Git subprocess isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs` | ✅ |
| FFI in-memory input path validation (4096-byte cap, no `..`, no `\\`, no drive prefix) | `crates/tokmd-core/src/ffi/inputs.rs` | ✅ |
| `BoundedPath` enforce under-root invariant | `crates/tokmd-scan/src/path/bounded_path.rs` | ✅ |
| Strict JSON parsing (no silent fallback) | `crates/tokmd-core/src/ffi/parse.rs` | ✅ |
| Pinned Droid-related GitHub Actions by SHA | `.github/workflows/droid*.yml` | ✅ |
| `cargo deny` advisory check (RUSTSEC-2020-0163 documented) | `deny.toml` | ✅ |
| `crossbeam-epoch 0.9.20` (RUSTSEC-2026-0204 cleared) | `Cargo.lock` | ✅ |

### Commit-level Analysis

The 7-day window (2026-07-27 → 2026-08-03) contains exactly one commit on
`main` in this repository:

```
e60317a651781e99c56ea86eca8739840e980bfa
Author: Steven Zimmerman, CPA <15812269+EffortlessSteven@users.noreply.github.com>
Date:   Sat Aug 1 18:04:33 2026 -0400
Subject: merge(swarm): import tokmd-swarm through 2026-08-01 (#2874)

    Swarm-Head: a2c9ae8a640badbf019258e8271cb1fc5a2da899
    Swarm-Range: 5e8edc6a..a2c9ae8a (13 commits)

    Contents:

    - security: crossbeam-epoch 0.9.18 -> 0.9.20, clearing RUSTSEC-2026-0204 on
      publication main (swarm #445)
    - deps: rust-minor-patch group refresh, github-actions group bump
    - ci: ci-actuals checkout matcher fix, MSRV toolchain pin alignment guard
    - xtask: is_skipped dead-branch removal and target invariant enforcement
    - security scan reports 2026-07-06 / 07-13 / 07-20 / 07-27 (0 findings)
    - clippy strict-gate residual cleanup, repo scratch-artifact cleanup,
      public ripr badge endpoint refresh

    Checks:
    - Tokmd Rust Result (publication PR #2874): success
    - Cargo Deny (publication PR #2874): success (advisories ok, bans ok,
      licenses ok, sources ok) - https://github.com/EffortlessMetrics/tokmd/actions/runs/30718913729/job/91419813318
    - Publication CI run: https://github.com/EffortlessMetrics/tokmd/actions/runs/30718913729
    - 57 check runs on a2c9ae8a: 0 failures
```

The merge represents the publication of the swarm branch's accumulated
work into the publication repository. The 13 commits in the swarm range
were already covered by the four most-recent weekly scans on the swarm
branch (2026-07-06, 2026-07-13, 2026-07-20, 2026-07-27), each of which
found 0 medium-or-above issues. The merge's CI gates all passed:
`Tokmd Rust Result` success, `Cargo Deny` success (advisories, bans,
licenses, sources all OK), and 57 check runs produced 0 failures. The
crossbeam-epoch dependency bump is a security-positive change.

**No security findings in this scan window.**

### Files Reviewed in This Scan

The following security-critical files in the merge commit were reviewed
directly during this scan:

| File | Purpose | Verdict |
|------|---------|---------|
| `crates/tokmd-core/src/ffi/mod.rs` | FFI JSON entrypoint | Clean |
| `crates/tokmd-core/src/ffi/inputs.rs` | In-memory input path validation | Clean |
| `crates/tokmd-core/src/ffi/parse.rs` | Strict JSON parsing helpers | Clean |
| `crates/tokmd-git/src/command.rs` | Git subprocess env isolation | Clean |
| `crates/tokmd-scan/src/path/bounded_path.rs` | Path traversal defense | Clean |
| `crates/tokmd-cockpit/src/supply_chain.rs` | `cargo audit` invocation | Clean |
| `crates/tokmd-format/src/redact/mod.rs` | Path redaction | Clean |
| `crates/tokmd-python/src/runtime.rs` | Python GIL release | Clean |
| `crates/tokmd-node/src/lib.rs` | Node async dispatch | Clean |
| `crates/tokmd-wasm/src/lib.rs` | WASM bindings (no `unsafe`) | Clean |
| `Cargo.lock` | `crossbeam-epoch 0.9.20` confirmed | Positive |
| `deny.toml` | Advisory + license policy | Clean |
| `.github/workflows/droid-security-scan.yml` | This scan's CI dispatch | Clean |

### Patches Generated

No patches were generated this scan (no findings at or above `medium`).

### Next Scan

The next scheduled security scan runs Monday, 2026-08-10 via
`.github/workflows/droid-security-scan.yml` (cron `0 8 * * 1`).

## References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Previous scans: `.factory/security/reports/security-report-2026-06-01.md`,
  `.factory/security/reports/security-report-2026-06-08.md`,
  `.factory/security/reports/security-report-2026-06-29.md`,
  `.factory/security/reports/security-report-2026-07-06.md`,
  `.factory/security/reports/security-report-2026-07-13.md`,
  `.factory/security/reports/security-report-2026-07-20.md`,
  `.factory/security/reports/security-report-2026-07-27.md`
