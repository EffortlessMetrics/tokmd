# Security Scan Report

**Generated:** 2026-09-28 08:15 UTC
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Branch Scanned:** `main`
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-09-21 → 2026-09-28)

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
identified during this scan. The 7-day window (2026-09-21 → 2026-09-28) contains
**zero commits** on `main` in this branch (`git log --since="7 days ago"
--pretty=oneline` returns no output). The most recent commit on `main` is
`ff05889 import: finalize stable release after consumer proof` from
**2026-08-05** — a history-preserving two-parent publication-import of the
stable-release ordering fix, **47 days before** this scan window opened. That
import added 2,589 files (~1,022,014 insertions) and was reviewed in depth
during the 2026-07-27 baseline re-verification, which itself was the standing
defense re-check following the 2026-06-29 comprehensive 2,579-file initial-
import review.

Because no new code or configuration entered the branch during the scan window,
the codebase was reviewed against:

1. The existing `.factory/threat-model/threat-model.md` (last modified
   2026-08-02, still within the 90-day freshness window — 57 days old).
2. The current standing defense set (D-01 through D-23) recorded in the
   2026-07-27 weekly scan.
3. Spot re-reads of the highest-risk security-critical modules
   (`tokmd-git/src/command.rs`, `tokmd-git/src/refs.rs`,
   `tokmd-core/src/ffi/mod.rs`, `tokmd-core/src/ffi/inputs.rs`,
   `tokmd-scan/src/path/bounded_path.rs`, `tokmd-format/src/redact/mod.rs`),
   which all match the patterns recorded in the threat model.

No regressions were detected. The codebase continues to demonstrate a
security-first design with the workspace-level lint floor
(`unsafe_code = "forbid"`, `unwrap_used = "deny"`, `expect_used = "deny"`,
`panic = "deny"`, `unreachable = "deny"`, `dbg_macro = "deny"`,
`todo = "deny"`, `unimplemented = "deny"`) and the standing defense set
unchanged since the last scan.

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
All "carried" observations are unchanged from the 2026-07-27 baseline; no new
informational observations were introduced this week. Where the present scan
differs from the previous weekly (2026-07-27), it is noted explicitly.

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
| **Status** | Not patched — mixed strategy (re-verified this scan) |

**Description:** The Droid-related and high-privilege third-party workflows
pin by SHA (`EffortlessMetrics/droid-action-safe@7c1377ccbacddc95560d1570547a5baa51de01ec`,
`EffortlessMetrics/ub-review@e1e41124e0468b3714827fd32574c8c583803b72`,
`actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`,
`actions/github-script@f28e40c7f34bde8b3046d885e986cb6290c5673b # v7`,
`actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.0`,
`cachix/install-nix-action@630ae543ea3a38a9a4166f03376c02c50f408342 # v31`,
`docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a # v7`,
`docker/login-action@dbcb813823bdd20940b903addbd779551569679f # v4`,
`docker/setup-buildx-action@bb05f3f5519dd87d3ba754cc423b652a5edd6d2c # v4`,
`docker/setup-qemu-action@96fe6ef7f33517b61c61be40b68a1882f3264fb8 # v4`).
Other workflows pin first-party and well-known third-party actions by tag
(`actions/attest-build-provenance@v4`, `actions/cache@v6`,
`actions/download-artifact@v8`, `dtolnay/rust-toolchain@stable`,
`Swatinem/rust-cache@v2`, `codecov/codecov-action@v7`,
`peter-evans/create-pull-request@v8`, `softprops/action-gh-release@v3`,
`taiki-e/install-action@v2`, `thollander/actions-comment-pull-request@v3`,
`DeterminateSystems/determinate-nix-action@v3`,
`DeterminateSystems/flakehub-cache-action@v3`,
`DeterminateSystems/magic-nix-cache-action@v14`,
`DeterminateSystems/nix-installer-action@v22`, `EndBug/label-sync@v2`,
`docker/metadata-action@v6`). The threat model claims SHA pinning
workspace-wide, which is no longer strictly accurate for non-Droid workflows.

**Re-verification (this scan):** Spot check across all 24 workflow files
returned 11 unique SHA-pinned `uses:` lines and 19 unique tag-pinned `uses:`
lines. Numbers are unchanged from the 2026-07-27 baseline; no new
tag-pinned first-party action was introduced since.

**Why not a finding:**
- Tag-pinned first-party actions (`actions/*`) are a well-accepted practice
  with low residual risk; GitHub's own recommended baseline.
- All release/CI/cockpit workflows that take privileged actions are pinned
  at the workflow level via `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`
  consistently across the workspace, providing a uniform policy.
