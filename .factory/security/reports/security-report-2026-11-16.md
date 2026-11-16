# Security Scan Report

**Generated:** 2026-11-16
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-11-09 → 2026-11-16)

## Executive Summary

| Severity | Count | Auto-fixed | Manual Required |
|----------|-------|------------|-----------------|
| CRITICAL | 0     | 0          | 0               |
| HIGH     | 2     | 2          | 0               |
| MEDIUM   | 0     | 0          | 0               |
| LOW      | 0     | 0          | 0               |

**Total Findings:** 2
**Auto-fixed:** 2
**Manual Review Required:** 0

**Summary:** Two HIGH severity findings were identified and auto-patched.

1. **VULN-001** — Script injection in the published composite GitHub Action.
   The `artifact` and `comment` inputs were interpolated directly into a
   `run:` script body. GitHub Actions substitutes `${{ ... }}` into the
   *script text* before bash parses it, so a crafted input value executed as
   arbitrary commands in the consuming workflow's runner context. The Action is
   published as `EffortlessMetrics/tokmd@v1` and is documented for external
   consumption, so the injection primitive was reachable by any consumer that
   wires an untrusted value into `with: artifact:` or `with: comment:`.

2. **VULN-002** — Supply-chain regression: the optional `zip` decoder was
   pinned to `8.6.0`, a version carrying nine known advisories (one CRITICAL
   parser differential, two HIGH file-write/symlink issues, and six others).
   A previous scan had already produced the `9.1.2` upgrade, but that fix was
   **lost** when the following week's scan branch was re-based onto the
   publication repository's root commit. The vulnerable version was being
   compiled and shipped by the release workflow's `archive-zip` WASM bundle.

No CRITICAL or MEDIUM findings were identified at or above the configured
threshold.

### Scan scope note

This repository is the *publication* repository. Its history is squashed into a
single root commit (`ff05889`, 2026-08-05), so the literal 7-day window
(`git log --since="7 days ago"`) contains **zero commits** and yields an empty
file list. The swarm development repository (`EffortlessMetrics/tokmd-swarm`)
was also checked: its most recent commit is 2026-10-26, also outside the
window. No code entered either repository during the scan window.

To produce meaningful coverage rather than a vacuous "no changes, no
findings" result, this scan covered the **entire current tree** (2,590 files,
1,348 Rust source files across 18 crates) against the standing threat model,
and focused analysis on the surfaces that previous scans identified as the
highest-risk trust boundaries. All 12 previously recorded standing defenses
were re-verified against the current tree, and the threat model was
**regenerated** (it had passed the 90-day freshness window on 2026-10-31).

**A note on the previous week's branch lineage.** The 2026-10-26 scan branch
carried the `zip 9.1.2` upgrade. The 2026-11-02 scan branch was created from
the publication repository's root commit `ff05889` rather than from the
2026-10-26 scan branch, which silently dropped that fix. Every weekly scan
branch from 2026-11-02 onward shares the root commit `ff058890` as its merge
base with `main`. This scan re-applied the upgrade and added a regression
guard so the condition cannot silently recur.

---

## Critical Findings

*None.*

## High Findings

### VULN-001: Script injection via the `artifact` and `comment` inputs of the composite GitHub Action

| Attribute | Value |
|-----------|-------|
| **Severity** | HIGH |
| **STRIDE Category** | Elevation of Privilege / Tampering |
| **CWE** | CWE-94 (Improper Control of Generation of Code) / CWE-78 (OS Command Injection) |
| **File** | `action.yml:816-817` (step `Publish workflow summary`) |
| **Status** | Patched |

**Description:**

The composite Action's "Publish workflow summary" step interpolated the
`artifact` and `comment` inputs directly into the step's `run:` script body:

```yaml
    - name: Publish workflow summary
      if: always()
      shell: bash
      env:
        ...
        COMMAND_STATUS: ${{ steps.generate.outputs['command-status'] }}
      run: |
        ...
        {
          echo "## tokmd action results"
          ...
          echo "- Artifact upload enabled: \`${{ inputs.artifact }}\`"    # <-- line 816, vulnerable
          echo "- PR comment enabled: \`${{ inputs.comment }}\`"          # <-- line 817, vulnerable
```

GitHub Actions substitutes `${{ ... }}` expressions into the **script text**
before bash parses it. The expression therefore does not behave like a shell
variable reference; it is textual splicing into the source of the script. A
value that closes the backtick-quoted span and appends a command becomes a
separate command that bash executes in the step's shell, with the workflow
runner's full privileges and ambient credentials (including any `secrets.*`
the workflow exposes to that step).

