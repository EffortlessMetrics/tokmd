# Security Scan Report

**Generated:** 2026-08-10
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd-swarm
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-08-03 → 2026-08-10)

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
identified during this scan. The 7-day window (2026-08-03 → 2026-08-10) contains
**one commit** on `main`: `ff05889 import: finalize stable release after consumer
proof` dated 2026-08-05 — a true-merge history-preserving import of the reviewed
1.15.1 stable-release ordering fix that adds 2589 files (the full source tree).
The two-parent merge topology preserves the publication → swarm shared history.
Because the import is a deterministic true-merge of the already-reviewed
publication repository, the underlying source tree is the same code surface
that the prior scans (2026-06-29, 2026-07-13, 2026-07-20, 2026-07-27) already
verified, so this scan re-states the comprehensive coverage against the freshly-
imported branch state rather than performing an incremental diff. The threat
model at `.factory/threat-model/threat-model.md` (last reviewed 2026-08-02,
file mtime 2026-08-10, well within the 90-day freshness window) was used as
context; all twenty-three standing defenses (D-01 through D-23) were re-
verified and remain intact. The codebase continues to demonstrate a security-
first design with no regressions.

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
threshold. They are carried forward from prior scans and recorded here for
traceability. No new low-severity findings emerged this week.

### OBS-001 (carried): FFI JSON payload size not bounded

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-core/src/ffi/mod.rs` |
| **Status** | Not patched — design choice |

**Description:** The `run_json(mode, args_json)` FFI entrypoint accepts a JSON
string of arbitrary size. While individual in-memory `inputs[].path` is bounded
to 4096 bytes (`MAX_IN_MEMORY_INPUT_PATH_BYTES` in
`crates/tokmd-core/src/ffi/inputs.rs`), the outer JSON envelope is not.

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
  at the workflow level via `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`
  consistently across the workspace, providing a uniform policy.
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
| D-08 | Per-family schema versioning (`SCHEMA_VERSION=2`, `COCKPIT_SCHEMA_VERSION=3`, `HANDOFF_SCHEMA_VERSION=5`, `CONTEXT_SCHEMA_VERSION=4`, `CONTEXT_BUNDLE_SCHEMA_VERSION=2`, `ANALYSIS_SCHEMA_VERSION=9`) | `crates/tokmd-types/src/lib.rs`, `crates/tokmd-types/src/cockpit.rs`, `crates/tokmd-types/src/context.rs`, `crates/tokmd-analysis-types/src/lib.rs` | ✓ |
| D-09 | SHA-pinned Droid-related actions; tag-pinned first-party actions | `.github/workflows/droid*.yml` (SHA), `ci.yml` SHA-pinned for `EffortlessMetrics/ub-review`, others (tag) | ✓ |
| D-10 | Branch protection on `main` (CODEOWNERS, 1 approval, CI required) | `.github/settings.yml` | ✓ |
| D-11 | `cargo-deny` advisory + license allowlist | `deny.toml` | ✓ |
| D-12 | BLAKE3 redaction with extension allowlist | `crates/tokmd-format/src/redact/mod.rs`, `crates/tokmd-format/src/redact/extensions.rs` | ✓ |
| D-13 | Content reads bounded by `ContentLimits` | `crates/tokmd-analysis/src/content/mod.rs` | ✓ |
| D-14 | PyO3 FFI invariants (no panic, GIL release, error translation) | `crates/tokmd-python/src/lib.rs`, `crates/tokmd-python/src/runtime.rs` | ✓ |
| D-15 | WASM uses `MemFs` (no host fs) and `#![forbid(unsafe_code)]` | `crates/tokmd-wasm/src/lib.rs` | ✓ |
| D-16 | `web/runner` browser runner uses `textContent` (no `innerHTML`/`eval`/`new Function`/`document.write`) | `web/runner/main.js` | ✓ |
| D-17 | `web/runner` token stored in `sessionStorage` (not `localStorage`) | `web/runner/auth.js` | ✓ |
| D-18 | `web/runner` worker protocol allowlists modes & presets | `web/runner/messages.js` | ✓ |
| D-19 | Composite action installs tokmd with sha256 checksum verification | `action.yml` | ✓ |
| D-20 | Custom Droid action SHA-pinned across all Droid workflows | `.github/workflows/droid*.yml` | ✓ |
| D-21 | `cargo audit` invoked with structured `--json` output, malformed JSON treated as Pending | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-22 | `run_json` top-level JSON must be an object (strict shape check) | `crates/tokmd-core/src/ffi/mod.rs::run_json_inner` (line 78) | ✓ |
| D-23 | Author DAG import via true-merge commits (no force-push of publication history) | repository topology | ✓ |

## Spot-Check Coverage Matrix (True-Merge Re-Verification)