- The custom Droid action — the highest-privilege third-party surface — IS
  SHA-pinned.
- Below the `medium` severity threshold for this scan; flagged for the next
  threat-model refresh (target: 2026-11-01 or earlier if scope changes).

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

| ID  | Defense | Location | Verified |
|-----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` (`[workspace.lints.rust]`) | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `todo`, `unimplemented`, `get_unwrap`, `unwrap_in_result`, `panic_in_result_fn` lints denied | `Cargo.toml` (`[workspace.lints.clippy]`) | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`: 14 names including `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_COMMON_DIR`, `GIT_CEILING_DIRECTORIES`, `GIT_SSH`, `GIT_SSH_COMMAND`, `GIT_ASKPASS`, `GIT_PAGER`, `GIT_EDITOR`, `GIT_PROXY_COMMAND`, `GIT_EXTERNAL_DIFF`) | `crates/tokmd-git/src/command.rs` | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` rejects `--`-prefix, whitespace, control, and `\\` in refs; `--end-of-options` separator used) | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root (`BoundedPath::existing_relative` / `existing_child`, `normalize_bounded_relative_path` rejects empty / `..` / absolute / drive-prefix paths, `ensure_under_root` rejects escape) | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation (`MAX_IN_MEMORY_INPUT_PATH_BYTES = 4096`, rejects empty / control / leading `/` or `\\` / Windows drive prefix / `..` segments / all-`.` paths) | `crates/tokmd-core/src/ffi/inputs.rs` | ✓ |
| D-07 | Strict JSON parsing with type validation (`run_json_inner` requires top-level object; no silent defaults; explicit type checks in `parse_scan_settings`) | `crates/tokmd-core/src/ffi/mod.rs`, `parse.rs` | ✓ |
| D-08 | Per-family schema versioning (`SCHEMA_VERSION=2`, `COCKPIT_SCHEMA_VERSION=3`, `HANDOFF_SCHEMA_VERSION=5`, `CONTEXT_SCHEMA_VERSION=4`, `CONTEXT_BUNDLE_SCHEMA_VERSION=2`) | `crates/tokmd-types/src/` | ✓ |
| D-09 | Mixed SHA/tag pinning (Droid-related + high-privilege third-party actions pinned by SHA; first-party and well-known actions pinned by tag with workspace-uniform `actions/checkout@v7.0.1` SHA) | `.github/workflows/*.yml` (24 files) | ✓ |
| D-10 | Branch protection on `main` (CODEOWNERS, 1 approval, CI required) | `.github/settings.yml` | ✓ |
| D-11 | `cargo-deny` advisory + license allowlist (RUSTSEC-2020-0163 documented, MIT/Apache/BSD/ISC/CC0/MPL/BSL/Unicode/NCSA/Zlib/Unlicense allowlist) | `deny.toml` | ✓ |
| D-12 | BLAKE3 redaction with extension allowlist (`short_hash` → 16-char prefix, `redact_path` → extension-preserving via `safe_path_extension_suffix`) | `crates/tokmd-format/src/redact/mod.rs`, `extensions.rs` | ✓ |
| D-13 | Content reads bounded by `ContentLimits` | `crates/tokmd-analysis/src/content/mod.rs` | ✓ |
| D-14 | PyO3 FFI invariants (no panic, GIL release, error translation) | `crates/tokmd-python/src/lib.rs` | ✓ |
| D-15 | WASM uses `MemFs` (no host fs) | `crates/tokmd-wasm/` | ✓ |
| D-16 | `web/runner` browser runner uses `textContent` (no `innerHTML`/`eval`/`new Function`/`document.write`) | `web/runner/main.js` | ✓ |
| D-17 | `web/runner` token stored in `sessionStorage` (not `localStorage`) | `web/runner/auth.js` | ✓ |
| D-18 | `web/runner` worker protocol allowlists modes & presets | `web/runner/messages.js` | ✓ |
| D-19 | Composite action installs tokmd with sha256 checksum verification | `action.yml` | ✓ |
| D-20 | Custom Droid action SHA-pinned across all Droid workflows | `.github/workflows/droid*.yml` | ✓ |
| D-21 | `cargo audit` invoked with structured `--json` output, malformed JSON treated as Pending | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-22 | `run_json` top-level JSON must be an object (strict shape check) | `crates/tokmd-core/src/ffi/mod.rs::run_json_inner` | ✓ |
| D-23 | Author DAG import via two-parent merge commits (no force-push of publication history) | repository topology | ✓ |


## Appendix

### Threat Model

- **Status:** Current (verified unchanged since 2026-08-02)
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Modified:** 2026-08-02 (57 days ago — within the 90-day window)
- **Methodology:** STRIDE
- **Next review:** 2026-11-01 (90-day cadence) or upon architecture change
- **No regeneration this scan** — within freshness window and no new
  external surface, subprocess invocation, or trust-boundary shift was
  introduced since 2026-08-02. The single post-threat-model commit
  (`ff05889` on 2026-08-05) is the stable-release publication merge; its
  content is the same files that were comprehensively reviewed during the
  2026-06-29 baseline scan (true-merge initial import of 2,579 files) and
  re-verified during the 2026-07-27 standing-defense spot-check.

### Scan Metadata

- **Commits Scanned:** 0 (the 7-day window `git log --since="7 days ago"`
  returns no commits; the most recent commit on `main` is
  `ff05889 import: finalize stable release after consumer proof` from
  2026-08-05, **47 days** before this scan window opened)
- **Files in scope:** Not applicable — no commit diff. The comprehensive
  baseline of 2,589 files was last fully reviewed in the 2026-06-29 scan
  (true-merge initial import) and re-verified in the 2026-07-27 scan
  (standing-defense spot-check). Spot re-reads during this scan covered the
  highest-risk security-critical modules listed in the standing-defense
  table above; all match the patterns recorded in the threat model.
- **Scan Duration:** ~3m (focused baseline re-verification, no diff
  resolution required)
- **Skills Used:** commit-security-scan (manual), vulnerability-validation
  (manual), security-review (manual)
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **False Positive Filter:** applied — see Observations above

### Scan Coverage Matrix

Because no commit delta was present in the window, the coverage matrix
below re-states the comprehensive coverage applied to the underlying
baseline (last fully scanned 2026-06-29, spot-checked 2026-07-27 and
this scan).

| Area | Files reviewed | Findings |
|------|----------------|----------|
| CLI argv parsing | `crates/tokmd/src/cli/`, `crates/tokmd/src/commands/*.rs` | 0 |
| Subprocess invocation | `crates/tokmd-git/src/command.rs`, `crates/tokmd-git/src/refs.rs`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd-cockpit/src/gates/contracts.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` | 0 |
| Path handling | `crates/tokmd-scan/src/path/bounded_path.rs`, `crates/tokmd-scan/src/roots.rs`, `crates/tokmd-scan/src/walk/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/mod.rs`, `inputs.rs`, `parse.rs`, `byte_mode.rs`, `crates/tokmd-python/src/`, `crates/tokmd-node/src/` | 0 |
| File content reads | `crates/tokmd-analysis/src/content/mod.rs`, `crates/tokmd-io-port/src/` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/mod.rs`, `extensions.rs` | 0 |
| GitHub workflows | `.github/workflows/*.yml` (24 files), `.github/settings.yml`, `action.yml` | 0 |
| Build / lint | `Cargo.toml` (workspace + crate lints), `deny.toml`, `clippy.toml`, `.cargo/config.toml` | 0 |
| Githooks | `.githooks/pre-commit`, `.githooks/pre-push`, `.claude/hooks/format-rust.sh` | 0 |
| Web runner (browser) | `web/runner/main.js`, `worker.js`, `auth.js`, `messages.js`, `runtime.js`, `ingest.js` | 0 |
| Threat model | `.factory/threat-model/threat-model.md` | unchanged |

### Commit-level Analysis

The 7-day window (2026-09-21 → 2026-09-28) contains no commits on `main`
in this repository. The most recent commit on `main` is
`ff05889 import: finalize stable release after consumer proof`, dated
**2026-08-05 01:44:26 -0400** (47 days before this scan window opened):

```
ff0588903142cedda2dbc4903bbb7128fa1cbb3e
Author: Steven Zimmerman, CPA <15812269+EffortlessSteven@users.noreply.github.com>
Date:   Wed Aug 5 01:44:26 2026 -0400
Subject: import: finalize stable release after consumer proof

    History-preserving import of the reviewed 1.15.1 stable-release
    ordering fix. Preserve the two-parent publication topology.
```

The commit's annotated message documents it as a history-preserving
two-parent publication import of the stable-release ordering fix. It
introduced 2,589 files and 1,022,014 insertions; all files were
previously reviewed in the 2026-06-29 baseline scan (true-merge initial
import of 2,579 files) and re-verified in the 2026-07-27 standing-defense
spot-check. No additional diff review is required for this scan.

**No security findings in this scan window.**

### Patches Generated

No patches were generated this scan (no findings at or above `medium`).

### Next Scan

The next scheduled security scan runs Monday, 2026-10-05 via
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