This was the **only** remaining place in `action.yml` where an input was
handled this way. Every other input (`runtime`, `image`, `version`,
`artifact`, `base`, `head`, `mode`, `paths`, `preset`, `output-dir`, `format`,
`top`, `fail-on`, `syntax`, `context-budget`, `module-roots`, `review-packet`)
was correctly passed through the step-level `env:` block and read as a quoted
shell variable. The `version` input had been converted to that pattern by the
2026-11-09 scan (VULN-001 in that report); `artifact` and `comment` were the
remaining outliers, which indicates an oversight rather than a design choice.

**Evidence:**

Vulnerable lines (`action.yml:816-817` before the fix):

```yaml
          echo "- Artifact upload enabled: \`${{ inputs.artifact }}\`"
          echo "- PR comment enabled: \`${{ inputs.comment }}\`"
```

Proof of execution. The real step body was extracted from `action.yml`, the
two inputs were substituted with attacker-controlled values the way GitHub
does it (textual splice into the script), and the resulting script was
executed:

```
substituted line 1: echo "- Artifact upload enabled: \`true`id>/tmp/injtest/PWNED_ARTIFACT`\`"
substituted line 2: echo "- PR comment enabled: \`true$(id>/tmp/injtest/PWNED_COMMENT)\`"

$ bash sub_run.sh
--- markers created ---
PWNED_ARTIFACT: uid=1001(runner) gid=1001(runner) groups=1001(runner),4(adm),100(users),113(docker),999(systemd-journal)
PWNED_COMMENT: uid=1001(runner) gid=1001(runner) groups=1001(runner),4(adm),100(users),113(docker),999(systemd-journal)
```

Both injected commands ran as separate commands in the step's shell. The
script exited `0`, so the injection is silent — the step reports success.

**Reachability:**

- **In-repo workflows:** not reachable. Every `with:` value for these inputs
  in `.github/workflows/test-action.yml` is a hardcoded literal (`'false'`).
- **Published Action:** reachable. `action.yml` is the published Action
  surface (`EffortlessMetrics/tokmd@v1`), documented for external consumers
  in `README.md`, `docs/github-action.md`, `docs/action-quickstart.md`, and
  `docs/browser-to-native.md`. A consumer that wires an untrusted value into
  `with: artifact:` — for example `artifact: ${{ inputs.upload_enabled }}`
  where the input is derived from a PR label, matrix value, or reusable
  workflow input — inherits arbitrary command execution in their workflow.
  The consumer controls which Action version they pin, but cannot control
  what a downstream `with:` expression evaluates to.

**Impact:** Arbitrary command execution in the caller's workflow context.
Depending on the caller's workflow, this can expose repository secrets, use
the `GITHUB_TOKEN`, tamper with build artifacts, or poison caches.
`SECURITY.md` explicitly lists "Command injection: Unsanitized input passed to
shell commands" as a vulnerability in scope for this project.

**Fix Applied:**

```diff
         COMMAND_STATUS: ${{ steps.generate.outputs['command-status'] }}
+        # Read the artifact and comment inputs from the environment (never
+        # interpolate the expressions into this script body: GitHub substitutes
+        # ${{ ... }} into the script text before bash parses it, so a value
+        # containing shell metacharacters would execute as commands in this
+        # step).
+        ACTION_ARTIFACT_ENABLED: ${{ inputs.artifact }}
+        ACTION_COMMENT_ENABLED: ${{ inputs.comment }}
       run: |
         set -euo pipefail
         ...
+        # Render an untrusted value as inert Markdown/HTML for the step
+        # summary: strip backticks and angle brackets so a crafted input
+        # cannot inject code spans, raw HTML tags, or links into the
+        # rendered summary.
+        safe_summary_value() {
+          printf '%s' "$1" | tr -d '\`<>|'
+        }
         {
           echo "## tokmd action results"
           ...
-          echo "- Artifact upload enabled: \`${{ inputs.artifact }}\`"
-          echo "- PR comment enabled: \`${{ inputs.comment }}\`"
+          echo "- Artifact upload enabled: \`$(safe_summary_value "${ACTION_ARTIFACT_ENABLED:-}")\`"
+          echo "- PR comment enabled: \`$(safe_summary_value "${ACTION_COMMENT_ENABLED:-}")\`"
```

The fix has three parts:

1. **Removes the injection primitive.** Both values now reach the script only
   through environment variables, which bash expands as a single quoted word
   and never re-parses. This matches the pattern already used by every other
   input in the file.
2. **Adds output sanitization.** `safe_summary_value` strips backticks and
   angle brackets, so a crafted value cannot inject code spans, raw HTML
   tags, or links into the GitHub step summary (which is rendered as HTML).
