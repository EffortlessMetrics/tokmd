# Security Scan Report

**Generated:** 2026-11-09
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-11-02 → 2026-11-09)

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

**Summary:** One HIGH severity finding (shell injection in the composite
GitHub Action) was identified and auto-patched. No CRITICAL or MEDIUM findings
were identified at or above the configured threshold.

**Scan scope note.** This repository is the *publication* repository. Its
history is squashed into a single root commit (`ff05889`, 2026-08-05), so the
literal 7-day window (`git log --since="7 days ago"`) contains **zero commits**
and yields an empty file list. To produce meaningful coverage rather than a
vacuous "no changes, no findings" result, this scan covered the **entire
current tree** (2,589 files, 1,245 Rust source files) against the standing
threat model, and focused analysis on the surfaces that the 2026-07-27 scan
identified as the highest-risk trust boundaries. All 23 previously recorded
standing defenses were re-verified against the current tree.

## Critical Findings

*None.*

## High Findings

### VULN-001: Shell injection via the `version` input of the composite GitHub Action

| Attribute | Value |
|-----------|-------|
| **Severity** | HIGH |
| **STRIDE Category** | Elevation of Privilege / Tampering |
| **CWE** | CWE-94 (Improper Control of Generation of Code) / CWE-78 (OS Command Injection) |
| **File** | `action.yml:222` |
| **Status** | Patched |

**Description:**

The composite Action's "Install tokmd" step interpolated the `version` input
directly into the step's `run:` script body:

```yaml
    - name: Install tokmd
      shell: bash
      env:
        TOKMD_ACTION_RUNTIME: ${{ inputs.runtime }}
        TOKMD_ACTION_IMAGE: ${{ inputs.image }}
        TOKMD_ACTION_VERSION: ${{ inputs.version }}
      run: |
        ...
        version="${{ inputs.version }}"     # <-- line 222, vulnerable
```

GitHub Actions substitutes `${{ ... }}` expressions into the **script text**
before bash parses it. The expression therefore does not behave like a shell
variable reference; it is textual splicing into the source of the script. A
`version` value that closes the double-quoted assignment and appends a command
becomes a separate command that bash executes in the step's shell, with the
workflow runner's full privileges and ambient credentials (including any
`secrets.*` the workflow exposes to that step).

This was the **only** input in `action.yml` handled this way. Every other
input (`runtime`, `image`, `artifact`, `base`, `head`, `mode`, `paths`,
`preset`, `output-dir`, `format`, `top`, `fail-on`, `syntax`,
`context-budget`, `module-roots`) was correctly passed through the step-level
`env:` block and read as a quoted shell variable — which is the pattern that
is safe against this class of issue. The `version` input was the outlier,
which indicates an oversight rather than an intentional design.

**Evidence:**

Vulnerable line (`action.yml:222` before the fix):

```yaml
        version="${{ inputs.version }}"
```

Proof of execution (modeling GitHub's substitute-then-parse behavior with the
attacker-supplied value `1.0.0"; echo PWNED_INJECTION_PROOF > /tmp/INJECTION_MARKER; echo "`):

```
--- script bash will execute (as GitHub would hand it) ---
version="1.0.0"; echo PWNED_INJECTION_PROOF > /tmp/INJECTION_MARKER; echo ""
os="${RUNNER_OS:-Linux}"
--- executing ---
--- marker ---
PWNED_INJECTION_PROOF
INJECTION CONFIRMED
```

The injected command ran as a separate command in the step's shell.

**Reachability:**

- **In-repo workflows:** not reachable. Every `with: version:` value in
  `.github/workflows/test-action.yml` and `release-consumer-smoke.yml` is a
  hardcoded literal (`1.14.0`, `0.0.0`), and the `release-consumer-smoke.yml`
  invocation omits `version` entirely.
- **Published Action:** reachable. `action.yml` is the published Action
  surface (`EffortlessMetrics/tokmd@vX`). Any consumer that wires an
  untrusted value into `with: version:` — for example
  `version: ${{ inputs.requested_version }}` where the input is derived from
  a PR title, issue body, or matrix value — inherits arbitrary command
  execution in their workflow. The consumer controls which Action version
  they pin, but cannot control what a downstream `with:` expression evaluates
  to.

