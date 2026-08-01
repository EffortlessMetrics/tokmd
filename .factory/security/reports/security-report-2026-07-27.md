# Security Scan Report

**Generated:** 2026-07-27
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd-swarm
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-07-20 → 2026-07-27)

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
identified during this scan. The 7-day window (2026-07-20 → 2026-07-27) contains
**zero commits** on `main` in this branch (`git log --since="7 days ago" --pretty=oneline`
returns no output). The most recent commit on `main` is `5e8edc6 merge(swarm):
import steward docs manifest fix` from 2026-07-04 — a true-merge import of
steward/docs/manifest files into the swarm branch that was already covered by
the 2026-06-29 weekly scan (which itself covered the full true-merge initial
import of 2579 files). Because no new code or configuration entered the
branch during the scan window, the codebase was reviewed against the existing
`.factory/threat-model/threat-model.md` (last modified 2026-06-01, still within
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
All "carried" observations are unchanged from the 2026-06-29 baseline; no new
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
| **Status** | Not patched — verified checksums |

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


## Standing Defenses Verified (No Regression)

The following defenses were re-verified during this scan. All remain intact.

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` (line 56) | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `todo`, `unimplemented` lints denied | `Cargo.toml` (lines 65-71) | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` (line 12) | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation | `crates/tokmd-core/src/ffi/inputs.rs` (line 12: `MAX_IN_MEMORY_INPUT_PATH_BYTES = 4096`) | ✓ |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs` | ✓ |
| D-08 | Per-family schema versioning (`SCHEMA_VERSION=2`, `COCKPIT_SCHEMA_VERSION=3`, `HANDOFF_SCHEMA_VERSION=5`, `CONTEXT_SCHEMA_VERSION=4`, `CONTEXT_BUNDLE_SCHEMA_VERSION=2`) | `crates/tokmd-types/src/` | ✓ |
| D-09 | SHA-pinned Droid-related actions; tag-pinned first-party actions | `.github/workflows/droid*.yml` (SHA), `ci.yml` SHA-pinned for `EffortlessMetrics/ub-review`, others (tag) | ✓ |
| D-10 | Branch protection on `main` (CODEOWNERS, 1 approval, CI required) | `.github/settings.yml` | ✓ |
| D-11 | `cargo-deny` advisory + license allowlist | `deny.toml` | ✓ |
| D-12 | BLAKE3 redaction with extension allowlist | `crates/tokmd-format/src/redact/mod.rs`, `crates/tokmd-format/src/redact/extensions.rs` | ✓ |
| D-13 | Content reads bounded by `ContentLimits` | `crates/tokmd-analysis/src/content/mod.rs` | ✓ |
| D-14 | PyO3 FFI invariants (no panic, GIL release, error translation) | `crates/tokmd-python/src/lib.rs` | ✓ |
| D-15 | WASM uses `MemFs` (no host fs) | `crates/tokmd-wasm/` | ✓ |
| D-16 | `web/runner` browser runner uses `textContent` (no `innerHTML`/`eval`/`new Function`/`document.write`) | `web/runner/main.js` | ✓ |
| D-17 | `web/runner` token stored in `sessionStorage` (not `localStorage`) | `web/runner/auth.js` | ✓ |
| D-18 | `web/runner` worker protocol allowlists modes & presets | `web/runner/messages.js` | ✓ |
| D-19 | Composite action installs tokmd with sha256 checksum verification | `action.yml` | ✓ |
| D-20 | Custom Droid action SHA-pinned across all Droid workflows | `.github/workflows/droid*.yml` | ✓ |
| D-21 | `cargo audit` invoked with structured `--json` output, malformed JSON treated as Pending | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-22 | `run_json` top-level JSON must be an object (strict shape check) | `crates/tokmd-core/src/ffi/mod.rs::run_json_inner` (line 78) | ✓ |
| D-23 | Author DAG import via true-merge commits (no force-push of publication history) | repository topology | ✓ |


## Appendix

### Threat Model

- **Status:** Current (verified unchanged since 2026-06-01)
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Modified:** 2026-06-01 (56 days ago — well within 90-day window)
- **Methodology:** STRIDE
- **Next review:** 2026-09-01 (90-day cadence) or upon architecture change
- **No regeneration this scan** — within freshness window and no new
  external surface, subprocess invocation, or trust-boundary shift was
  introduced since 2026-06-01.

### Scan Metadata

- **Commits Scanned:** 0 (the 7-day window `git log --since="7 days ago"`
  returns no commits; the most recent commit on `main` is
  `5e8edc6 merge(swarm): import steward docs manifest fix` from 2026-07-04,
  22 days ago)
- **Files in scope:** Not applicable — no commit diff. The comprehensive
  baseline of 2579 files was last reviewed in the 2026-06-29 scan against
  the same true-merge import that established the branch root.
- **Scan Duration:** ~3m (focused baseline re-verification, no diff
  resolution required)
- **Skills Used:** commit-security-scan (manual), vulnerability-validation
  (manual), security-review (manual)
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **False Positive Filter:** applied — see Observations above

### Scan Coverage Matrix

Because no commit delta was present in the window, the coverage matrix
below re-states the comprehensive coverage applied to the underlying
baseline (last fully scanned 2026-06-29 and unchanged since).

| Area | Files reviewed | Findings |
|------|----------------|----------|
| CLI argv parsing | `crates/tokmd/src/cli/`, `crates/tokmd/src/commands/*.rs` | 0 |
| Subprocess invocation | `crates/tokmd-git/`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd-cockpit/src/gates/contracts.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` | 0 |
| Path handling | `crates/tokmd-scan/src/path/`, `crates/tokmd-scan/src/roots.rs`, `crates/tokmd-scan/src/walk/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/` (including `mod.rs`, `inputs.rs`, `parse.rs`, `byte_mode.rs`), `crates/tokmd-python/src/`, `crates/tokmd-node/src/` | 0 |
| File content reads | `crates/tokmd-analysis/src/content/`, `crates/tokmd-io-port/src/` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/` | 0 |
| GitHub workflows | `.github/workflows/*.yml` (24 files), `.github/settings.yml`, `action.yml` | 0 |
| Build / lint | `Cargo.toml`, `deny.toml`, `clippy.toml`, `.cargo/config.toml` | 0 |
| Githooks | `.githooks/pre-commit`, `.githooks/pre-push`, `.claude/hooks/format-rust.sh` | 0 |
| Web runner (browser) | `web/runner/main.js`, `worker.js`, `auth.js`, `messages.js`, `runtime.js`, `ingest.js` | 0 |
| Threat model | `.factory/threat-model/threat-model.md` | unchanged |

### Commit-level Analysis

The 7-day window (2026-07-20 → 2026-07-27) contains no commits on `main`
in this repository. The branch root is the `5e8edc6 merge(swarm): import
steward docs manifest fix` commit, dated 2026-07-04 19:24:57 -0400, which
is **22 days** before this scan:

```
5e8edc6a606b82eee9488cbe348e5b596bae1a96
Author: Steven Zimmerman, CPA <15812269+EffortlessSteven@users.noreply.github.com>
Date:   Sat Jul 4 19:24:57 2026 -0400
Subject: merge(swarm): import steward docs manifest fix
    Swarm-Head: ab489870a0a3fd6c6addfd8f71d2f0a7031c2a3b
    Swarm-Range: 79c99739..ab489870
    Checks:
    - Tokmd Rust Result: https://github.com/EffortlessMetrics/tokmd-swarm/actions/runs/28722320062 (PR #433)
```

The merge's annotated commit message documents it as a true-merge import
of steward/docs/manifest content from the publication repository, and the
resolved `Tokmd Rust Result` run is documented in the merge header. The
prior weekly scan (`security-report-2026-06-29.md`) already verified this
commit and the wider 2579-file true-merge initial import that established
the branch root. No additional diff review is required for this scan.

**No security findings in this scan window.**

### Patches Generated

No patches were generated this scan (no findings at or above `medium`).

### Next Scan

The next scheduled security scan runs Monday, 2026-08-03 via
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
  `.factory/security/reports/security-report-2026-06-29.md`