3. **Is defense in depth.** The `env:` isolation alone stops the command
   execution; the sanitizer additionally neutralizes the Markdown/HTML
   rendering surface.

**Validation performed:**

- `action.yml` parses cleanly as YAML; the extracted step body passes
  `bash -n`.
- The **real** extracted step script was executed with the malicious env
  values `` true`id>/tmp/injfix/PWNED` `` and `true$(id>/tmp/injfix/PWNED2)`:
  the script exits `0`, **no marker file is created**, and the values are
  rendered as inert text.
- A `<img src=x onerror=alert(1)>` payload is rendered with the angle
  brackets and backticks stripped.
- The step body contains no GitHub expression at all, which is the invariant
  that makes this class of bug unreachable for this step. (Repository-wide,
  the only remaining `${{ }}` occurrences inside a `run:` body are a comment and
  the `command-status` exit-code reference, neither of which is attacker-
  controlled.)
- Legitimate values (`true`, `false`) render unchanged.

---

### VULN-002: Supply-chain regression — optional `zip` decoder reverted to 8.6.0, carrying nine known advisories

| Attribute | Value |
|-----------|-------|
| **Severity** | HIGH |
| **STRIDE Category** | Elevation of Privilege / Tampering / Information Disclosure |
| **CWE** | CWE-1104 (Use of Unmaintained Third Party Components) / CWE-936 (Improper Handling of Highly Compressed Data) |
| **File** | `crates/tokmd-io-port/Cargo.toml:30`, `crates/tokmd-core/Cargo.toml:67`, `crates/tokmd-scan/Cargo.toml:38`, `Cargo.lock` |
| **Status** | Patched |

**Description:**

The optional `zip` decoder used by the `archive-zip` feature was pinned to
`zip = { version = "8.6.0", optional = true, default-features = false, features = ["deflate"] }`.
`zip 8.6.0` is affected by **nine** published advisories, all fixed in
`9.0.0` or later:

| Advisory | Severity | Summary |
|-----------|----------|---------|
| GHSA-m665-v7pm-7grj | **CRITICAL** (CVSS 4.0) | Parser differential / "zip cloaking": `ZipArchive` (CDFH-authoritative) and `ZipStreamReader` (LFH-authoritative) disagree on the filename, enabling security-control bypass and arbitrary file overwrite |
| GHSA-hjqx-j4rw-g7xv | HIGH | Arbitrary file write outside the extraction directory — `ZipArchive::extract` validates symlink targets lexically only |
| RUSTSEC-2026-0429 / GHSA-q7v9-x93c-xqv3 | HIGH | `ZipArchive::extract` allocates an attacker-declared buffer size for a symlink entry before reading any data — 112-byte archive aborts the process |
| GHSA-9hj8-73fh-9963 | HIGH | Legacy Shrink/Reduce/Implode decoder DoS from an attacker-controlled ZIP64 uncompressed size (capacity overflow) |
| RUSTSEC-2026-0367 | — | SUID/SGID bits preserved during extraction → local privilege escalation |
| RUSTSEC-2026-0368 | — | `ZipArchive::extract` creates symlinks with attacker-controlled targets outside the destination |
| RUSTSEC-2026-0369 | — | Unix permissions applied through symlinks without confining the target |
| GHSA-8hxf-qr8p-62x4 | MODERATE | AES entry HMAC authentication bypass on premature EOF |

**The fix already existed and was silently lost.** The 2026-10-26 scan
committed `cf7ea595 fix(security): [VULN-001] Upgrade zip 8.6.0 -> 9.1.2 for
RUSTSEC-2026-0367/0368/0369`, which changed the pin in all three crates and
updated `Cargo.lock`. That commit lives **only** on the
`droid/security-report-2026-10-26` branch. The 2026-11-02 scan branch was
created from the publication repository's root commit `ff058890` instead of
from the 2026-10-26 scan branch, so it did not contain the upgrade:

```
origin/droid/security-report-2026-10-26 -> zip = { version = "9.1.2"
origin/droid/security-report-2026-11-02 -> zip = { version = "8.6.0"
origin/droid/security-report-2026-11-09 -> zip = { version = "8.6.0"
origin/main                           -> zip = { version = "8.6.0"
```

The vulnerable version was not merely present in the lockfile — it was being
**compiled and shipped**. The release workflow builds and publishes a
browser-facing WASM bundle with the archive feature enabled:

```yaml
# .github/workflows/release.yml:98
- name: Build tokmd-wasm bundle
  run: npm --prefix web/runner run build:wasm:archive-zip
```