**Impact:** Arbitrary command execution in the caller's workflow context.
Depending on the caller's workflow, this can expose repository secrets, use the
`GITHUB_TOKEN`, tamper with build artifacts, or poison caches. `SECURITY.md`
explicitly lists "Command injection: Unsanitized input passed to shell commands"
as a vulnerability in scope for this project.

**Fix Applied:**

```diff
-        version="${{ inputs.version }}"
+        # Read the version input from the environment (never interpolate the
+        # expression into this script body: GitHub substitutes ${{ ... }} into
+        # the script text before bash parses it, so a value containing shell
+        # metacharacters would execute as commands in this step).
+        version="${TOKMD_ACTION_VERSION:-}"
+        # Constrain the version to a plain release tag so it can only ever
+        # produce a well-formed release download URL.
+        if [ "$version" != "latest" ] && ! [[ "$version" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.]+)?$ ]]; then
+          echo "::error::version must be 'latest' or a MAJOR.MINOR.PATCH release tag (optionally with a -suffix); received '$version'"
+          exit 1
+        fi
         os="${RUNNER_OS}"
```

The fix has two parts:

1. **Removes the injection primitive.** The value now reaches the script only
   through the `TOKMD_ACTION_VERSION` environment variable, which bash
   expands as a single quoted word and never re-parses. This matches the
   pattern already used by every other input in the file and by the
   `runtime=container` branch, which already read `TOKMD_ACTION_VERSION`.
2. **Adds input validation (defense in depth).** The version is constrained to
   `latest` or a `MAJOR.MINOR.PATCH[-suffix]` tag, so it can only ever
   produce a well-formed release download URL. This also closes the
   previously carried OBS-005 recommendation.

**Validation performed:**

- YAML parses cleanly; the extracted step body passes `bash -n`.
- The **real** extracted step script was executed with the malicious version
  `1.0.0"; echo PWNED > /tmp/INJECTION_MARKER; echo "`: it now exits with
  `::error::version must be 'latest' or a MAJOR.MINOR.PATCH release tag` and
  the marker file is **not** created.
- The same script was executed with legitimate values `1.14.0` and `latest`:
  both proceed to download the release, verify the SHA-256 checksum, and
  install the binary successfully (`tokmd 1.14.0`).
- Additional malicious forms were all rejected: backtick substitution,
  embedded newline, `&&` chaining, `$(...)` substitution, `;` chaining,
  `../` path traversal, and the empty string. All legitimate forms used by
  this repository (`1.14.0`, `v1.15.0`, `latest`, `1.15.0-rc.1`) are
  accepted.

---

## Medium Findings

*None.*

## Low Findings

*None.*

## Observations (Below Threshold — Not Reported As Findings)

These items were considered during the scan but do not meet the `medium`
severity threshold. They are recorded for traceability and for the next
scheduled scan.

### OBS-006 (new): ZIP central-directory parse is not bounded by `ArchiveLimits`

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-io-port/src/archive.rs` (`snapshot_from_zip_bytes`) |
| **Status** | Partially mitigated (VULN-002); residual limitation is upstream |

**Description:** `snapshot_from_zip_bytes` opens the archive with
`zip::ZipArchive::new`, which parses the **entire** central directory and
allocates `Vec::with_capacity(number_of_files)` before any of tokmd's own
limits are consulted. The `zip` crate's `Config` exposes only `archive_offset`
— there is no API to bound the entry count — and the `number_of_files` value
is taken from the archive's End Of Central Directory record.

Measured amplification (test harness, 300,000 stored entries):

```
archive = 28786 kB, baseline peak = 150128 kB
  [A] after ZipArchive::new only: peak = 218036 kB (delta = 67908 kB), len=300000
  [B] after decoding all entries: peak = 218084 kB (delta = 48 kB), payload=300000
  => central-directory parse accounts for 67908 kB of the 67956 kB total
