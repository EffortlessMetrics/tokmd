# Security Scan Report

**Generated:** 2026-10-26
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-10-19 -> 2026-10-26)

## Executive Summary

| Severity | Count | Auto-fixed | Manual Required |
|----------|-------|------------|-----------------|
| CRITICAL | 0     | 0          | 0               |
| HIGH     | 1     | 1          | 0               |
| MEDIUM   | 0     | 0          | 0               |
| LOW      | 0     | 0          | 0               |

**Total Findings:** 1
**Auto-fixed:** 1
**Manual Review Required:** 0

**Summary:** One HIGH finding was identified and auto-fixed: the pinned `zip`
crate version `8.6.0` is affected by three RustSec advisories
(`RUSTSEC-2026-0367`, `RUSTSEC-2026-0368`, `RUSTSEC-2026-0369`), all fixed
in `zip` `9.0.0`. The `zip` crate is reachable in tokmd through the
`archive-zip` byte-mode ingestion feature, which accepts untrusted ZIP bytes
from FFI, WASM, and library callers. The fix upgrades `zip` to `9.1.2` and
adapts the one upstream API change (`ZipFile::name()` now returns a `Result`).

**Reachability validation:** tokmd does **not** call any of the APIs named in
the three advisories. The advisories all describe the extraction APIs
(`ZipArchive::extract`, `ZipStreamReader::extract`), which write archive
entries to the host filesystem. tokmd's ZIP codec adapter
(`crates/tokmd-io-port/src/archive.rs::snapshot_from_zip_bytes`) reads entries
entirely in memory via `by_index` + a bounded `read_to_end`, and hands them to
the admission engine (`snapshot_from_entries`), which rejects symlinks and
other non-regular entries, validates every name against traversal and
absolute-path forms, and stages bytes into an in-memory `MemFs` without ever
touching the host filesystem. A repository-wide search for `.extract(`,
`extract_to`, and `ZipStreamReader` returns no `zip`-crate call sites (the
`.extract()` matches in `tokmd-python` are PyO3's unrelated
`PyObject::extract()`).