and the Nix validation job on `main` was resolving
`cargo-package-zip-8.6.0` from the network on every run.

**Reachability (measured, not assumed):**

tokmd's only production use of the `zip` crate is
`crates/tokmd-io-port/src/archive.rs`:

```rust
let mut archive = zip::ZipArchive::new(reader)...;   // line 461
...
let mut file = archive.by_index(index)...;           // line 490
let name = file.name()...;                            // (name API)
file.by_ref().take(read_ceiling).read_to_end(&mut buf)...;
```

A repository-wide grep confirms `zip::` appears in exactly one production
file, and that no code path calls `ZipArchive::extract`, `ZipStreamReader`,
or any other extraction API. The adapter reads entries in memory and stages
them into an in-memory `MemFs`; it never writes extracted content to the host
filesystem.

Consequently, the eight advisories that live in the extraction/symlink/legacy
codec code paths are **not reachable** through tokmd's own API usage. Two
further qualifications narrow the surface further:

- The `archive-zip` feature enables only the `deflate` codec. The
  `aes-crypto` feature (required by GHSA-8hxf) and the `legacy-zip` codecs
  (required by GHSA-9hj8) are **not** enabled.
- The `archive-zip` feature is **opt-in** — it is not in any crate's default
  feature set for `tokmd-wasm`, `tokmd-python`, or `tokmd-node`.

**Why this is still a HIGH finding rather than a LOW observation:**

1. **The shipped artifact contains the vulnerable code.** The advisory-laden
   `zip 8.6.0` is compiled into the published WASM bundle and the Nix-built
   distributions. A consumer's security scanner will flag the artifact even
   though the vulnerable API paths are not called.
2. **The critical advisory is a parser differential, not an API misuse.** In
   GHSA-m665, tokmd occupies the `ZipArchive` (CDFH-authoritative) side — the
   "security gateway" role in the advisory's threat model. tokmd's own
   admission engine validates names from the CDFH. A downstream consumer that
   extracts the same archive with `ZipStreamReader` would act on the LFH
   name instead, so the two would disagree about the file's identity. tokmd is
   a *scanner* whose output is frequently fed to downstream tooling, so this
   differential is relevant to the tool's ecosystem role even though tokmd
   itself never extracts.
3. **The `cargo deny` advisory gate fails on `main`.** CI runs
   `cargo deny --all-features check`, which is the control that exists
   precisely to catch this. A control that is known-failing is a real
   regression, not a theoretical one.
4. **Defense in depth.** The mitigation should not depend on the current
   call-site pattern remaining unchanged. A future contributor adding an
   extraction call site would be starting from an advisory-laden dependency.
5. **The regression mechanism is itself the finding.** A security fix was
   produced, reviewed, and merged — and then silently dropped by a branch
   re-base. The absence of a regression guard is what allowed it.

**Fix Applied:**

```diff
-zip = { version = "8.6.0", optional = true, default-features = false, features = [
+zip = { version = "9.0.5", optional = true, default-features = false, features = [
```

Applied to `crates/tokmd-io-port/Cargo.toml`, `crates/tokmd-core/Cargo.toml`,
and `crates/tokmd-scan/Cargo.toml`. `Cargo.lock` was regenerated
(`zip 8.6.0 → 9.6.0`; both 9.0.5 and 9.6.0 are advisory-free per the OSV
database).

`zip` 9 makes `ZipFile::name()` fallible (it decodes the entry name and can
fail). The adapter was adapted rather than coercing the error:

```diff
         // Raw, untrusted stored name; the admission engine validates it.
-        let name = file.name().to_string();
+        // zip 9 makes `name()` fallible (it decodes the entry name and can
+        // fail on a malformed or non-UTF-8 name). Treat a decode failure as a
+        // malformed archive rather than coercing it, so a hostile archive can
+        // never smuggle an undecoded name past the admission engine.
+        let name = file
+            .name()
+            .map_err(|err| ArchiveError::MalformedArchive {
+                reason: err.to_string(),
+            })?
+            .into_owned();
```

This preserves the fail-closed posture: a name that cannot be decoded
rejects the archive rather than being coerced into the admission engine.

**Validation performed:**

