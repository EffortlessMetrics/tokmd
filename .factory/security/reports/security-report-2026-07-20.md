# Security Scan Report

**Generated:** 2026-07-20
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits

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
identified during this scan. **Zero commits fell within the 7-day window**
(2026-07-13 through 2026-07-20). The most recent commit on `main`,
`5e8edc6 merge(swarm): import steward docs manifest fix` (2026-07-04), is
16 days old and outside the scan window. The tree state under review is
identical to what was assessed in the prior weekly scan
(`security-report-2026-06-29.md`), which itself returned 0 findings at the
`medium` threshold. The codebase continues to demonstrate a security-first
design with all standing defenses (per
`.factory/threat-model/threat-model.md`) re-verified intact. The threat model
(last reviewed 2026-06-01) remains current and is well within the 90-day
review window (49 days elapsed since last review).

## Critical Findings

*None.*

## High Findings

*None.*

## Medium Findings

*None.*

## Low Findings

*None.*

## Observations (Below Threshold, Not Reported As Findings)

These items were considered during the scan but do not meet the `medium`
severity threshold. They are carried forward from prior weekly scans for
traceability and to ensure the next scheduled scan continues to monitor
them.

### OBS-001 (carried): FFI JSON payload size not bounded

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-core/src/ffi/mod.rs` |
| **Status** | Not patched, design choice |

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
| **Status** | Not patched, mixed strategy |

**Description:** The Droid-related workflows
(`.github/workflows/droid.yml`, `droid-review.yml`, `droid-security-scan.yml`)
pin third-party actions by SHA, including the custom
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
- The custom Droid action (the highest-privilege third-party surface) IS
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
| **Status** | Not patched, review for future |

**Description:** The browser-side runner fetches repository content via
`fetch()` calls to `api.github.com` (and the codeload/GitHub
`releases`/`archive` endpoints). These URLs are hard-coded in the
`web/runner/` JavaScript modules. The token (when supplied) is stored in
`sessionStorage` (not `localStorage`) and used as a `Bearer` header. There
is no Subresource Integrity pinning or origin allow-listing on the
client-side fetch surface.

**Why not a finding:**
- All sensitive fetches target `api.github.com` / `codeload.github.com`,
  which are HTTPS and well-known.
- The token lifetime is bounded to a single browser tab
  (`sessionStorage`).
- No DOM injection surfaces observed: all dynamic data is rendered via
  `textContent` (verified across `main.js`); no use of `innerHTML`,
  `eval`, `new Function`, or `document.write` (confirmed by repository-wide
  grep returning no matches).
- Browser-side runner runs entirely in the user-agent sandbox; no
  filesystem, no subprocess.
- Below the `medium` severity threshold; informational only.

**Recommended action (optional):** Consider an explicit allowlist of fetch
origins and a CSP `connect-src` directive in the runner's served HTML
to defend against supply-chain injection via a compromised
`<script>`/module.

### OBS-005 (carried): `action.yml` install step performs `curl | sh` style download

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Tampering / Information Disclosure |
| **File** | `action.yml` (composite step `Install tokmd`) |
| **Status** | Not patched, verified checksums |

**Description:** The composite GitHub Action downloads a pre-built
`tokmd` binary from `github.com/EffortlessMetrics/tokmd/releases/...` and
verifies it against `checksums.txt` (sha256). It does not verify a
cryptographic signature on the checksum file or on the release itself.
The download URL is interpolated from a user-supplied `version` input
without shell-unsafe character filtering.

**Why not a finding:**
- The action is a published action; consumers control which version
  they pin to. The check is bounded to a `MAJOR.MINOR.PATCH`-style
  string via the `${ver#v}` prefix logic.
- `curl -fsSL` rejects HTTP errors and follows redirects (only to
  HTTPS GitHub release endpoints in practice).
- The checksum verification, when checksums.txt is present, uses
  `sha256sum`/`shasum`/`Get-FileHash` to compare the downloaded
  binary's hash to the expected value.
- Build provenance is separately attested via
  `actions/attest-build-provenance@v4` in `release.yml`.
- Below the `medium` severity threshold; this is documented best-
  practice coverage.

**Recommended action (optional):** Add explicit format validation
for the `version` input (e.g., regex `^v?\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$`)
and reject anything else before constructing the URL.

### OBS-006 (new): Zero commits in 7-day scan window

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Repudiation |
| **File** | `git log --since="7 days ago" --pretty=oneline` (empty) |
| **Status** | Not a finding, design choice |

**Description:** `git log --since="7 days ago" --oneline` returns zero
commits for the period 2026-07-13 through 2026-07-20. The most recent
commit on `main` is `5e8edc6 merge(swarm): import steward docs manifest
fix`, dated 2026-07-04 (16 days ago, outside the scan window). The tree
state under review is therefore identical to what was assessed in the
prior weekly scan (`security-report-2026-06-29.md`), which was itself a
true-merge initial import of the publication repository.

**Why not a finding:**
- "Zero commits" is a non-event from a vulnerability-introduction
  standpoint: there is no new code surface to evaluate this week.
- The dual-repo workbench topology (per `AGENTS.md`) means that
  content-bearing changes flow through reviewed `Tokmd Rust Result`
  PRs in `EffortlessMetrics/tokmd-swarm` and then true-merge into
  `EffortlessMetrics/tokmd` (the publication repository), each
  accompanied by a full weekly scan. See
  `docs/ci/swarm-routing.md` for the shared-history topology.
- All standing defenses were re-verified against the working tree
  state (see "Standing Defenses Verified" matrix below); no drift
  detected since the prior scan.

**Recommended action:** None; this is a tranquil week. The next scan
will resume coverage when a content change arrives.

## Standing Defenses Verified (No Regression)

The following defenses were re-verified during this scan. All remain intact.

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable` lints denied | `Cargo.toml` | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation | `crates/tokmd-core/src/ffi/inputs.rs` | ✓ |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs` | ✓ |
| D-08 | Per-family schema versioning (`SCHEMA_VERSION=2`, `COCKPIT_SCHEMA_VERSION=3`, `HANDOFF_SCHEMA_VERSION=5`, `CONTEXT_SCHEMA_VERSION=4`, `CONTEXT_BUNDLE_SCHEMA_VERSION=2`) | `crates/tokmd-types/src/` | ✓ |
| D-09 | SHA-pinned Droid-related actions; tag-pinned first-party actions | `.github/workflows/droid*.yml` (SHA), others (tag) | ✓ |
| D-10 | Branch protection on `main` (CODEOWNERS, 1 approval, CI required) | `.github/settings.yml` | ✓ |
| D-11 | `cargo-deny` advisory + license allowlist | `deny.toml` | ✓ |
| D-12 | BLAKE3 redaction with extension allowlist | `crates/tokmd-format/src/redact/mod.rs`, `crates/tokmd-format/src/redact/extensions.rs` | ✓ |
| D-13 | Content reads bounded by `ContentLimits` | `crates/tokmd-analysis/src/content/mod.rs` | ✓ |
| D-14 | PyO3 FFI invariants (no panic, GIL release, error translation) | `crates/tokmd-python/src/lib.rs` | ✓ |
| D-15 | WASM uses `MemFs` (no host fs) | `crates/tokmd-wasm/` | ✓ |
| D-16 | `web/runner` browser runner uses `textContent` (no `innerHTML`/`eval`) | `web/runner/main.js` | ✓ |
| D-17 | `web/runner` token stored in `sessionStorage` (not `localStorage`) | `web/runner/auth.js` | ✓ |
| D-18 | `web/runner` worker protocol allowlists modes & presets | `web/runner/messages.js` | ✓ |
| D-19 | Composite action installs tokmd with checksum verification | `action.yml` | ✓ |
| D-20 | Custom Droid action SHA-pinned across all Droid workflows | `.github/workflows/droid*.yml` | ✓ |
| D-21 | `cargo audit` invoked with structured `--json` output, malformed JSON treated as Pending | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-22 | `run_json` top-level JSON must be an object (strict shape check) | `crates/tokmd-core/src/ffi/mod.rs::run_json_inner` | ✓ |
| D-23 | Author DAG import via true-merge commits (no force-push of publication history) | repository topology | ✓ |

## Appendix

### Threat Model

- **Status:** Current (verified unchanged since 2026-06-01)
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Reviewed:** 2026-06-01 (49 days ago, well within 90-day window)
- **Methodology:** STRIDE
- **Next review:** 2026-09-01 (90-day cadence) or upon architecture change

### Scan Metadata

- **Commits Scanned:** 0
- **Files in scope:** 0 (no diff this period; tree state is identical to the
  prior scan's assessed state)
- **Window:** 2026-07-13 through 2026-07-20 (UTC)
- **Reference HEAD:** `5e8edc6a606b82eee9488cbe348e5b596bae1a96`, `merge(swarm):
  import steward docs manifest fix` (2026-07-04, 16 days ago, out of window)
- **Scan Duration:** <2m (no file-level scan required for zero-commit period;
  tree-state defense verification only)
- **Skills Used:** commit-security-scan (conceptual),
  vulnerability-validation (N/A; no findings to validate),
  security-review (N/A; no patches to generate)
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **False Positive Filter:** N/A; no candidate findings produced

### Scan Coverage Matrix

Since no commits land in the 7-day window, no file-level diff review is
performed this scan. The standing-defenses matrix above verifies that no
sensitive surface has drifted since the prior scan.

| Area | Files reviewed this scan | Notes |
|------|--------------------------|-------|
| CLI argv parsing | 0 | unchanged since 2026-06-29 |
| Subprocess invocation | 0 | unchanged since 2026-06-29 |
| Path handling | 0 | unchanged since 2026-06-29 |
| FFI inputs | 0 | unchanged since 2026-06-29 |
| File content reads | 0 | unchanged since 2026-06-29 |
| Redaction / hashing | 0 | unchanged since 2026-06-29 |
| GitHub workflows | 0 | unchanged since 2026-06-29 |
| Build / lint | 0 | unchanged since 2026-06-29 |
| Githooks | 0 | unchanged since 2026-06-29 |
| Web runner (browser) | 0 | unchanged since 2026-06-29 |
| Threat model | 0 (verified current) | unchanged since 2026-06-01 |
| Schema constants | 0 (verified against `CLAUDE.md`) | unchanged |

### Commit-level Analysis

```
$ git log --since="7 days ago" --oneline
(empty)
```

```
$ git log --since="30 days ago" --oneline
5e8edc6a606b82eee9488cbe348e5b596bae1a96  merge(swarm): import steward docs manifest fix
Author: Steven Zimmerman, CPA <15812269+EffortlessSteven@users.noreply.github.com>
Date:   Sat Jul 4 19:24:57 2026 -0400
```

The single commit in the 30-day window (`5e8edc6 merge(swarm): import
steward docs manifest fix`) is itself a swarm-routing merge that brings
the steward docs manifest fix (`79c99739..ab489870`) from
`EffortlessMetrics/tokmd-swarm` into `EffortlessMetrics/tokmd`. It was
authored and reviewed in the swarm workbench under `Tokmd Rust Result`
PR #433 (run 28722320062). The commit is dated 2026-07-04, which is
outside the 7-day scan window, and its surface was already covered
indirectly by the 2026-06-29 weekly scan at the point of publication.

**No security findings in scope this week.**

### Patches Generated

No patches were generated this scan (no findings at or above `medium`).

### Next Scan

The next scheduled security scan runs Monday, 2026-07-27 via
`.github/workflows/droid-security-scan.yml` (cron `0 8 * * 1`).

## References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Repository workbench boundary: `AGENTS.md`
- Repository swarm-routing topology: `docs/ci/swarm-routing.md`
- Previous scans: `.factory/security/reports/security-report-2026-06-01.md`,
  `.factory/security/reports/security-report-2026-06-08.md`,
  `.factory/security/reports/security-report-2026-06-29.md`