The finding is nonetheless reported and fixed at HIGH because the vulnerable
version was **actually shipping** and **actively breaking the supply-chain
gate**: the `cargo deny` advisories check is currently failing on `main`
(`Cargo Deny` and `Tokmd Rust Result` are red on every open PR, including the
2026-10-19 scan PR #2953). A dependency pinned to a version with three open
security advisories is a standing supply-chain risk even when the specific
vulnerable functions are not called, and it blocks the repository's own
advisory gate. The upgrade removes the advisories at the root cause, restores
the gate, and hardens the trust surface against future refactors that might
adopt the extraction APIs.

**Scan-window note:** the 7-day window contains exactly one commit across all
885 remote refs, and it is the previous week's scan report
(`9828d27c1`, 2026-10-19) — no production code changed. `main` is unchanged
since `ff05889` (2026-08-05, **82 days** before this scan). Because no new
code entered the branch, the codebase was reviewed against the existing
threat model and all standing defenses were re-verified against the working
tree rather than a diff. The HIGH finding above was surfaced from the live
`cargo deny` gate failure rather than from the (empty) commit diff.

## Critical Findings

*None.*

## High Findings

### VULN-001: Vulnerable `zip` crate version pinned (RUSTSEC-2026-0367 / 0368 / 0369)

| Attribute | Value |
|-----------|-------|
| **Severity** | HIGH |
| **STRIDE Category** | Elevation of Privilege (primary), Tampering / Information Disclosure (secondary) |
| **CWE** | CWE-269 (Improper Privilege Management), CWE-59 (Improper Link Resolution Before File Access), CWE-732 (Incorrect Permission Assignment for Critical Resource) |
| **File** | `crates/tokmd-io-port/Cargo.toml:30`, `crates/tokmd-core/Cargo.toml:67`, `crates/tokmd-scan/Cargo.toml:38` |
| **Status** | Patched |

**Description:**

Three RustSec advisories affect `zip` versions below `9.0.0`. All three are
reported against the extraction APIs:

- **RUSTSEC-2026-0367** — "Insecure Preservation of SUID/SGID Bits during ZIP
  Extraction Leading to Potential Privilege Escalation". The `zip` crate
  preserves privileged Unix permission bits (SUID, SGID, sticky) from ZIP
  `external_attributes` during extraction instead of masking them. A
  privileged process extracting an attacker-controlled ZIP can create
  setuid-root or setgid files.
- **RUSTSEC-2026-0368** — "`ZipArchive::extract` creates symbolic links whose
  attacker-controlled targets are outside the destination directory". Only the
  output entry path is validated; the link target bytes are passed directly to
  the platform symlink API.
- **RUSTSEC-2026-0369** — "`ZipStreamReader::extract()` and
  `ZipArchive::extract()` apply Unix permissions through symlinks without
  confining the target".

tokmd pinned `zip = "8.6.0"` in three manifests. The crate is pulled in by
the `archive-zip` feature, which exposes the byte-mode ingestion path that
accepts **untrusted ZIP bytes** from FFI (`run_json_bytes`), WASM
(`runJsonBytes`), and library callers. The relevant feature chain is:

```
tokmd-wasm/archive-zip -> tokmd-core/archive-zip -> tokmd-scan/archive-zip
    -> tokmd-io-port/archive-zip -> dep:zip
```

**Evidence:**

Vulnerable version pinned in three manifests:

```toml
# crates/tokmd-io-port/Cargo.toml:30
zip = { version = "8.6.0", optional = true, default-features = false, features = [
    "deflate",
] }
```

```toml
# crates/tokmd-core/Cargo.toml:67
zip = { version = "8.6.0", default-features = false, features = ["deflate"] }
```

```toml
# crates/tokmd-scan/Cargo.toml:38
zip = { version = "8.6.0", default-features = false, features = ["deflate"] }
```

The live `cargo deny` advisories gate fails on `main` with exactly these three
advisories (from the `Cargo Deny` job on PR #2953, run 39221549964):

```
error[vulnerability]: Insecure Preservation of SUID/SGID Bits during ZIP
Extraction Leading to Potential Privilege Escalation
   ID: RUSTSEC-2026-0367
error[vulnerability]: zip: `ZipArchive::extract` creates symbolic links whose
attacker-controlled targets are outside the destination directory
   ID: RUSTSEC-2026-0368
error[vulnerability]: ZipStreamReader::extract() and ZipArchive::extract()
apply Unix permissions through symlinks without confining the target
   ID: RUSTSEC-2026-0369
advisories FAILED, bans ok, licenses ok, sources ok
```

**Reachability validation (why this is not a false positive, and why it is
still only HIGH rather than CRITICAL):**

A repository-wide search confirms tokmd never calls the vulnerable APIs:

```
$ grep -rn "\.extract(\|extract_to\|ZipStreamReader" crates/ xtask/ --include=*.rs
crates/tokmd-python/src/runtime.rs:168:  ... .extract()?;      # PyO3 PyObject::extract
crates/tokmd-python/src/envelope.rs:20:   ... .extract()?;      # PyO3 PyObject::extract
...
```

The only `zip` production call sites are in the codec adapter, which reads in
memory and never extracts to disk:

```rust
// crates/tokmd-io-port/src/archive.rs:473-492
let mut file = archive
    .by_index(index)
    .map_err(|err| ArchiveError::MalformedArchive { reason: err.to_string() })?;

// Raw, untrusted stored name; the admission engine validates it.
let name = file.name().to_string();

// Classify: symlink/device/other (via unix type bits) before directory,
// so a symlink whose name lacks a trailing slash is still rejected.
let kind = classify_zip_entry(file.unix_mode(), file.is_dir());
```

The admission engine then rejects the specific classes the advisories
describe, and stages into `MemFs` (no host filesystem):

```rust
// crates/tokmd-io-port/src/archive.rs:331-350
let mut staged = MemFs::new();
...
match entry.kind {
    EntryKind::Other => {
        return Err(ArchiveError::NonRegularEntry { name: entry.name });
    }
    EntryKind::Directory => { continue; }
    EntryKind::File => {}
}
```

Because the vulnerable functions are not called, there is no direct
privilege-escalation path in tokmd today. The finding is HIGH rather than
CRITICAL on that basis. It is reported as HIGH (not below threshold) because:

1. The vulnerable version is the version that actually ships and is pinned in
   three manifests, so any future change that adopts the documented extraction
   API becomes immediately exploitable with no further review trigger.
2. The repository's own advisory gate is red on `main` because of it, so the
   project's declared supply-chain control is not functioning.
3. The fix is a one-line-per-manifest dependency bump with no behavioral
   downside, which makes leaving it unfixed strictly worse than fixing it.

**Fix Applied:**

```diff
--- a/crates/tokmd-io-port/Cargo.toml
+++ b/crates/tokmd-io-port/Cargo.toml
-zip = { version = "8.6.0", optional = true, default-features = false, features = [
+zip = { version = "9.1.2", optional = true, default-features = false, features = [
     "deflate",
 ] }
```

```diff
--- a/crates/tokmd-core/Cargo.toml
+++ b/crates/tokmd-core/Cargo.toml
-zip = { version = "8.6.0", default-features = false, features = ["deflate"] }
+zip = { version = "9.1.2", default-features = false, features = ["deflate"] }
```

```diff
--- a/crates/tokmd-scan/Cargo.toml
+++ b/crates/tokmd-scan/Cargo.toml
-zip = { version = "8.6.0", default-features = false, features = ["deflate"] }
+zip = { version = "9.1.2", default-features = false, features = ["deflate"] }
```

`zip` 9 changes one API used by tokmd: `ZipFile::name()` now returns
`ZipResult<Cow<'_, str>>` instead of `Cow<'_, str>`. The adapter was adapted
to fail closed on a name-decode error rather than silently coercing it:

```diff
--- a/crates/tokmd-io-port/src/archive.rs
+++ b/crates/tokmd-io-port/src/archive.rs
         // Raw, untrusted stored name; the admission engine validates it.
-        let name = file.name().to_string();
+        //
+        // `ZipFile::name` is fallible: the raw name is decoded from the entry
+        // header and can fail to decode. A name that cannot be decoded is
+        // treated as a malformed archive rather than being silently coerced.
+        let name = file
+            .name()
+            .map_err(|err| ArchiveError::MalformedArchive {
+                reason: err.to_string(),
+            })?
+            .into_owned();
```

The upstream `zip` 9 fixes are confirmed present in the resolved dependency:

```rust
// zip-9.1.2/src/format/functions.rs:27
unix_perms & !0o7000     // masks SUID/SGID/sticky bits

// zip-9.1.2/src/read/extract.rs
set_permissions_no_follow_symlink(...)   // no permission writes through symlinks
```

**Validation performed:**

| Check | Result |
|-------|--------|
| `cargo build --workspace --all-features` | pass |
| `cargo build -p tokmd-wasm --target wasm32-unknown-unknown --features archive-zip` | pass (`nix` is only pulled in on macOS/BSD targets) |
| `cargo test -p tokmd-io-port --all-features` | pass (44 tests across 6 binaries) |
| `cargo test -p tokmd-core --all-features` | pass (1123 tests), including `byte_mode_fails_closed_on_hostile_entry`, `byte_mode_enforces_archive_limits`, `byte_mode_rejects_malformed_archive` |
| `cargo test -p tokmd-scan --all-features` | pass (19 binaries, 0 failures) |
| `cargo test --workspace --all-features` | no new failures (see note below) |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-features --all-targets -- -D warnings` | pre-existing failure only (see note below) |
| `cargo xtask gate --check` | 3/4 steps pass; clippy step fails on the pre-existing issue below |
| `cargo deny check advisories` | not run locally (cargo-deny not installed); verified by resolved version `zip 9.1.2 >= 9.0.0` |

**Pre-existing failures (NOT caused by this change, verified by re-running at
the base commit `ff05889`):**

- `cargo clippy` — `error: useless use of format!` at
  `crates/tokmd-cockpit/src/render/evidence.rs:59`. Identical error at base.
- `cargo test -p xtask --test publish_w71` —
  `publishable_internal_dev_dependencies_use_loose_versions` fails on internal
  `tokmd-*` dev-dependency version forms. Identical failure at base.
- `cargo test -p tokmd-wasm --target wasm32-unknown-unknown` — `Exec format
  error (os error 8)`; environmental (no wasm runtime configured locally).
  Identical at base.

The failing-test-binary set was compared before and after the change and is
identical (`--test publish_w71` only), confirming no regressions were
introduced.

## Medium Findings

*None.*

## Low Findings

*None.*

## Observations (Below Threshold — Not Reported As Findings)

These items were considered during the scan but do not meet the `medium`
severity threshold. All "carried" observations are unchanged from the
2026-10-19 baseline, except OBS-006 and OBS-007, which are new this scan.

### OBS-001 (carried): FFI JSON payload size not bounded

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-core/src/ffi/mod.rs` |
| **Status** | Not patched — design choice |

**Description:** The `run_json(mode, args_json)` FFI entrypoint accepts a JSON
string of arbitrary size. While individual in-memory `inputs[].path` is
bounded to 4096 bytes (`MAX_IN_MEMORY_INPUT_PATH_BYTES` at
`crates/tokmd-core/src/ffi/inputs.rs:12`, re-verified this scan), the outer
JSON envelope is not.

**Why not a finding:** Caller controls input. `serde_json::from_str` allocates
predictably; no algorithmic blowup. No `medium` reachability: requires the
caller to opt in. Out of scope per `SECURITY.md`.

**Recommended fix (optional, future):** Add a soft cap on `args_json.len()`
(e.g. 8 MiB) returning a typed
`TokmdError::invalid_field("args", "JSON args exceed 8 MiB cap")` from
`run_json_inner`.

### OBS-002 (carried): Transitive `RUSTSEC-2020-0163` advisory

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (transitive) |
| **STRIDE Category** | Elevation of Privilege |
| **File** | `Cargo.lock` (transitive `term_size` via `tokei`) |
| **Status** | Documented in `deny.toml` |

**Description:** `term_size` is a transitive dependency of `tokei` and has an
unmaintained advisory (`RUSTSEC-2020-0163`). Still present at version `0.3.2`
after this scan's `zip` upgrade.

**Why not a finding:** Already documented in `deny.toml` with rationale. Out of
scope per `SECURITY.md` (third-party dependency issues are reported upstream).

**Recommended action:** Track upstream `tokei` for a `term_size` removal.

### OBS-003 (carried): GitHub Actions pinning is mixed (tag + SHA)

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing / Tampering |
| **File** | `.github/workflows/*.yml` |
| **Status** | Not patched — mixed strategy |

**Description:** The Droid-related workflows (`droid.yml`,
`droid-review.yml`, `droid-security-scan.yml`) pin the custom Droid action
by SHA (`EffortlessMetrics/droid-action-safe@7c1377ccbacddc95560d1570547a5baa51de01ec`,
re-verified present at all three sites this scan). Other workflows pin
first-party actions by SHA in release-grade workflows and by tag in
development CI. The threat model claims SHA pinning workspace-wide, which is
not strictly accurate for every non-Droid workflow.

**Why not a finding:** Tag-pinned first-party actions (`actions/*`) are a
well-accepted practice with low residual risk. The custom Droid action — the
highest-privilege third-party surface — IS SHA-pinned. Below the `medium`
threshold.

**Recommended action (optional, future):** Convert all third-party actions to
SHA-pinned references and codify the rotation process in `.factory/rules/`,
or update the threat model to describe the mixed policy accurately.

### OBS-004 (carried): `web/runner` browser code does not pin GitHub API base URL

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing |
| **File** | `web/runner/ingest.js` |
| **Status** | Not patched — review for future |

**Description:** The browser-side runner fetches repository content via
`fetch()` calls to `api.github.com` and the codeload/GitHub
`releases`/`archive` endpoints, with URLs hard-coded in the `web/runner/`
JavaScript modules. The token (when supplied) is stored in `sessionStorage`
and used as a `Bearer` header. There is no Subresource Integrity pinning or
origin allow-listing on the client-side fetch surface.

**Why not a finding:** All sensitive fetches target well-known HTTPS GitHub
endpoints. The token lifetime is bounded to a single browser tab
(`sessionStorage`, re-verified at `web/runner/auth.js:9`). No DOM injection
surfaces: all dynamic data is rendered via `textContent` (15 uses in
`web/runner/main.js`); a repository-wide grep for `innerHTML`,
`document.write`, `eval(`, and `new Function` across `web/runner/` returns no
matches (re-verified this scan). Below the `medium` threshold.

**Recommended action (optional):** Consider an explicit allowlist of fetch
origins and a CSP `connect-src` directive in the runner's served HTML.

### OBS-005 (carried): `action.yml` install step performs `curl | sh` style download

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Tampering / Information Disclosure |
| **File** | `action.yml` (composite step `Install tokmd`) |
| **Status** | Not patched — verified checksums |

**Description:** The composite GitHub Action downloads a pre-built `tokmd`
binary from the releases endpoint and verifies it against `checksums.txt`
(sha256). It does not verify a cryptographic signature on the checksum file
or on the release itself. The download URL is interpolated from a user-supplied
`version` input without shell-unsafe character filtering.

**Why not a finding:** Consumers control which version they pin to. `curl
-fsSL` rejects HTTP errors. The checksum verification uses
`sha256sum`/`shasum`/`Get-FileHash`. Build provenance is separately attested
via `actions/attest-build-provenance@v4` in `release.yml`. Below the `medium`
threshold.

**Recommended action (optional):** Add explicit format validation for the
`version` input (e.g. regex `^v?\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$`) before
constructing the URL.

### OBS-006 (new): Branch protection requires a status check that no workflow emits

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Tampering / Spoofing (gate bypass) |
| **File** | `.github/settings.yml:56` |
| **Status** | Not patched — threat model corrected |

**Description:** `.github/settings.yml` requires the status check
`"Codex Review Gate"` on `main` (added 2026-08-02 in `e515fadd6`, alongside
the deliberate relaxation of `required_approving_review_count` from `1` to
`0` and `require_code_owner_reviews` from `true` to `false`). A
repository-wide search finds the string `Codex Review Gate` in exactly one
place — the `settings.yml` context list itself. No workflow job, no
`xtask` task, and no external system in this repository emits a check with
that name. The live check list on PR #2953 confirms it: the check runs
`Tokmd Rust Result`, `CI Lane Whitelist`, `Cargo Deny`, `droid-review`,
`droid`, `ripr`, `CodeRabbit`, `GitGuardian Security Checks`, and others, but
no `Codex Review Gate`.

**Why not a finding:** The threat model explicitly states that live branch
protection is control-plane state that must be queried separately, and that
the settings file is not proof that `main` is protected. The setting may
correspond to an external reviewer service (Codex) whose checks are posted
outside this repository, and the intent is documented in the file's own
comments ("Codex is the exact-head reviewer for this single-maintainer
workflow"). A stale required-check name typically blocks merges rather than
enabling them, so the practical risk is availability, not bypass. It is
below the `medium` threshold because no attacker-controlled path to a merge
bypass is demonstrated.

**Action taken:** The threat model's standing-defense item 9 was corrected to
describe the actual posture (0 required approvals, both required status
checks, non-linear history permitted) and to record that the
`Codex Review Gate` check has no in-repo producer and that
`Tokmd Rust Result` is currently red on `main`.

**Recommended action (optional, future):** Confirm whether the
`Codex Review Gate` check is produced by an external service. If it is not,
remove it from the `required_status_checks` context list so the required-check
set matches reality; if it is, document the producing system in
`docs/ci/`.

### OBS-007 (new): `ci.yml` passes an input the `ub-review` action does not accept

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Tampering (advisory lane) |
| **File** | `.github/workflows/ci.yml:158` |
| **Status** | Not patched — already fixed on an unmerged branch |

**Description:** The `UB Review (advisory)` step in `.github/workflows/ci.yml`
passes `pr-thread-context: target/ci-core/precontext.md` to the
`EffortlessMetrics/ub-review` action. The live CI log on PR #2953 shows the
action rejecting it:

```
##[warning]Unexpected input(s) 'pr-thread-context', valid inputs are
['preset', 'profile', 'root', 'base', 'head', 'out', 'install-tools',
 'tool-bundle', 'setup-rust', 'install-mode', 'binary-path', 'allow-heavy',
 'posting', 'mode', 'github-token', ...]
```

The pinned action version does not accept that input, so the deterministic
precontext file is generated by the `ci-core` step and then never consumed by
the advisory reviewer.

**Why not a finding:** The `ub-review` step is explicitly advisory
(`continue-on-error: true`, "UB Review (advisory)"), and the
`Tokmd Rust Result` gate does not depend on it — the workflow comments state
that the "deterministic gate verdict stands" independently of the advisory
lane. The stale input does not weaken the merge gate; it only reduces the
context available to an advisory reviewer. It is below the `medium` threshold.

**Action taken:** None in this scan. Commit `c9bed621e` (2026-10-07, branch
`origin/surveyor-workspace-hygiene-6254374676452782516`, outside this scan
window and not merged to `main`) already removes the `pr-thread-context` line
from `.github/workflows/ci.yml` and drops the corresponding marker from
`xtask/src/tasks/ci_gate_contract.rs::REQUIRED_MARKERS`. This scan did not
duplicate that change, because the `REQUIRED_MARKERS` edit is part of a
separate, unmerged branch's file-policy scope and touching
`xtask/src/tasks/ci_gate_contract.rs` here would create a conflict with it.

**Recommended action:** Land or cherry-pick `c9bed621e` (or its `ci.yml` hunk
plus the matching `REQUIRED_MARKERS` removal) so the advisory lane consumes
the precontext file as intended.

## Standing Defenses Verified (No Regression)

Each defense below was re-verified against the working tree during this scan.
All remain intact.

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml:56` | yes |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `todo`, `unimplemented` lints denied | `Cargo.toml:65-71` | yes |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs:9` (16 vars), `crates/tokmd/src/git_support.rs:3`, `crates/tokmd-scan/src/walk/git.rs:12` | yes |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) | `crates/tokmd-git/src/refs.rs:13`, `:80` | yes |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` (`ensure_under_root`), `validated_root.rs` | yes |
| D-06 | FFI in-memory input path validation (4 KiB cap) | `crates/tokmd-core/src/ffi/inputs.rs:12`, `:116` | yes |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs`, `settings_parse.rs` | yes |
| D-08 | Per-family schema versioning (`SCHEMA_VERSION=2`, `ANALYSIS_SCHEMA_VERSION=9`, `COCKPIT_SCHEMA_VERSION=3`, `HANDOFF_SCHEMA_VERSION=5`, `CONTEXT_SCHEMA_VERSION=4`, `CONTEXT_BUNDLE_SCHEMA_VERSION=2`) | `crates/tokmd-types/src/`, `crates/tokmd-analysis-types/src/lib.rs:101` | yes |
| D-09 | SHA-pinned Droid-related actions; SHA-pinned release-grade first-party actions; tag-pinned development CI actions | `.github/workflows/droid*.yml` | yes |
| D-10 | Branch protection on `main` | `.github/settings.yml` | yes (see OBS-006; posture corrected in the threat model) |
| D-11 | `cargo-deny` advisory + license allowlist | `deny.toml` | yes (gate is red due to VULN-001; now fixed) |
| D-12 | BLAKE3 redaction with extension allowlist | `crates/tokmd-format/src/redact/mod.rs:91` | yes |
| D-13 | Content reads bounded by `ContentLimits` | `crates/tokmd-analysis/src/content/mod.rs:17` (`DEFAULT_MAX_FILE_BYTES = 128 * 1024`) | yes |
| D-14 | PyO3 FFI invariants (no panic, GIL release, error translation) | `crates/tokmd-python/src/lib.rs` | yes |
| D-15 | WASM uses `MemFs` (no host fs) | `crates/tokmd-io-port/src/lib.rs:80`, `crates/tokmd-wasm/src/lib.rs` | yes |
| D-16 | `web/runner` browser runner uses `textContent` (no `innerHTML`/`eval`/`new Function`/`document.write`) | `web/runner/main.js` | yes |
| D-17 | `web/runner` token stored in `sessionStorage` (not `localStorage`) | `web/runner/auth.js:9` | yes |
| D-18 | `web/runner` worker protocol allowlists modes & presets | `web/runner/messages.js:39`, `:135` | yes |
| D-19 | Composite action installs tokmd with sha256 checksum verification | `action.yml:258-274` | yes |
| D-20 | Custom Droid action SHA-pinned across all Droid workflows | `.github/workflows/droid*.yml` | yes |
| D-21 | `cargo audit` invoked with structured `--json` output; malformed JSON treated as Pending | `crates/tokmd-cockpit/src/supply_chain.rs:36`, `:134` | yes |
| D-22 | `run_json` top-level JSON must be an object (strict shape check) | `crates/tokmd-core/src/ffi/mod.rs:74` | yes |
| D-23 | Author DAG import via true-merge commits (no force-push of publication history) | repository topology | yes |
| D-24 | No shell invocation of user-controlled input (`Command::new` with `arg()`, never `sh -c`) | `crates/tokmd-git/`, `crates/tokmd-cockpit/`, `crates/tokmd-scan/src/walk/git.rs`, `crates/tokmd-core/src/context_git/mod.rs` | yes |
| D-25 | ZIP byte-mode admission engine: symlink/non-regular rejection, name traversal validation, `MemFs` staging, per-entry/total/ratio caps | `crates/tokmd-io-port/src/archive.rs` (`snapshot_from_entries`, `validate_and_normalize_name`) | yes |

## Appendix

### Threat Model

- **Status:** Refreshed this scan (was 85 days old, due 2026-10-31)
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Reviewed (content):** 2026-08-02
- **Last Committed:** 2026-08-05 in commit `ff05889`
- **Refresh applied by this scan:** standing-defense item 9 (branch-protection
  posture) corrected to match `.github/settings.yml` as verified this scan;
  header `Last Reviewed` and the §6 review-cadence next-review date updated.
  No STRIDE analysis was invalidated: no new external surface, subprocess
  invocation, or trust boundary was introduced in the scan window.
- **Methodology:** STRIDE
- **Next review due:** 2027-01-24 (90 days after the 2026-10-26 refresh)

### Scan Metadata

- **Commits Scanned:** 0 production commits. The 7-day window
  (`git log --all --since="7 days ago"`) contains exactly one commit across all
  885 remote refs: `9828d27c1` (2026-10-19), the previous week's scan report,
  which touches only `.factory/security/reports/security-report-2026-10-19.md`.
  The most recent commit on `main` is `ff05889` from 2026-08-05, **82 days**
  before this scan.
- **Files in scope:** Not applicable from a diff — no production code changed
  in the window. The scan therefore proceeded as a full standing-defense
  re-verification against the working tree, plus a live-gate audit of the
  repository's `cargo deny` advisories check, which surfaced VULN-001.
- **Scan Duration:** ~35m
- **Skills Used:** threat-model-generation (refresh applied),
  commit-security-scan, vulnerability-validation, security-review
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **False Positive Filter:** applied — see the reachability analysis under
  VULN-001, which establishes that the advisory-named APIs are not called
  while still justifying the fix.

### Scan Coverage Matrix

Because no production diff was present in the 7-day window, the coverage
matrix below re-states the comprehensive coverage applied to the underlying
baseline.

| Area | Files reviewed | Findings |
|------|----------------|----------|
| CLI argv parsing | `crates/tokmd/src/cli/`, `crates/tokmd/src/commands/*.rs` | 0 |
| Subprocess invocation | `crates/tokmd-git/`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd-cockpit/src/gates/contracts.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs`, `crates/tokmd-core/src/context_git/mod.rs` | 0 |
| Path handling | `crates/tokmd-scan/src/path/` (`bounded_path.rs`, `validated_root.rs`, `error.rs`), `crates/tokmd-scan/src/roots.rs`, `crates/tokmd-scan/src/walk/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/` (`mod.rs`, `inputs.rs`, `parse.rs`, `byte_mode.rs`, `envelope.rs`, `modes.rs`, `settings_parse.rs`), `crates/tokmd-python/src/`, `crates/tokmd-node/src/` | 0 |
| Archive / ZIP ingestion | `crates/tokmd-io-port/src/archive.rs` (admission engine + ZIP codec), `crates/tokmd-scan/src/`, `crates/tokmd-core/tests/archive_*.rs` | **1 (VULN-001)** |
| File content reads | `crates/tokmd-analysis/src/content/`, `crates/tokmd-io-port/src/` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/` (`mod.rs`, `extensions.rs`) | 0 |
| GitHub workflows | `.github/workflows/*.yml` (29 files), `.github/settings.yml`, `action.yml` | 0 findings (2 observations) |
| Build / lint / supply chain | `Cargo.toml`, `Cargo.lock`, `deny.toml`, `clippy.toml`, `.cargo/config.toml` | **1 (VULN-001)** |
| Githooks | `.githooks/pre-commit`, `.githooks/pre-push`, `.claude/hooks/format-rust.sh` | 0 |
| Web runner (browser) | `web/runner/main.js`, `worker.js`, `auth.js`, `messages.js`, `runtime.js`, `ingest.js` | 0 |
| Threat model | `.factory/threat-model/threat-model.md` | refreshed (item 9 corrected) |

### Commit-level Analysis

The 7-day window (2026-10-19 -> 2026-10-26) contains no production commits
across any of the 885 remote refs. The single commit in the window is the
prior scan's report:

```
9828d27c1f36163d7ef14565a7191ff4b21c8fad
Author: Droid Security Scan
Date:   Mon Oct 19 08:15:45 2026 +0000
Subject: fix(security): Security scan report - 2026-10-19

 .factory/security/reports/security-report-2026-10-19.md | 373 +++++++++++++++++++++
 1 file changed, 373 insertions(+)
```

That branch (`origin/droid/security-report-2026-10-19`) is a direct child of
`main` and has not been merged (`gh pr view 2953` reports `state: OPEN`,
`mergeStateStatus: BLOCKED`).

The most recent commit on `main` is the true-merge publication import:

```
ff0588903142cedda2dbc4903bbb7128fa1cbb3e
Author: Steven Zimmerman, CPA <15812269+EffortlessSteven@users.noreply.github.com>
Date:   Wed Aug 5 01:44:26 2026 -0400
Subject: import: finalize stable release after consumer proof
```

This is **82 days** before the scan date and was already covered by the
2026-07-27 and 2026-10-19 scans. The only non-scan commits in the last 30 days
are `c9bed621e` / `0d9febe00` (2026-10-07, "Fix workspace publication
boundaries and file policy drift"), which live only on
`origin/surveyor-workspace-hygiene-6254374676452782516` and are not merged to
`main`. Their diff (removal of `pr-thread-context` from `ci.yml` and from the
`ci_gate_contract` REQUIRED_MARKERS list) was reviewed for security impact
and found to be a correctness fix, not a gate weakening — see OBS-007.

Because the window produced no diff, the scan additionally audited the
**live** repository gates, which is how VULN-001 was found: the `Cargo Deny`
job on PR #2953 (run `39221549964`) reports `advisories FAILED` with the
three `zip` advisories, and `Tokmd Rust Result` is consequently red on
`main`. The `CI Lane Whitelist` check is also red on that PR.

### Patches Generated

One patch was generated and applied this scan (VULN-001):

- `crates/tokmd-io-port/Cargo.toml` — `zip` `8.6.0` -> `9.1.2`
- `crates/tokmd-core/Cargo.toml` — `zip` `8.6.0` -> `9.1.2`
- `crates/tokmd-scan/Cargo.toml` — `zip` `8.6.0` -> `9.1.2`
- `crates/tokmd-io-port/src/archive.rs` — adapt to the fallible
  `ZipFile::name()` API, failing closed on a decode error
- `Cargo.lock` — `zip 9.1.2`, `typed-path 0.13.0`, plus `cfg_aliases` and
  `nix` (macOS/BSD-only) added transitively

Commit: `fix(security): [VULN-001] Upgrade zip 8.6.0 -> 9.1.2 for RUSTSEC-2026-0367/0368/0369`

### Next Scan

The next scheduled security scan runs Monday, 2026-11-02 via
`.github/workflows/droid-security-scan.yml` (cron `0 8 * * 1`).

Recommended follow-ups before then:

1. Confirm the `Codex Review Gate` required status check has a producer, or
   remove it from `.github/settings.yml` (OBS-006).
2. Land the `ci.yml` `pr-thread-context` removal so the advisory reviewer
   consumes the deterministic precontext (OBS-007).
3. Clear the `CI Lane Whitelist` failure on `main`.
4. Address the two pre-existing failures that keep `cargo xtask gate --check`
   red on `main`: the `tokmd-cockpit` `useless use of format!` clippy error
   and the `xtask` `publish_w71` internal dev-dependency version test.

## References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [RUSTSEC-2026-0367](https://rustsec.org/advisories/RUSTSEC-2026-0367.html)
- [RUSTSEC-2026-0368](https://rustsec.org/advisories/RUSTSEC-2026-0368.html)
- [RUSTSEC-2026-0369](https://rustsec.org/advisories/RUSTSEC-2026-0369.html)
- [GHSA-vv5m-jxxh-4627 (zip SUID/SGID)](https://github.com/zip-rs/zip2/security/advisories/GHSA-vv5m-jxxh-4627)
- [GHSA-4v9h-p7m9-rh26 (zip symlink escape)](https://github.com/zip-rs/zip2/security/advisories/GHSA-4v9h-p7m9-rh26)
- [GHSA-4684-h2q9-2vcp (zip symlink permissions)](https://github.com/zip-rs/zip2/security/advisories/GHSA-4684-h2q9-2vcp)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Previous scans: `.factory/security/reports/security-report-2026-06-01.md`,
  `security-report-2026-06-08.md`, `security-report-2026-06-29.md`,
  `security-report-2026-07-06.md`, `security-report-2026-07-13.md`,
  `security-report-2026-07-20.md`, `security-report-2026-07-27.md`,
  `security-report-2026-10-05.md`, `security-report-2026-10-12.md`,
  `security-report-2026-10-19.md`
