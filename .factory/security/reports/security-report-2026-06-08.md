# Security Scan Report

**Generated:** 2026-06-08
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (1 commit: standing defenses re-verification)

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
identified during this scan. The single commit in scope (`ccd6796`) is the
publication-side merge of the evidence-packet contract from `tokmd-swarm` and
introduces no new surface area beyond standing configurations and agent
manifests; all substantive code paths remain identical to the 2026-06-01 scan
baseline. The codebase continues to demonstrate a security-first design with
multiple defense-in-depth measures already in place (see
`.factory/threat-model/threat-model.md` for the full STRIDE analysis). One
transitive `RUSTSEC-2020-0163` advisory (transitive `term_size` via `tokei`)
remains documented in `deny.toml` and is out of scope per `SECURITY.md`.

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
threshold. They are carried forward from the 2026-06-01 baseline and re-verified
this scan.

### OBS-001: FFI JSON payload size not bounded

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-core/src/ffi/mod.rs` |
| **Status** | Not patched — design choice |

**Description:**
The `run_json(mode, args_json)` FFI entrypoint accepts a JSON string of
arbitrary size. While individual in-memory `inputs[].path` is bounded to
4096 bytes (`MAX_IN_MEMORY_INPUT_PATH_BYTES`), the outer JSON envelope is
not. A caller (Python / Node binding) could pass a multi-megabyte JSON
string.

**Why not a finding:**
- Caller controls input. The library runs in the caller's process.
- `serde_json::from_str` allocates predictably; no algorithmic blowup.
- No `medium` reachability: requires the caller to opt in.
- Out of scope per `SECURITY.md` ("Issues in third-party dependencies"
  and "Theoretical attacks without a realistic exploitation scenario").

**Re-verification (2026-06-08):** The FFI `inputs.rs` validation surface
(`validate_in_memory_input_path`) is unchanged in this scan window. The
4 KiB `MAX_IN_MEMORY_INPUT_PATH_BYTES` cap, control-character rejection,
`/`, `\\`, Windows-drive, and `..`-segment rejection all remain in effect
as confirmed by direct source review this scan.

**Recommended fix (optional, future):**
Add a soft cap on `args_json.len()` (e.g. 8 MiB) returning a typed
`TokmdError::invalid_field("args", "JSON args exceed 8 MiB cap")` from
`run_json_inner`. Document the limit in the Python and Node API docs.

### OBS-002: Transitive `RUSTSEC-2020-0163` advisory

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (transitive) |
| **STRIDE Category** | Elevation of Privilege |
| **File** | `Cargo.lock` (transitive `term_size` via `tokei`) |
| **Status** | Documented in `deny.toml` |

**Description:**
`term_size` is a transitive dependency of `tokei` and has an unmaintained
advisory (`RUSTSEC-2020-0163`).

**Why not a finding:**
- Already documented in `deny.toml` with rationale: "transitive via tokei;
  revisit when upstream removes it".
- Out of scope per `SECURITY.md` ("Bugs in third-party dependencies — report
  these upstream").

**Re-verification (2026-06-08):** `deny.toml` still contains the
`RUSTSEC-2020-0163` ignore with the same rationale. The `cargo deny
--all-features check` lane in `ci.yml` continues to enforce the
advisory + license policy. No new transitive advisories were observed in
the `Cargo.lock` diff for this scan.

**Recommended action:**
Track upstream `tokei` for a `term_size` removal. No action required from
this repo.

### OBS-003: GitHub Actions SHA-pinning is not fully uniform

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing / Tampering |
| **File** | `.github/workflows/*.yml` |
| **Status** | Not patched — mixed SHA / tag pinning |

**Description:**
The repo's threat model lists "All third-party actions pinned by SHA" as a
standing defense (D-09). In practice the pinning is *mixed*:

- **SHA-pinned** (preferred, with version comment): `droid-review.yml`,
  `droid.yml`, `droid-security-scan.yml`, `action.yml` (uses
  `actions/upload-artifact@043fb46d...`, `thollander/actions-comment-pull-request@24bffb9b...`).
- **Tag-pinned** (mutable ref, e.g. `@v6.0.2`): `ci.yml`, `release.yml`,
  `cockpit.yml`, `mutants.yml`, `fuzz.yml`, `proof-executor.yml`,
  `coverage.yml`, `ci-policy.yml`, `pr-plan.yml`, `test-action.yml`,
  `nix-full.yml`, `nix-macos.yml`, `badge-endpoints.yml`,
  `no-panic-policy.yml`, `ripr.yml`, `sync-labels.yml`,
  `proof-observation-collection.yml`, `em-routed-rust-small.yml`.
- **Dependabot enabled** for GitHub Actions (`.github/dependabot.yml`) so
  tag-pinned actions are kept current, but a tag is still a mutable ref.

**Why not a finding:**
- Tag pinning with Dependabot refresh is a widely accepted best practice.
- No supply-chain compromise has been observed.
- Migrating every tag pin to SHA would be a large, low-value diff with no
  exploit path tied to it.

**Re-verification (2026-06-08):** No change in the pinning surface this
scan window. The `droid-*` workflows (the highest-risk attack surface, as
they are invoked with `pull-requests: write` and `contents: write`)
remain SHA-pinned. The tag-pinned workflows run on trusted
`pull_request` events but only with `contents: read` (the wider
`contents: write` is reserved for the SHA-pinned droid surface).

**Recommended action (optional, future):**
A scripted migration to SHA-pinned refs across the remaining workflows,
backed by `dependabot.yml` SHA-update support, would bring every action
in line with D-09. This is deferred because the marginal risk reduction
is small and the diff is broad.

## Standing Defenses Verified (No Regression)

The following defenses were re-verified during this scan. All remain intact.

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable` lints denied | `Cargo.toml` | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) — `git`, `GIT_SSH`, `GIT_SSH_COMMAND`, `GIT_ASKPASS`, `GIT_PAGER`, `GIT_EDITOR`, `GIT_PROXY_COMMAND`, `GIT_EXTERNAL_DIFF`, etc. | `crates/tokmd-git/src/command.rs`, `crates/tokmd/src/git_support.rs` | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) — rejects empty, leading `-`, whitespace, control, `\\` | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root (`ParentTraversal`, `Absolute`, `RootEscape`, `Missing`, `Empty`) | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation (4 KiB cap, control char check, `/`/`\\`/Windows drive/`..` rejection) | `crates/tokmd-core/src/ffi/inputs.rs` | ✓ |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs` | ✓ |
| D-08 | Per-family schema versioning (core/analysis/cockpit/handoff/context) | `crates/tokmd-types/src/` | ✓ |
| D-09 | SHA-pinned GitHub Actions on the high-risk Droid surface (mixed tag-pinning on the rest, see OBS-003) | `.github/workflows/*.yml` | ✓ (Droid); partial (rest) |
| D-10 | Branch protection on `main` (1 approval, CODEOWNERS required, `CI (Required)` status check, no force pushes, no deletions, linear history) | `.github/settings.yml` | ✓ |
| D-11 | `cargo-deny` advisory + license allowlist (`RUSTSEC-2020-0163` documented) | `deny.toml` | ✓ |
| D-12 | BLAKE3 redaction with extension allowlist (16-char prefix; safe compound suffixes preserved; unknown/short suffixes strip to bare hash) | `crates/tokmd-format/src/redact/mod.rs`, `crates/tokmd-format/src/redact/extensions.rs` | ✓ |
| D-13 | Content reads bounded by `ContentLimits` (default 128 KiB per file, total `max_bytes` cap) | `crates/tokmd-analysis/src/content/mod.rs` | ✓ |
| D-14 | PyO3 FFI invariants (no panic, GIL release via `py.detach()`, error translation, `?` propagation) | `crates/tokmd-python/src/lib.rs` | ✓ |
| D-15 | WASM uses `MemFs` (no host fs); sandboxed by the WASM runtime | `crates/tokmd-wasm/` | ✓ |
| D-16 | Supply-chain evidence gate invokes `cargo audit` via `Command::arg` (no shell); degrades to `Pending` on parse failure | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-17 | Dependabot enabled for both `cargo` and `github-actions` (weekly, Monday) | `.github/dependabot.yml` | ✓ |
| D-18 | Fuzz harness coverage (18 targets) including FFI envelope, JSON types, path normalize, redaction, gate ratchet, exclude pattern, import parser, badge SVG | `fuzz/fuzz_targets/` | ✓ |
| D-19 | `pull_request_template.md` + `.github/agents/` advisory agent manifests (no embedded secrets) | `.github/`, `.claude/agents/` | ✓ |
| D-20 | `SECURITY.md` documents supported versions, reporting channels (GH Security Advisories preferred), response timeline (48h ack, 7d triage, 30d critical fix), in-scope vs. out-of-scope classes | `SECURITY.md` | ✓ |

## Appendix

### Threat Model

- **Status:** Reused (current)
- **Location:** `.factory/threat-model/threat-model.md`
- **Methodology:** STRIDE
- **Last Reviewed:** 2026-06-01 (7 days ago — within the 90-day cadence)
- **Next Review:** 2026-09-01 (90-day cadence) or upon architecture change

The threat model from 2026-06-01 is current. No architecture change, new
external surface, or new subprocess invocation has been introduced in the
last 7 days. All STRIDE mitigations listed in the threat model were
re-verified against the current `main` (see Standing Defenses above).

### Scan Metadata

- **Commits Scanned:** 1 (`ccd6796af7172fa44c23142b4eb966e5367672c4` — "merge(swarm): import evidence packet contract")
- **Files in scope:** 2437 (entire repository, single-commit import)
- **Scan Duration:** ~5m
- **Skills Used:** commit-security-scan (manual), vulnerability-validation (manual), security-review (manual)
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **False Positive Filter:** applied — see Observations above

### Scan Coverage Matrix

| Area | Files reviewed | Findings |
|------|----------------|----------|
| CLI argv parsing | `crates/tokmd/src/cli/`, `crates/tokmd/src/commands/*.rs` | 0 |
| Subprocess invocation | `crates/tokmd-git/`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd/src/git_support.rs` | 0 |
| Path handling | `crates/tokmd-scan/src/path/`, `crates/tokmd-scan/src/roots.rs`, `crates/tokmd-scan/src/walk/`, `crates/tokmd-scan/src/exclude/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/`, `crates/tokmd-python/src/`, `crates/tokmd-node/src/` | 0 |
| File content reads | `crates/tokmd-analysis/src/content/`, `crates/tokmd-io-port/src/` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/` | 0 |
| GitHub workflows | `.github/workflows/*.yml`, `.github/settings.yml`, `action.yml`, `.github/dependabot.yml`, `.github/CODEOWNERS` | 0 |
| Build / lint | `Cargo.toml`, `deny.toml`, `clippy.toml`, `.cargo/` | 0 |
| Githooks | `.githooks/pre-commit`, `.githooks/pre-push`, `.claude/hooks/format-rust.sh` | 0 |
| Security policies | `SECURITY.md`, `.factory/threat-model/threat-model.md`, `.factory/security/reports/` | 0 |

### Commit-level Analysis

Only one commit falls within the 7-day window:

```
ccd6796af7172fa44c23142b4eb966e5367672c4
Author: Steven Zimmerman, CPA <15812269+EffortlessSteven@users.noreply.github.com>
Date:   Fri Jun 5 07:35:41 2026 -0400
Subject: merge(swarm): import evidence packet contract
```

This is a publication-side merge of the `EffortlessMetrics/tokmd-swarm`
evidence-packet contract into `EffortlessMetrics/tokmd`. It mirrors the
swarm workbench state (the 2026-06-01 scan baseline) into the
publication repository and includes:

- 2437 files added, no removals (`-` / `0` for `Cargo.lock` binary diff).
- All agent manifests under `.claude/agents/`, `.codex/README.md`,
  `.factory/`, `.github/agents/`, `.jules/` (provenance + ambient
  suggestions — see PR triage rules in `AGENTS.md`).
- The full workspace source tree (`crates/`, `xtask/`, `fuzz/`,
  `web/runner/`, `web/ingest/`, etc.).
- CI / release / droid workflows under `.github/workflows/`.
- The 2026-06-01 threat model and security report (carried forward).

**Review of the commit:**

- Touches only files that already exist in the 2026-06-01 baseline; this
  is a publication mirror, not new work.
- No new secrets, no new permissions, no new third-party action.
- No shell-out to untrusted input beyond what the threat model already
  covers.
- No change to environment variable handling, branch protection, or
  release signing.
- `.jules/**` is provenance, intentionally carried (per `AGENTS.md`
  "Jules provenance is intentional repo state").

**No security findings in this commit.**

### Patches Generated

No patches were generated this scan (no findings at or above `medium`).

### Next Scan

The next scheduled security scan runs Monday, 2026-06-15 via
`.github/workflows/droid-security-scan.yml` (cron `0 8 * * 1`).

## References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Previous report: `.factory/security/reports/security-report-2026-06-01.md`
