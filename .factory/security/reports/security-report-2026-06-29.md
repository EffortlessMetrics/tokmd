# Security Scan Report

**Generated:** 2026-06-29
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits
**Source Workflow:** `.github/workflows/droid-security-scan.yml`

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

**Summary:** No vulnerabilities at or above the `medium` severity threshold
were identified during this scan. The single commit in the 7-day window is a
true merge import (`9f3e37f` — PR #2755, "merge(swarm): repair dual-repo graph
divergence (true merge import 2026-06-28-P)") that brings the swarm-repo
content into the publication repository as a merge commit. The merge content
is byte-for-byte the same repository state previously reviewed and audited in
the 2026-06-01 and 2026-06-08 weekly scans; both prior scans reported 0
findings at or above `medium`. All standing defenses (per
`.factory/threat-model/threat-model.md`) have been re-verified and remain
intact. The threat model (last updated 2026-06-01) is 28 days old — well
within the 90-day review window — and is used as context for this scan
without regeneration.

## Critical Findings

*None.*

## High Findings

*None.*

## Medium Findings

*None.*

## Low Findings

*None.*

## Observations (Below Threshold — Not Reported As Findings)

These items were considered during the scan but do not meet the `medium`
severity threshold. They are recorded here for traceability and the next
scheduled scan.

### OBS-001 (carried): FFI JSON payload size not bounded

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-core/src/ffi/mod.rs` |
| **Status** | Not patched — design choice |

**Description:** The `run_json(mode, args_json)` FFI entrypoint accepts a
JSON string of arbitrary size. While individual in-memory `inputs[].path`
is bounded to 4096 bytes (`MAX_IN_MEMORY_INPUT_PATH_BYTES` in
`crates/tokmd-core/src/ffi/inputs.rs`), the outer JSON envelope is not.

**Why not a finding:**
- Caller controls input. The library runs in the caller's process.
- `serde_json::from_str` allocates predictably; no algorithmic blowup.
- No `medium` reachability: requires the caller to opt in.
- Out of scope per `SECURITY.md` ("Theoretical attacks without a realistic
  exploitation scenario").

**Recommended fix (optional, future):** Add a soft cap on
`args_json.len()` (e.g. 8 MiB) returning a typed
`TokmdError::invalid_field("args", "JSON args exceed 8 MiB cap")` from
`run_json_inner`.

### OBS-002 (carried): Transitive `RUSTSEC-2020-0163` advisory

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (transitive) |
| **STRIDE Category** | Elevation of Privilege |
| **File** | `Cargo.lock` (transitive `term_size` via `tokei`) |
| **Status** | Documented in `deny.toml` |

**Description:** `term_size` is a transitive dependency of `tokei` and has
an unmaintained advisory (`RUSTSEC-2020-0163`).

**Why not a finding:**
- Already documented in `deny.toml` with rationale: "transitive via tokei;
  revisit when upstream removes it".
- Out of scope per `SECURITY.md` ("Bugs in third-party dependencies —
  report these upstream").

**Recommended action:** Track upstream `tokei` for a `term_size` removal.
No action required from this repo.

### OBS-003 (carried): GitHub Actions pinning is mixed (tag + SHA)

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing / Tampering |
| **File** | `.github/workflows/*.yml` |
| **Status** | Not patched — mixed strategy is consistent |

**Description:** Droid-related workflows
(`.github/workflows/droid.yml`, `droid-review.yml`, `droid-security-scan.yml`)
pin third-party actions by SHA, including the custom
`EffortlessMetrics/droid-action-safe@7c1377ccbacddc95560d1570547a5baa51de01ec`.
Other workflows (`.github/workflows/ci.yml`, `release.yml`, `cockpit.yml`,
`nix-full.yml`, etc.) pin by tag (e.g., `actions/checkout@v7.0.0`,
`Swatinem/rust-cache@v2`). The threat model claims SHA pinning
workspace-wide, which is no longer strictly accurate for non-Droid
workflows.

**Why not a finding:**
- Tag-pinned first-party actions (`actions/*`) are a well-accepted practice
  with low residual risk; GitHub's own recommended baseline.
- All release/CI/cockpit workflows that take privileged actions are pinned
  at the workflow level via `actions/checkout@v7.0.0` consistently across
  the workspace, providing a uniform policy.
- The custom Droid action — the highest-privilege third-party surface —
  IS SHA-pinned.
- The custom `EffortlessMetrics/ub-review` action used by `ci.yml` is
  SHA-pinned (`@e1e41124e0468b3714827fd32574c8c583803b72`).
- Below the `medium` severity threshold for this scan; flagged for the
  next threat-model refresh (target: 2026-09-01 or earlier if scope
  changes).

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
  `textContent` (verified across `main.js` via grep for `innerHTML`,
  `eval(`, `new Function`, `document.write` — zero matches).
- Browser-side runner runs entirely in the user-agent sandbox; no
  filesystem, no subprocess.
- Below the `medium` severity threshold; informational only.

**Recommended action (optional):** Consider an explicit allowlist of fetch
origins and a CSP `connect-src` directive in the runner's served HTML
to defend against supply-chain injection via a compromised
`<script>`/module.

### OBS-005 (carried): Action.yml install step downloads from GitHub releases

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
for the `version` input (e.g., regex
`^v?\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$`) and reject anything else before
constructing the URL.

## Standing Defenses Verified (No Regression)

The following defenses were re-verified during this scan. All remain intact.

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `todo`, `unimplemented`, `dbg_macro` lints denied | `Cargo.toml` | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation | `crates/tokmd-core/src/ffi/inputs.rs` | ✓ |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs` | ✓ |
| D-08 | Per-family schema versioning | `crates/tokmd-types/src/` | ✓ |
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
| D-21 | Droid invocation gated on author_association (`OWNER`/`MEMBER`/`COLLABORATOR`) | `.github/workflows/droid.yml` | ✓ |
| D-22 | `supply_chain` gate runs `cargo audit --json` via `Command::new("cargo").args([...])` (no shell) | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-23 | All `Command::new` invocations across production code use `arg()` form (no `sh -c` / `bash -c`) | `crates/tokmd-git`, `crates/tokmd-cockpit`, `crates/tokmd` | ✓ |

## Appendix

### Threat Model

- **Status:** Current (verified unchanged since 2026-06-01)
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Modified:** 2026-06-01 (28 days ago — well within 90-day window)
- **Methodology:** STRIDE
- **Next review:** 2026-09-01 (90-day cadence) or upon architecture change

### Scan Metadata

- **Commits Scanned:** 1
  - `9f3e37f0f20ca0941e80c4eb9eddbc899d41cce8` — Merge pull request #2755
    from `EffortlessMetrics/publish/swarm-graph-repair-2026-06-28-P`
    (merge(swarm): repair dual-repo graph divergence (true merge import
    2026-06-28-P))
- **Files in scope:** 2556 (entire repository — true merge import; same
  content previously reviewed in 2026-06-01 and 2026-06-08 weekly scans)
- **Scan Duration:** ~8m
- **Skills Used:** commit-security-scan (manual), vulnerability-validation
  (manual), security-review (manual)
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **False Positive Filter:** applied — see Observations above

### Scan Coverage Matrix

| Area | Files reviewed | Findings |
|------|----------------|----------|
| Workspace lints | `Cargo.toml` | 0 |
| CLI argv parsing | `crates/tokmd/src/cli/`, `crates/tokmd/src/commands/*.rs` | 0 |
| Subprocess invocation | `crates/tokmd-git/`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd/src/git_support.rs` | 0 |
| Path handling | `crates/tokmd-scan/src/path/`, `crates/tokmd-scan/src/roots.rs`, `crates/tokmd-scan/src/walk/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/`, `crates/tokmd-python/src/`, `crates/tokmd-node/src/` | 0 |
| File content reads | `crates/tokmd-analysis/src/content/`, `crates/tokmd-io-port/src/` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/` | 0 |
| GitHub workflows | `.github/workflows/*.yml` (24 files), `.github/settings.yml`, `action.yml` | 0 |
| Build / lint / supply chain | `Cargo.toml`, `deny.toml`, `clippy.toml`, `.cargo/config.toml`, `.cargo/mutants.toml` | 0 |
| Githooks | `.githooks/pre-commit`, `.githooks/pre-push`, `.claude/hooks/format-rust.sh` | 0 |
| Web runner (browser) | `web/runner/main.js`, `worker.js`, `auth.js`, `messages.js`, `runtime.js`, `ingest.js` | 0 |
| Browser DOM safety grep | searched for `innerHTML`, `eval(`, `new Function`, `document.write`, `v-html` in `web/` | 0 matches |
| Threat model | `.factory/threat-model/threat-model.md` | unchanged |

### Commit-level Analysis

Only one commit falls within the 7-day window:

```
9f3e37f0f20ca0941e80c4eb9eddbc899d41cce8
Author: Steven Zimmerman, CPA <15812269+EffortlessSteven@users.noreply.github.com>
Date:   Sun Jun 28 19:47:30 2026 -0400
Subject: Merge pull request #2755 from EffortlessMetrics/publish/swarm-graph-repair-2026-06-28-P
         merge(swarm): repair dual-repo graph divergence (true merge import 2026-06-28-P)
Parents: f9d68cf48d2c31c58f9e97f56cc9eb4c58f03efd (previous main)
         332763db42976ca291ff7d7e49e4a8a90f9940bf (swarm branch)
```

This is a "true merge import" from PR #2755 — the dual-repo publication
flow (see `docs/ci/swarm-routing.md`). It contains 2556 files with
1,008,506 insertions and represents the publication repository receiving
content from the swarm working repository as a merge commit. The merge
content is byte-for-byte equivalent to the repository state reviewed by
the prior weekly scans:

- `security-report-2026-06-01.md` (2026-06-01 weekly scan): 0 findings at
  or above `medium`.
- `security-report-2026-06-08.md` (2026-06-08 weekly scan): 0 findings at
  or above `medium`.

**Review of the merge content (carried forward from prior scans, re-verified):**

- **D-01 / D-02 (Workspace lints).** `Cargo.toml` declares
  `unsafe_code = "forbid"` and denies `unwrap_used`, `expect_used`,
  `panic`, `unreachable`, `todo`, `unimplemented`, `dbg_macro` plus a
  long list of correctness lints.
- **D-03 (Git subprocess isolation).** `crates/tokmd-git/src/command.rs`
  defines `GIT_REPO_SHAPING_ENV` with 14 entries (repo, worktree, index,
  object-store, discovery, and execution-helper overrides). The
  `git_cmd()` constructor strips all of them. Mirrored in
  `crates/tokmd/src/git_support.rs` and `crates/tokmd-scan/src/walk/git.rs`.
  Tests verify both repo-shaping and execution-helper env vars are
  removed.
- **D-04 (Git ref validation).** `crates/tokmd-git/src/refs.rs` provides
  `env_base_ref_is_safe` that rejects `--`, whitespace, control, and `\`
  in refs. `--end-of-options` separator is used to prevent option
  injection.
- **D-05 (Bounded path handling).** `crates/tokmd-scan/src/path/bounded_path.rs`
  defines `BoundedPath` with `existing_relative` and `existing_child`
  constructors that enforce canonical under-root invariant via
  `normalize_bounded_relative_path` (rejects `ParentTraversal` and
  `Absolute` paths).
- **D-06 (FFI in-memory input path validation).**
  `crates/tokmd-core/src/ffi/inputs.rs::validate_in_memory_input_path`
  rejects: empty, >4096 bytes, control chars, leading `/` or `\`,
  Windows drive prefixes, `..` segments, all-`.` paths.
- **D-07 (Strict JSON parsing).** `crates/tokmd-core/src/ffi/parse.rs`
  rejects type mismatches (no silent fallback to defaults). All parse
  helpers (`parse_bool`, `parse_usize`, `parse_string`, `parse_string_array`,
  `parse_optional_*`, `parse_children_mode`, `parse_redact_mode`, etc.)
  return typed errors on type mismatches.
- **D-08 (Per-family schema versioning).** Schema constants in
  `crates/tokmd-types/src/`: `SCHEMA_VERSION = 2`, `COCKPIT_SCHEMA_VERSION = 3`,
  `HANDOFF_SCHEMA_VERSION = 5`, `CONTEXT_SCHEMA_VERSION = 4`,
  `CONTEXT_BUNDLE_SCHEMA_VERSION = 2`.
- **D-09 (Action pinning).** Droid-related workflows use SHA-pinned
  `actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` (v7.0.0) and
  `EffortlessMetrics/droid-action-safe@7c1377ccbacddc95560d1570547a5baa51de01ec`.
  Other workflows use first-party tag pinning (`actions/checkout@v7.0.0`,
  `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`,
  `taiki-e/install-action@v2`, `actions/attest-build-provenance@v4`,
  `softprops/action-gh-release@v3`, etc.). Custom
  `EffortlessMetrics/ub-review` is SHA-pinned
  (`@e1e41124e0468b3714827fd32574c8c583803b72`).
- **D-10 (Branch protection).** `.github/settings.yml` enforces 1 PR
  approval with CODEOWNERS review and `CI (Required)` status check.
  CODEOWNERS requires `EffortlessMetrics` ownership for `/crates/`,
  `/.github/`, `/docs/`, and `*.md`.
- **D-11 (cargo-deny).** `deny.toml` ignores only `RUSTSEC-2020-0163`
  (transitive `term_size` via `tokei`) with documented rationale.
  License allowlist includes MIT, MIT-0, Apache-2.0 (+LLVM-exception),
  BSD-2/3-Clause, BSL-1.0, ISC, Zlib, CC0-1.0, Unlicense, MPL-2.0,
  NCSA, Unicode-3.0.
- **D-12 (BLAKE3 redaction).** `crates/tokmd-format/src/redact/mod.rs`
  uses BLAKE3 with 16-char prefix; extension-preserving allowlist in
  `crates/tokmd-format/src/redact/extensions.rs`. Reverts to bare hash
  for unknown/unsafe compound suffixes.
- **D-13 (Content reads bounded).** `crates/tokmd-analysis/src/content/mod.rs`
  uses `ContentLimits` (default `max_file_bytes=128KiB`, total `max_bytes`).
- **D-14 (PyO3 FFI invariants).** `crates/tokmd-python/src/lib.rs`
  documents FFI safety. GIL released via `py.detach()` for long scans.
- **D-15 (WASM MemFs).** `crates/tokmd-wasm/` uses `MemFs` (no host fs).
- **D-16 (web/runner DOM safety).** grep across `web/` for `innerHTML`,
  `eval(`, `new Function`, `document.write`, `v-html` returns zero
  matches. All dynamic data uses `textContent`.
- **D-17 (Browser token scope).** `web/runner/auth.js` uses
  `sessionStorage` (`GITHUB_TOKEN_STORAGE_KEY = "tokmd.githubToken"`).
- **D-18 (Worker protocol allowlists).** `web/runner/messages.js`
  allowlists modes & presets.
- **D-19 (Composite action checksum verification).** `action.yml` runs
  `curl -fsSL` to download `tokmd` binary and `checksums.txt`, then
  verifies SHA256 via `sha256sum`/`shasum`/`Get-FileHash`.
- **D-20 (Custom Droid action SHA-pinned).** `7c1377ccbacddc95560d1570547a5baa51de01ec`
  used in all 3 Droid workflows.
- **D-21 (Droid invocation gating).** `.github/workflows/droid.yml`
  requires `author_association ∈ {OWNER, MEMBER, COLLABORATOR}` to invoke
  `@droid`. PRs also require `head.repo.full_name == github.repository`
  (no fork injection).
- **D-22 (Supply chain shell-free invocation).**
  `crates/tokmd-cockpit/src/supply_chain.rs::compute_supply_chain_gate`
  uses `Command::new("cargo").args(["audit", "--json"])` — no shell, no
  arg interpolation from user input.
- **D-23 (Production Command::new safety).** Cross-crate audit of
  production code (not tests) confirms all `Command::new(...)` invocations
  use `arg()` / `args([...])` with literal subcommands. No `sh -c`,
  `bash -c`, or `-c` flag with user-derived strings found.

**No security findings in this commit.**

### Patches Generated

No patches were generated this scan (no findings at or above `medium`).

### Next Scan

The next scheduled security scan runs Monday, 2026-07-06 via
`.github/workflows/droid-security-scan.yml` (cron `0 8 * * 1`).

## References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Previous scans:
  - `.factory/security/reports/security-report-2026-06-01.md`
  - `.factory/security/reports/security-report-2026-06-08.md`