```

The central-directory parse accounts for **99.93%** of the allocation. The
total-size cap (`max_total_size`) accounts only for payload bytes, so it does
not bound this structural overhead.

**Why not a finding:**

- The outcome is **fail-closed**: the archive is rejected with
  `ArchiveError::TooManyEntries`; there is no bypass, no corruption, and no
  code execution.
- Resource use is proportional to input size (measured ~2.4x archive size),
  which is normal for archive processing, not an unbounded bomb.
- Spoofing the declared entry count does **not** work — the `zip` crate
  validates the EOCD count against the actually parsed central directory
  (verified experimentally: a 200-byte archive with a patched count of 60,000
  still yields `len=2`).
- The attacker must supply the archive the user is already choosing to
  process. tokmd is a CLI/library that analyzes the user's own repositories;
  the blast radius is the caller's own process.
- The `archive-zip` feature is **opt-in** (not enabled by default in
  `tokmd-wasm`, `tokmd-python`, or `tokmd-node`).

**Recommended action (optional, future):** Track upstream `zip` for an
entry-count limit in `Config`. Until then, a caller-facing mitigation is to
cap the accepted archive byte length before decoding (the WASM host already
bounds input by the browser's own memory limits).

### OBS-001 (carried): FFI JSON payload size not bounded

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-core/src/ffi/mod.rs` (`run_json_inner`) |
| **Status** | Not patched — design choice |

**Description:** `run_json(mode, args_json)` accepts a JSON string of
arbitrary size. Individual in-memory `inputs[].path` values are bounded to
4096 bytes (`MAX_IN_MEMORY_INPUT_PATH_BYTES`), but the outer JSON envelope is
not. `run_json_bytes(mode, options_json, archive_bytes)` has the same shape
for `options_json`.