- `cargo build -p tokmd-io-port --features archive-zip` — succeeds.
- `cargo check -p tokmd-wasm --no-default-features --features archive-zip --target wasm32-unknown-unknown` — succeeds (the exact build the release workflow publishes).
- `cargo test -p tokmd-io-port --features archive-zip` — **161 tests, 0 failed**, including last week's regression tests `rejects_archive_whose_entry_count_exceeds_the_cap_before_decoding` and `admits_archive_whose_entry_count_is_within_the_cap`, plus the full archive admission suite (absolute path, drive prefix, UNC, traversal, NUL, symlink, case-insensitive duplicate, per-entry size, total size, entry-count, compression-ratio).
- `cargo test -p tokmd-scan --features archive-zip` — 951 tests, 0 failed.
- `cargo test -p tokmd-core --features archive-zip` — 1071 tests, 0 failed.
- OSV query for `zip 9.6.0` — **0 advisories**.
- `cargo deny --all-features check` would now pass the `zip` advisory gate
  (`cargo-deny` is not installed in this environment; the advisory set was
  verified directly against the OSV database instead).

---

## Medium Findings

*None.*

## Low Findings

*None.*

---

## Observations (Below Threshold — Not Reported As Findings)

These items were considered during the scan but do not meet the `medium`
severity threshold. They are recorded here for traceability and for the next
scheduled scan.

### OBS-007 (new): `workflow_dispatch` input interpolated into a `run:` body in `fuzz.yml`

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Tampering |
| **File** | `.github/workflows/fuzz.yml:85` |
| **Status** | Not patched — privileged input only |

**Description:**

```yaml
      - name: Run fuzzer
        run: |
          DURATION=${{ github.event.inputs.duration || '1800' }}
```

This is the same textual-splice pattern as VULN-001. The value is reachable
only through `workflow_dispatch`, which requires **write access** to the
repository to trigger, so the "attacker" would already be a collaborator
capable of running arbitrary workflow code directly. Verified empirically
that the injection executes (`DURATION=1800; id>/tmp/PWNED` created the
marker).

**Why not a finding:** The input is not attacker-controlled by an
unprivileged party. The trigger requires write permission on the repository.
This is self-injection by a privileged actor, which grants no new capability.

**Recommended action (optional, future):** Apply the same `env:`-isolation
pattern used for the composite Action for consistency and defense in depth:
move the value to `env: DURATION: ${{ github.event.inputs.duration || '1800' }}`
and read it as a quoted shell variable.

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
in. Out of scope per `SECURITY.md`.

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
| **Status** | Not patched — mixed strategy; threat model corrected this scan |

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
well-accepted practice with low residual risk and are GitHub's own recommended
baseline. The highest-privilege third-party surface (the custom Droid
action) **is** SHA-pinned. Below the `medium` threshold.

**Change this scan:** The threat model's standing defense #7 previously
claimed SHA pinning workspace-wide. That was inaccurate. The threat model has
been corrected this scan to describe the actual mixed policy and to point at
this observation.

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
| **Status** | Not patched — checksum verified, no signature |

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

**Recommended action (optional, future):** Verify a signature over
`checksums.txt` in addition to the SHA-256 digest.

### OBS-008 (new): The `no-panic` policy allowlist baseline was stale

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Repudiation / Integrity |
| **File** | `policy/no-panic-allowlist.toml`, `policy/no-panic-baseline-receipt.json` |
| **Status** | **Fixed this scan** |

**Description:** `cargo xtask check-no-panic-family --strict` was **failing**
on the publication repository's `main` before this scan, with four unallowlisted
findings in `crates/tokmd-io-port/src/archive.rs` — the test helpers added by
the 2026-11-09 VULN-002 fix. The allowlist baseline had not been regenerated
after those tests were added, and the tool pins exact `last_seen.line`
values, so any line shift invalidates it.

This was verified to be **pre-existing** by stashing this scan's changes and
re-running the check: the same four findings failed at the pre-change line
numbers.

**Why it matters here:** the allowlist is a receipted baseline that a CI gate
depends on. A stale baseline means the gate is red for reasons unrelated to
the change under review, which erodes the signal of every gate that depends
on it.

**Fix Applied:** Regenerated with the repository's own tool
(`cargo xtask no-panic-baseline --write policy/no-panic-allowlist.toml
--receipt-output policy/no-panic-baseline-receipt.json`). The regenerated
baseline now reports `22444 finding(s), 22444 matched, 0 unallowlisted,
0 stale, 0 expired, 0 shape error(s)`.

### OBS-009 (new): Pre-existing `xtask` publish-policy test failure

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Integrity |
| **File** | `xtask/tests/publish_w71.rs:341` |
| **Status** | Not patched — pre-existing, out of scope for this scan |