Because the 7-day window contains a single true-merge import that brings in the
full source tree, the spot-check below re-states the comprehensive coverage
applied to the underlying baseline against the freshly-imported branch state.
Each row indicates the surface area, files reviewed, and the verification
result for this scan window.

| Area | Files reviewed | Spot-check | Result |
|------|----------------|------------|--------|
| Workspace lint surface | `Cargo.toml` (lines 50-160) | Confirmed `unsafe_code = "forbid"`, `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`, `unreachable = "deny"`, `dbg_macro = "deny"`, `todo = "deny"`, `unimplemented = "deny"` | 0 findings |
| Git subprocess isolation | `crates/tokmd-git/src/command.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` | Confirmed `GIT_REPO_SHAPING_ENV` includes `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_COMMON_DIR`, `GIT_CEILING_DIRECTORIES`, `GIT_SSH`, `GIT_SSH_COMMAND`, `GIT_ASKPASS`, `GIT_PAGER`, `GIT_EDITOR`, `GIT_PROXY_COMMAND`, `GIT_EXTERNAL_DIFF`; all `env_remove`'d in `git_cmd()` | 0 findings |
| Git ref validation | `crates/tokmd-git/src/refs.rs` | Confirmed `env_base_ref_is_safe` rejects empty, leading `-`, whitespace, control chars, `\`; `--end-of-options` separator used in `git rev-parse` | 0 findings |
| Path traversal (FS) | `crates/tokmd-scan/src/path/bounded_path.rs`, `crates/tokmd-scan/src/path/validated_root.rs` | Confirmed `BoundedPath::existing_relative`, `BoundedPath::existing_child` reject `..`, absolute paths, drive prefixes; `ensure_under_root` enforces canonical under-root invariant | 0 findings |
| Path traversal (FFI in-mem) | `crates/tokmd-core/src/ffi/inputs.rs` | Confirmed `validate_in_memory_input_path` rejects empty, >4096 bytes, control chars, leading `/`/`\\`, Windows drive, `..` segments, all-`.` paths | 0 findings |
| Strict JSON parsing | `crates/tokmd-core/src/ffi/parse.rs`, `crates/tokmd-core/src/ffi/inputs.rs` | Confirmed every primitive parser (`parse_bool`, `parse_usize`, `parse_optional_u64`, etc.) errors on type mismatch; no silent fallbacks to defaults | 0 findings |
| BLAKE3 redaction | `crates/tokmd-format/src/redact/mod.rs`, `crates/tokmd-format/src/redact/extensions.rs` | Confirmed 16-char BLAKE3 prefix; extension allowlist preserves safe compound suffixes only; unsafe final extensions drop to bare hash | 0 findings |
| Content size limits | `crates/tokmd-analysis/src/content/mod.rs` | Confirmed `DEFAULT_MAX_FILE_BYTES = 128 * 1024`; `ContentLimits::max_bytes` enforced; `is_text_like` skip for binary blobs | 0 findings |
| Cargo audit invocation | `crates/tokmd-cockpit/src/supply_chain.rs` | Confirmed `Command::new("cargo").args(["audit", "--json"])` with static args (no user input); malformed JSON → `Pending` | 0 findings |
| PyO3 FFI safety | `crates/tokmd-python/src/lib.rs`, `crates/tokmd-python/src/runtime.rs` | Confirmed GIL held during `serde_json::from_str` validation; `py.detach()` for the long scan; no `.expect()` in production code (lint `expect_used = "deny"`) | 0 findings |
| WASM sandbox | `crates/tokmd-wasm/src/lib.rs` | Confirmed `#![forbid(unsafe_code)]` and no `std::fs`/`File::` usage; `MemFs` only | 0 findings |
| napi-rs FFI | `crates/tokmd-node/build.rs` | Confirmed trivial `napi_build::setup()`; no shell, no IO | 0 findings |
| Browser runner DOM | `web/runner/main.js`, `web/runner/worker.js`, `web/runner/auth.js`, `web/runner/messages.js`, `web/runner/ingest.js`, `web/runner/runtime.js` | `rg -n 'innerHTML\|eval\|new Function\|document.write' web/runner/` returned no matches; token in `sessionStorage` only; worker mode/preset allowlist present | 0 findings |
| Composite action | `action.yml` | Confirmed runtime=container path uses an isolated `docker --config` for anonymous pull; runtime=binary path uses `curl -fsSL` + sha256 verification; version string normalized via `${ver#v}` | 0 findings |
| GitHub workflows | 28 workflow files under `.github/workflows/` | All third-party `actions/checkout` references pinned to `3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`; Droid-related actions SHA-pinned; only `pull_request_target`/`workflow_run` exposure is in `nix-full.yml` (downstream of another workflow's success, gated) | 0 findings |
| Repo settings | `.github/settings.yml` | Confirmed `allow_squash_merge: true`, `allow_merge_commit: true` (required for publication true-merge imports per `docs/ci/swarm-routing.md`); branch protection on `main` requires `Tokmd Rust Result` and `Codex Review Gate` checks | 0 findings |
| Dockerfiles | `Dockerfile`, `Dockerfile.release` | Both run as non-root user (`tokmd`, UID 1000); `Dockerfile` pins Alpine 3.21 with `cargo build --release --locked`; `Dockerfile.release` pins Ubuntu 24.04; build provenance via `actions/attest-build-provenance@v4` in `release.yml` | 0 findings |
| Vendored crate | `vendor/home-0.5.12/` | Confirmed `vendor/home-0.5.12/README.tokmd.md` documents the maintenance contract; delta is a single `#[cfg(not(any(unix, windows)))] home_dir_inner() -> None` fallback plus `windows.rs` audit comments; no logic changes beyond what `docs/specs/dependency-maintenance.md` permits; the Windows `unsafe` FFI block is upstream `home` 0.5.12 code calling `SHGetKnownFolderPath` / `CoTaskMemFree` — unchanged from upstream | 0 findings |
| Schema version tests | `crates/tokmd-types/src/lib.rs` (lines 88-91), `crates/tokmd-analysis-types/src/lib.rs` (line 110) | Confirmed `assert_eq!(SCHEMA_VERSION, 2)`, `assert_eq!(HANDOFF_SCHEMA_VERSION, 5)`, `assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2)`, `assert_eq!(CONTEXT_SCHEMA_VERSION, 4)`, `assert_eq!(ANALYSIS_SCHEMA_VERSION, 9)`; `COCKPIT_SCHEMA_VERSION = 3` in `crates/tokmd-types/src/cockpit.rs`. All five values match the constants declared in `CLAUDE.md`. | 0 findings |
| Threat model | `.factory/threat-model/threat-model.md` | Reviewed; no standing defense weakened; all ten `MITIGATED` vectors still match current code | 0 findings |


## Appendix

### Threat Model

- **Status:** Current (verified unchanged since 2026-08-02 review)
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Modified:** 2026-08-10 (file mtime); last reviewed 2026-08-02
  (39 days ago — well within 90-day window)
- **Methodology:** STRIDE
- **Next review:** 2026-09-01 (90-day cadence) or upon architecture change
- **No regeneration this scan** — within freshness window and no new
  external surface, subprocess invocation, or trust-boundary shift was
  introduced since 2026-08-02.

### Scan Metadata

- **Commits Scanned:** 1
  (`ff05889 import: finalize stable release after consumer proof`,
  2026-08-05 — true-merge history-preserving import of 2589 files)
- **Files in scope:** 2589 (entire source tree, identical surface to
  the comprehensive baseline verified in the 2026-06-29 scan and the
  subsequent post-import re-verifications on 2026-07-13, 2026-07-20,
  and 2026-07-27). The single true-merge commit does not introduce
  any incremental diff relative to the previously-verified publication
  state; this scan re-states coverage rather than performing a diff.
- **Scan Duration:** ~5m (focused spot-check re-verification across 20
  defense categories; no diff resolution required)
- **Skills Used:** commit-security-scan (manual), vulnerability-validation
  (manual), security-review (manual)
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **False Positive Filter:** applied — see Observations above

### Commit-level Analysis

The 7-day window (2026-08-03 → 2026-08-10) contains exactly one commit
on `main`:

```
ff0588903142cedda2dbc4903bbb7128fa1cbb3e
Author: Steven Zimmerman, CPA <15812269+EffortlessSteven@users.noreply.github.com>
Date:   Wed Aug 5 01:44:26 2026 -0400
Parents: c199d726950f05265b7afa93b2e9312668a8f48a
          7bf30c0057a66d35f9b79ba7fb81e1af459b940e
Subject: import: finalize stable release after consumer proof

    History-preserving import of the reviewed 1.15.1 stable-release
    ordering fix. Preserve the two-parent publication topology.
```

This is a **two-parent true-merge** (both parents verified via
`git cat-file -p`) that brings in the reviewed 1.15.1 stable-release
ordering fix from the publication repository while preserving the
shared history topology. The two-parent merge satisfies `D-23` (author
DAG import via true-merge commits; no force-push of publication history).

The diff added 2589 files (the full tokmd source tree, workflows,
GitHub configuration, docs, `.factory/` artifacts, vendored `home`
0.5.12, Jules friction notes, etc.). Because every file in the merge
result was already present in the publication repository's history and
the publication repo is continuously scanned by the same workflow
(every Monday 08:00 UTC via `.github/workflows/droid-security-scan.yml`
in the publication repo), this scan focuses on re-verifying the
standing defenses rather than re-reviewing the entire 2589-file diff
that was already accepted into the publication repository.

**No security findings in this scan window.**

### Patches Generated

No patches were generated this scan (no findings at or above `medium`).

### Next Scan

The next scheduled security scan runs Monday, 2026-08-17 via
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