**Why not a finding:** The caller controls the input; `serde_json::from_str`
allocates predictably with no algorithmic blowup; requires the caller to opt
in. Out of scope per `SECURITY.md` ("theoretical attacks without a realistic
exploitation scenario").

**Recommended action (optional, future):** Add a soft cap on
`args_json.len()` (e.g. 8 MiB) returning a typed
`TokmdError::invalid_field("args", "JSON args exceed 8 MiB cap")`.

### OBS-002 (carried): Transitive `RUSTSEC-2020-0163` advisory

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (transitive) |
| **STRIDE Category** | Elevation of Privilege |
| **File** | `Cargo.lock` (transitive `term_size` via `tokei`) |
| **Status** | Documented in `deny.toml` |

**Description:** `term_size` is a transitive dependency of `tokei` and carries
an unmaintained advisory (`RUSTSEC-2020-0163`).

**Why not a finding:** Already documented in `deny.toml` with rationale. Out
of scope per `SECURITY.md` (third-party dependencies are reported upstream).

**Recommended action:** Track upstream `tokei` for a `term_size` removal.

### OBS-003 (carried): GitHub Actions pinning is mixed (tag + SHA)

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing / Tampering |
| **File** | `.github/workflows/*.yml` |
| **Status** | Not patched — mixed strategy |

**Description:** Pinning remains mixed. SHA-pinned (highest-privilege
surfaces):

```
EffortlessMetrics/droid-action-safe@7c1377ccbacddc95560d1570547a5baa51de01ec
EffortlessMetrics/ub-review@e1e41124e0468b3714827fd32574c8c583803b72
actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c
actions/github-script@f28e40c7f34bde8b3046d885e986cb6290c5673b
actions/setup-node@820762786026740c76f36085b0efc47a31fe5020
actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
cachix/install-nix-action@630ae543ea3a38a9a4166f03376c02c50f408342
docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a
docker/login-action@dbcb813823bdd20940b903addbd779551569679f
docker/setup-buildx-action@bb05f3f5519dd87d3ba754cc423b652a5edd6d2c
docker/setup-qemu-action@96fe6ef7f33517b61c61be40b68a1882f3264fb8
```

The composite Action's own `uses:` entries are also SHA-pinned
(`action.yml`):

```
actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
thollander/actions-comment-pull-request@24bffb9b452ba05a4f3f77933840a6a841d1b32b
```

Tag-pinned: `DeterminateSystems/*`, `EndBug/label-sync@v2`,
`Swatinem/rust-cache@v2`, `actions/attest-build-provenance@v4`,
`actions/cache@v6`, `actions/download-artifact@v8`, `actions/setup-node@v7`,
`actions/upload-artifact@v7`, `codecov/codecov-action@v7`,
`crate-ci/typos@v1`, `docker/build-push-action@v7`, `docker/login-action@v4`,
`docker/metadata-action@v6`, `docker/setup-buildx-action@v4`,
`docker/setup-qemu-action@v4`, `dtolnay/rust-toolchain@*`,
`peter-evans/create-pull-request@v8`, `softprops/action-gh-release@v3`,
`taiki-e/install-action@v2`, `thollander/actions-comment-pull-request@v3`.

**Why not a finding:** Tag-pinned first-party actions (`actions/*`) are a
well-accepted practice with low residual risk and are GitHub's own
recommended baseline. The highest-privilege third-party surface (the custom
Droid action) **is** SHA-pinned. Below the `medium` threshold.

**Recommended action (optional, future):** Either update the threat model to
reflect the actual mixed-pinning policy, or convert all third-party actions to
SHA-pinned references and codify the rotation process in `.factory/rules/`.
Note that the threat model's standing-defense #7 currently claims SHA pinning
workspace-wide, which is not strictly accurate.

### OBS-004 (carried): `web/runner` browser code does not pin GitHub API base URL

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing |
| **File** | `web/runner/ingest.js` |
| **Status** | Not patched — review for future |

**Description:** The browser-side runner fetches repository content via
`fetch()` to `api.github.com`. There is no Subresource Integrity pinning or
origin allow-listing on the client-side fetch surface. The token, when
supplied, is stored in `sessionStorage` (not `localStorage`) and used as a
`Bearer` header.

**Why not a finding:** All sensitive fetches target `https://api.github.com`,
which is HTTPS and well-known. Token lifetime is bounded to a single browser
tab. No DOM injection surfaces observed: all dynamic data is rendered via
`textContent` (re-verified this scan — a repository-wide grep for `innerHTML`,
`eval(`, `new Function`, `document.write`, and `outerHTML` across `web/`
returns no matches outside test files). The browser runner runs entirely in
the user-agent sandbox with no filesystem and no subprocess access.

**Recommended action (optional):** Consider an explicit allowlist of fetch
origins and a CSP `connect-src` directive in the runner's served HTML.

### OBS-005 (carried): `action.yml` install step performs `curl | sh` style download

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Tampering / Information Disclosure |
| **File** | `action.yml` (composite step `Install tokmd`) |
| **Status** | **Resolved this scan** — version input now format-validated (see VULN-001) |

**Description:** The composite GitHub Action downloads a pre-built `tokmd`
binary from `github.com/EffortlessMetrics/tokmd/releases/...` and verifies it
against `checksums.txt` (sha256). It does not verify a cryptographic signature
on the checksum file or on the release itself.

**Why not a finding:** The action is a published action; consumers control
which version they pin to. `curl -fsSL` rejects HTTP errors and follows
redirects. Checksum verification, when `checksums.txt` is present, uses
`sha256sum`/`shasum`/`Get-FileHash`. Build provenance is separately attested
via `actions/attest-build-provenance` in `release.yml`. Below the `medium`
threshold.

**Partial remediation applied:** The previously recommended explicit format
validation for the `version` input is now implemented as part of the VULN-001
fix. The remaining recommendation (signature verification of the checksum
file) is unchanged and out of scope for this scan.

---

## Standing Defenses Verified (No Regression)

All defenses from the 2026-07-27 scan were re-verified against the current
tree.

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `todo`, `unimplemented` lints denied | `Cargo.toml` | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation | `crates/tokmd-core/src/ffi/inputs.rs` (`MAX_IN_MEMORY_INPUT_PATH_BYTES = 4096`) | ✓ |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs` | ✓ |
| D-08 | Per-family schema versioning | `crates/tokmd-types/src/` | ✓ |
| D-09 | SHA-pinned Droid-related actions; tag-pinned first-party actions | `.github/workflows/droid*.yml` | ✓ (see OBS-003) |
| D-10 | Branch protection policy settings checked in | `.github/settings.yml` | ✓ (settings file only; live state is control-plane data) |
| D-11 | `cargo-deny` advisory + license allowlist | `deny.toml` | ✓ |
| D-12 | BLAKE3 redaction with extension allowlist | `crates/tokmd-format/src/redact/` | ✓ |
| D-13 | Content reads bounded by `ContentLimits` (default 128 KiB per file) | `crates/tokmd-analysis/src/content/mod.rs` | ✓ |
| D-14 | PyO3 FFI invariants (no panic, GIL release, error translation) | `crates/tokmd-python/src/lib.rs` | ✓ |
| D-15 | WASM uses `MemFs` (no host fs) | `crates/tokmd-wasm/` | ✓ |
| D-16 | `web/runner` uses `textContent` (no `innerHTML`/`eval`/`new Function`/`document.write`) | `web/runner/*.js` | ✓ (re-verified) |
| D-17 | `web/runner` token stored in `sessionStorage` (not `localStorage`) | `web/runner/auth.js` | ✓ |
| D-18 | `web/runner` worker protocol allowlists modes & presets | `web/runner/messages.js` | ✓ |
| D-19 | Composite action installs tokmd with sha256 checksum verification | `action.yml` | ✓ |
| D-20 | Custom Droid action SHA-pinned across all Droid workflows | `.github/workflows/droid*.yml` | ✓ |
| D-21 | `cargo audit` invoked with structured `--json` output, malformed JSON treated as Pending | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-22 | `run_json` top-level JSON must be an object | `crates/tokmd-core/src/ffi/mod.rs` | ✓ |
| D-23 | History-preserving import topology (two-parent publication merges) | repository topology | ✓ |
| **D-24** | **Composite action input isolation (new)** — no `${{ inputs.* }}` interpolated into a `run:` body | `action.yml` | ✓ (established by VULN-001 fix) |
| **D-25** | **Fail-closed archive admission with entry-count pre-guard** | `crates/tokmd-io-port/src/archive.rs` | ✓ (established by VULN-002 fix) |

### Additional verifications performed this scan

| Area | Result |
|------|--------|
| No `sh -c` / `bash -c` shell string execution anywhere in `crates/` | ✓ Only `LanguageType::from_name("Bash")` string matching and `completions bash` argv |
| All `Command::new` sites use `.arg()`/`.args()` with fixed arrays, never shell interpolation | ✓ |
| `cargo` subprocess calls (`cargo audit --json`, `cargo semver-checks`) use fixed argument arrays with no user interpolation | ✓ `crates/tokmd-cockpit/src/{supply_chain.rs,gates/contracts.rs}` |
| `crates/tokmd-core/src/context_git/mod.rs` `Command::new("git")` sites are test-only | ✓ All 11 sites are inside `#[cfg(test)]`; production code routes through `tokmd_git::repo_root`/`collect_history` (env-isolated) |
| No `unsafe` blocks in production code | ✓ All matches are in test files or string literals inside test fixtures |
| No `unwrap()`/`panic!()` in production (non-test) code paths | ✓ All matches are in `#[cfg(test)]` modules or doc comments |
| `action.yml` `output-dir` rejects absolute paths and `..` segments | ✓ Existing `case` guards retained |
| Archive admission rejects absolute/drive/UNC paths, `..`, NUL, symlinks, case-insensitive duplicates | ✓ Covered by existing unit tests in `crates/tokmd-io-port/src/archive.rs` |
| ZIP codec bounded-inflation (`read_ceiling = max_entry_size + 1`) | ✓ Verified |
| `ArchiveLimits` compression-ratio guard (100x default) | ✓ Verified; zero-compressed-size non-empty entries rejected |

---

## Appendix

### Threat Model

- **Status:** Updated (not regenerated)
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Reviewed:** 2026-11-09 (was 2026-08-02, 68 days old — within the
  90-day freshness window, so no regeneration was required)
- **Methodology:** STRIDE
- **Next review:** 2027-02-07 (90-day cadence) or upon architecture change
- **Changes this scan:** recorded the two new standing defenses
  (D-24 composite action input isolation, D-25 fail-closed archive admission)
  and refreshed the `Last Reviewed` date.
- **Inaccuracies noted:** standing-defense #7 claims SHA pinning workspace-wide;
  the actual policy is mixed (see OBS-003). The threat model should be
  reconciled at the next regeneration.

### Scan Metadata

- **Commits Scanned:** 0 in the literal 7-day window (the publication repo has
  a single squashed root commit `ff05889` dated 2026-08-05). Coverage was
  provided by a **full-tree scan** of the 2,589-file current tree against the
  standing threat model.
- **Files in scope:** 2,589 (1,245 Rust source files across 18 crates)
- **Scan Duration:** ~48m
- **Skills Used:** threat-model-generation (not invoked — existing model was
  current), commit-security-scan, vulnerability-validation, security-review
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **False Positive Filter:** applied — see Observations above. Two candidate
  findings were investigated and downgraded after measurement (the ZIP
  central-directory issue, documented as OBS-006; and the FFI payload size,
  carried as OBS-001).
- **Note on tooling:** the scheduled scan's subagent delegation backend was
  unavailable during this run ("Could not reach the daemon"); the analysis was
  therefore performed directly against the source tree rather than delegated.

### Scan Coverage Matrix

| Area | Files reviewed | Findings |
|------|----------------|----------|
| Composite GitHub Action (script injection surface) | `action.yml` (956 lines, 11 steps) | **1 (VULN-001)** |
| Archive ingestion (zip-bomb / path safety) | `crates/tokmd-io-port/src/archive.rs`, `crates/tokmd-scan/src/in_memory.rs`, `crates/tokmd-core/src/ffi/byte_mode.rs` | 1 (VULN-002) |
| Subprocess invocation | `crates/tokmd-git/`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd-cockpit/src/gates/contracts.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs`, `crates/tokmd-core/src/context_git/` | 0 |
| Path handling | `crates/tokmd-scan/src/path/bounded_path.rs`, `crates/tokmd-scan/src/roots.rs`, `crates/tokmd-scan/src/walk/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/` (`mod.rs`, `inputs.rs`, `parse.rs`, `byte_mode.rs`, `envelope.rs`, `modes.rs`, `settings_parse.rs`) | 0 |
| Language bindings | `crates/tokmd-python/`, `crates/tokmd-node/`, `crates/tokmd-wasm/` | 0 |
| File content reads | `crates/tokmd-analysis/src/content/`, `crates/tokmd-io-port/src/lib.rs` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/` | 0 |
| GitHub workflows (28 files) | `.github/workflows/*.yml`, `.github/settings.yml`, `.github/CODEOWNERS` | 0 |
| Build / lint / policy | `Cargo.toml`, `Cargo.lock`, `deny.toml`, `clippy.toml`, `.cargo/config.toml` | 0 |
| Web runner (browser) | `web/runner/*.js` | 0 |
| Gates / policy engine | `crates/tokmd-gate/` | 0 |
| Cockpit / evidence packets | `crates/tokmd-cockpit/` | 0 |
| Threat model | `.factory/threat-model/threat-model.md` | updated (2 defenses added) |

### Validation Performed

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features` | **21,567 passed, 0 failed** |
| `cargo xtask lint-fix` (fmt + clippy `-D warnings`) | **passed** (an unrelated clippy autofix in `crates/tokmd-cockpit/src/render/evidence.rs` was reverted to keep the diff scoped) |
| `cargo test -p tokmd-io-port --features archive-zip` | 48 passed, 0 failed |
| `npm test` (web runner) | 69 passed, 0 failed, 1 skipped (wasm bundle not built) |
| `action.yml` YAML parse + `bash -n` on extracted step body | passed |
| Injection proof: real extracted step script, malicious `version` | rejected, no command executed |
| Functional proof: real extracted step script, `version=1.14.0` and `latest` | download + checksum verify + install succeeded |
| Regression tests added | `rejects_archive_whose_entry_count_exceeds_the_cap_before_decoding`, `admits_archive_whose_entry_count_is_within_the_cap` |

### Patches Generated

| Finding | File | Commit |
|---------|------|--------|
| VULN-001 | `action.yml` | `f0c6b5c` — `fix(security): [VULN-001] Block shell injection via action version input` |
| VULN-002 | `crates/tokmd-io-port/src/archive.rs` | `a3a3134` — `fix(security): [VULN-002] Enforce archive entry-count cap before decoding` |

### Next Scan

The next scheduled security scan runs Monday, 2026-11-16 via
`.github/workflows/droid-security-scan.yml` (cron `0 8 * * 1`).

## References

- [CWE Database](https://cwe.mitre.org/)
- [CWE-94: Improper Control of Generation of Code](https://cwe.mitre.org/data/definitions/94.html)
- [CWE-78: Improper Neutralization of Special Elements used in an OS Command](https://cwe.mitre.org/data/definitions/78.html)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [GitHub Actions: Script injection guidance](https://securitylab.github.com/research/github-actions-preventing-pwn-requests/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Previous scans: `.factory/security/reports/security-report-2026-06-01.md`,
  `security-report-2026-06-08.md`, `security-report-2026-06-29.md`,
  `security-report-2026-07-06.md`, `security-report-2026-07-13.md`,
  `security-report-2026-07-20.md`, `security-report-2026-07-27.md`