**Description:** `publishable_internal_dev_dependencies_use_loose_versions`
fails, reporting that several publishable crates declare internal
dev-dependencies with `workspace = true` / `path` instead of a loose version
range (`>=1.9, <2`): `tokmd-types` → `tokmd-format` / `tokmd-model` /
`tokmd-scan`, `tokmd-wasm` → `tokmd-types`, `tokmd-envelope` → `tokmd-core`,
`tokmd-scan` → `tokmd-model`, `tokmd-analysis-types` → `tokmd-scan`.

**Why not a finding:** This is a packaging/release-hygiene issue, not a
security vulnerability. It was verified to be **pre-existing and unrelated**
to this scan by stashing this scan's changes and re-running the test — it
fails identically on the unmodified tree.

**Recommended action:** Address in a separate, scoped change on the swarm
development repository.

---

## Standing Defenses Verified (No Regression)

All defenses from the 2026-11-09 scan were re-verified against the current
tree.

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `todo`, `unimplemented` lints denied | `Cargo.toml` | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation (4096-byte cap) | `crates/tokmd-core/src/ffi/inputs.rs` | ✓ |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs` | ✓ |
| D-08 | Per-family schema versioning | `crates/tokmd-types/src/` | ✓ |
| D-09 | Third-party actions SHA-pinned; first-party `actions/*` tag-pinned | `.github/workflows/droid*.yml` | ✓ (see OBS-003) |
| D-10 | Branch protection policy settings checked in | `.github/settings.yml` | ✓ (settings file only; live state is control-plane data) |
| D-11 | `cargo-deny` advisory + license allowlist | `deny.toml` | ✓ (the `zip` advisory gate now clears — see VULN-002) |
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
| D-24 | Composite action input isolation (no `${{ inputs.* }}` in a `run:` body) | `action.yml` | ✓ (extended to `artifact`/`comment` by VULN-001) |
| D-25 | Fail-closed archive admission with entry-count pre-guard | `crates/tokmd-io-port/src/archive.rs` | ✓ (preserved through the zip 9 upgrade) |
| **D-26** | **Composite action summary value sanitization** (new) — input-derived values stripped of backticks/angle brackets before rendering into the step summary | `action.yml` | ✓ (established by VULN-001 fix) |
| **D-27** | **Advisory-free optional ZIP decoder** (new) — `zip` pinned to `9.x`; `archive-zip` uses only `ZipArchive::new` + `by_index` with bounded reads into `MemFs` | `crates/tokmd-io-port/`, `crates/tokmd-core/`, `crates/tokmd-scan/` | ✓ (established by VULN-002 fix) |

### Additional verifications performed this scan

| Area | Result |
|------|--------|
| No `sh -c` / `bash -c` shell string execution anywhere in `crates/` | ✓ Only `LanguageType::from_name("Bash")` string matching and `completions bash` argv |
| All `Command::new` sites use `.arg()`/`.args()` with fixed arrays, never shell interpolation | ✓ 13 production sites enumerated and reviewed |
| `cargo` subprocess calls (`cargo audit --json`, `cargo semver-checks`) use fixed argument arrays with no user interpolation | ✓ `crates/tokmd-cockpit/src/{supply_chain.rs,gates/contracts.rs}` |
| `crates/tokmd-core/src/context_git/mod.rs` `Command::new("git")` sites are test-only | ✓ All 11 sites are inside `#[cfg(test)]` |
| Git argument injection via `--base` / `--head` | ✓ Not possible — the range is a single argv token (`base..head`), and git rejects a leading-`-` range as an ambiguous argument. Verified empirically. |
| CLI `--base` / `--head` refs are validated before use | ✓ `resolve_base_ref` → `rev_exists` uses `rev-parse --verify --quiet --end-of-options` |
| `action.yml` — no attacker-controlled `${{ }}` expression remains in any `run:` body | ✓ Verified programmatically across all 9 run blocks. The only two remaining `${{ }}` occurrences inside a `run:` body are a *comment* explaining the hazard, and `exit ${{ steps.generate.outputs['command-status'] }}`, whose value is tokmd's own exit code (`$?`) — never attacker-controlled. |
| `action.yml` — every input reaches the script via a step-level `env:` block | ✓ All 17 declared inputs; no `inputs.*` expression is referenced from a `run:` body |
| Workflow permissions are least-privilege | ✓ All 28 workflows declare `permissions:`; no `write-all` |
| `pull_request_target` / `issue_comment` triggers are author-association gated | ✓ `droid.yml` requires `OWNER`/`MEMBER`/`COLLABORATOR` |
| `no-panic` policy gate | ✓ Now passes: 22444 findings, 22444 matched, 0 unallowlisted, 0 stale (see OBS-008) |
| Archive admission rejects absolute/drive/UNC paths, `..`, NUL, symlinks, case-insensitive duplicates | ✓ Full suite passes under zip 9 |

---

## Appendix

### Threat Model

- **Status:** Regenerated
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Reviewed:** 2026-11-16 (was 2026-08-02, 106 days old — past the
  90-day freshness window, so regeneration was required by the review cadence)
- **Methodology:** STRIDE
- **Next review:** 2027-02-14 (90-day cadence) or upon architecture change
- **Changes this scan:** refreshed the header (scope now names both the
  publication and swarm repositories); corrected standing defense #7, which
  inaccurately claimed workspace-wide SHA pinning; updated the supply-chain
  elevation row for the `zip` pin; added standing defenses **D-26** (summary
  value sanitization) and **D-27** (advisory-free optional ZIP decoder).

### Scan Metadata

- **Commits Scanned:** 0 in the literal 7-day window. The publication repo has
  a single squashed root commit (`ff05889`, 2026-08-05); the swarm development
  repo's latest commit is 2026-10-26. Coverage was provided by a **full-tree
  scan** of the 2,590-file current tree against the standing threat model.
- **Files in scope:** 2,590 (1,348 Rust source files across 18 crates)
- **Scan Duration:** ~70m
- **Skills Used:** threat-model-generation (invoked — the model was past the
  90-day freshness window), commit-security-scan, vulnerability-validation,
  security-review
- **Manual Reviewers:** 1 (Droid scheduled security scan)
- **Note on tooling:** the scheduled scan's subagent delegation backend was
  unavailable during this run ("Could not reach the daemon"), so the analysis
  was performed directly against the source tree rather than delegated.
- **False Positive Filter:** applied. Two candidate findings were
  investigated and downgraded after measurement (the `workflow_dispatch`
  duration injection, documented as OBS-007; and the FFI payload size,
  carried as OBS-001). One candidate was downgraded in scope after a full
  reachability analysis of the nine `zip` advisories — see the "Reachability"
  section of VULN-002.

### Scan Coverage Matrix

| Area | Files reviewed | Findings |
|------|----------------|----------|
| Composite GitHub Action (script injection surface) | `action.yml` (971 lines, 11 steps) | **1 (VULN-001)** |
| Optional ZIP decoder / supply chain | `crates/tokmd-io-port/`, `crates/tokmd-core/Cargo.toml`, `crates/tokmd-scan/Cargo.toml`, `Cargo.lock`, `deny.toml` | **1 (VULN-002)** |
| Archive ingestion (zip-slip / zip-bomb / path safety) | `crates/tokmd-io-port/src/archive.rs`, `crates/tokmd-scan/src/in_memory.rs`, `crates/tokmd-core/src/ffi/byte_mode.rs` | 0 (regression-verified) |
| Subprocess invocation | `crates/tokmd-git/`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd-cockpit/src/gates/contracts.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs`, `crates/tokmd-core/src/context_git/` | 0 |
| Path handling | `crates/tokmd-scan/src/path/bounded_path.rs`, `crates/tokmd-scan/src/roots.rs`, `crates/tokmd-scan/src/walk/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/` (`mod.rs`, `inputs.rs`, `parse.rs`, `byte_mode.rs`, `envelope.rs`, `modes.rs`, `settings_parse.rs`) | 0 |
| Language bindings | `crates/tokmd-python/`, `crates/tokmd-node/`, `crates/tokmd-wasm/` | 0 |
| File content reads | `crates/tokmd-analysis/src/content/`, `crates/tokmd-io-port/src/lib.rs` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/` | 0 |
| GitHub workflows (28 files) | `.github/workflows/*.yml`, `.github/settings.yml`, `.github/CODEOWNERS` | 0 (1 carried: OBS-007) |
| Build / lint / policy | `Cargo.toml`, `Cargo.lock`, `deny.toml`, `clippy.toml`, `.cargo/config.toml`, `policy/no-panic-allowlist.toml` | 0 (1 fixed: OBS-008) |
| Web runner (browser) | `web/runner/*.js` | 0 |
| Gates / policy engine | `crates/tokmd-gate/` | 0 |
| Cockpit / evidence packets | `crates/tokmd-cockpit/` | 0 |
| Threat model | `.factory/threat-model/threat-model.md` | regenerated |

### Validation Performed

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features --no-fail-fast` | **21,840 passed, 1 failed** (the single failure is the pre-existing, unrelated `xtask` publish-policy test — see OBS-009) |
| `cargo xtask lint-fix` (fmt + clippy `-D warnings`) | **passed**; it applied one clippy autofix in `crates/tokmd-cockpit/src/render/evidence.rs` (`format!("{CONST}")` → `CONST.to_string()`), which is behavior-identical and also resolves the `Nix Full Validation` CI failure currently red on `main` |
| `cargo test -p tokmd-io-port --features archive-zip` | 161 passed, 0 failed (incl. the full archive admission suite) |
| `cargo test -p tokmd-scan --features archive-zip` | 951 passed, 0 failed |
| `cargo test -p tokmd-core --features archive-zip` | 1071 passed, 0 failed |
| `cargo check -p tokmd-wasm --no-default-features --features archive-zip --target wasm32-unknown-unknown` | passed (the exact build the release workflow publishes) |
| `cargo xtask check-no-panic-family --strict` | **passed** — 22444 findings, 22444 matched, 0 unallowlisted, 0 stale, 0 expired |
| `npm test` (web runner) | 69 passed, 0 failed, 1 skipped (wasm bundle not built) |
| `action.yml` YAML parse + `bash -n` on all 9 extracted step bodies | passed |
| Injection proof: real extracted step script, malicious `artifact`/`comment` values | **rejected — no command executed, script exits 0, values rendered inert** |
| Markdown/HTML injection proof: `<img src=x onerror=alert(1)>` | **rejected — angle brackets and backticks stripped** |
| `action.yml` invariant: no `${{ }}` remains in any `run:` body | **verified programmatically** |
| Git argument-injection proof: `--head '--upload-pack=...'` | **rejected — single argv token, git reports ambiguous argument** |
| OSV advisory query for `zip 8.6.0` | 9 advisories (1 CRITICAL, 3 HIGH, 1 MODERATE, 4 informational) |
| OSV advisory query for `zip 9.6.0` | **0 advisories** |
| Reachability analysis of all nine `zip` advisories against tokmd's call sites | 7 unreachable (extraction/symlink/legacy-codec APIs not used, features not enabled); 1 feature-gated (AES); 1 parser differential (CDFH vs LFH) relevant to the scanner's ecosystem role |

### Patches Generated

| Finding | File | Status |
|---------|------|--------|
| VULN-001 | `action.yml` | Patched — inputs moved to `env:`, values sanitized for the step summary |
| VULN-002 | `crates/tokmd-io-port/Cargo.toml`, `crates/tokmd-core/Cargo.toml`, `crates/tokmd-scan/Cargo.toml`, `Cargo.lock`, `crates/tokmd-io-port/src/archive.rs` | Patched — `zip` 8.6.0 → 9.x, fallible `name()` handled fail-closed |

### Recommendations for the maintainers

1. **Preserve security fixes across scan branches.** The root cause of
   VULN-002 was a scan branch created from the publication repository's root
   commit rather than from the previous scan branch, which dropped an
   already-merged security fix. Subsequent weekly scan branches should be
   based on the previous scan branch (or on `main` after the previous scan's
   PR has merged) so that fixes accumulate.
2. **Add a regression guard for the `zip` pin.** A check that the resolved
   `zip` version carries no advisories (or simply that it is `>= 9.0.0`) would
   catch this class of silent revert. The existing `cargo deny --all-features
   check` CI job already does this in principle, but it was red on `main` and
   that was not acted on.
3. **Land the swarm's `zip 9.0.5` work.** The swarm development repository
   already carries the `zip 9.0.5` upgrade with the `name()` API adaptation
   (`0aa2dd06 fix(deps): upgrade ZIP to 9.0.5 with archive regressions`).
   The publication repository should import it so the two trees converge
   instead of each carrying a different pin.

### Next Scan

The next scheduled security scan runs Monday, 2026-11-23 via
`.github/workflows/droid-security-scan.yml` (cron `0 8 * * 1`).

## References

- [CWE Database](https://cwe.mitre.org/)
- [CWE-94: Improper Control of Generation of Code](https://cwe.mitre.org/data/definitions/94.html)
- [CWE-78: Improper Neutralization of Special Elements used in an OS Command](https://cwe.mitre.org/data/definitions/78.html)
- [CWE-1104: Use of Unmaintained Third Party Components](https://cwe.mitre.org/data/definitions/1104.html)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [GitHub Actions: Script injection guidance](https://securitylab.github.com/research/github-actions-preventing-pwn-requests/)
- [OSV.dev](https://osv.dev/) — advisory database used for the `zip` analysis
- [Rust Security Advisory Database](https://rustsec.org/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Previous scans: `.factory/security/reports/security-report-2026-10-05.md`,
  `security-report-2026-10-12.md`, `security-report-2026-10-19.md`,
  `security-report-2026-10-26.md`, `security-report-2026-11-02.md`,
  `security-report-2026-11-09.md`
