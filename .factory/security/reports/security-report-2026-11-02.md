# Security Scan Report

**Generated:** 2026-11-02
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-10-26 → 2026-11-02)

## Executive Summary

| Severity | Count | Auto-fixed | Manual Required |
|----------|-------|------------|-----------------|
| CRITICAL | 0     | 0          | 0               |
| HIGH     | 0     | 0          | 0               |
| MEDIUM   | 3     | 3          | 0               |
| LOW      | 0     | 0          | 0               |

**Total Findings:** 3
**Auto-fixed:** 3
**Manual Review Required:** 0

**Summary:** Three MEDIUM findings, all reproduced against the built binaries and
all fixed on this branch.

VULN-003 and VULN-004 are the two halves of one defect class in the cockpit
review-packet contract: neither the writer nor the verifier established that a
packet artifact actually lives inside the packet directory. A pull request can
commit `.tokmd/review` as a symlink (Git stores it as mode `120000` and restores
it on checkout), and `.tokmd/` is **not** gitignored in this repository — only
`.tokmd/cache/` is. `action.yml` uses `review_packet_dir=".tokmd/review"` by
default, so the symlinked path is the one the composite Action exercises.
Reproduced end to end: `tokmd cockpit --review-packet-dir .tokmd/review` wrote
all six packet artifacts outside the requested directory and exited 0, and
`xtask review-packet-check` then certified those out-of-packet artifacts with
`"ok": true` and `hashes_verified: 5`.

VULN-005 is a separate defect: `xtask` passed `--base`/`--head` revisions to
`git diff` as positional arguments. Git parses a revision starting with `-` as
an option, so `--output=<path>` made `git diff` write to a caller-chosen path,
truncating an existing file.

The scan window contains **no commits on `main`** of the publication repository
(`git log origin/main --since="7 days ago"` returns nothing; `main` is unchanged
since the 2026-08-05 import). Development continues in
`EffortlessMetrics/tokmd-swarm`, whose `main` received three commits inside the
window, all on 2026-10-26. Those three commits and the two open, unmerged
in-window pull requests were reviewed; see *Scan Coverage* below. Two of the
findings were first reported by the 2026-10-26 swarm scan and remained
unremediated on `main`; VULN-005 is new to this scan.

## Critical Findings

*None.*

## High Findings

*None.*

## Medium Findings

### VULN-003: Review-packet verifier certifies artifacts reached via a directory symlink

| Attribute | Value |
|-----------|-------|
| **Severity** | MEDIUM |
| **STRIDE Category** | Tampering |
| **CWE** | CWE-59 (Improper Link Resolution Before File Access) |
| **File** | `xtask/src/tasks/review_packet_check.rs:181` (pre-fix) |
| **Status** | Patched — commit `2787b0ea` |

**Description:**

`validate_manifest_artifacts` checked each manifest artifact with
`fs::symlink_metadata` and accepted it when `metadata.file_type().is_file()`.
`symlink_metadata` does not follow a symlink at the **final** path component,
but it does resolve a symlinked **intermediate** directory. A manifest entry of
`proof/proof-run-summary.json`, where `proof` is a directory symlink to an
external directory, therefore passed the lexical path check (`..` and absolute
components were already rejected by `packet_relative_path`) and the
regular-file check, and the BLAKE3 hash was then computed over a file living
**outside** the packet directory.

**Evidence:**

Pre-fix source:

```rust
let artifact_path = dir.join(&relative_path);
match fs::symlink_metadata(&artifact_path) {
    Ok(metadata) if metadata.file_type().is_file() => {
        verify_artifact_hash(&artifact_path, &display_path, artifact, &mut errors);
    }
```

Reproduced against the built `xtask` binary, with `proof` a symlink to a
directory outside the packet:

```
$ ln -s /tmp/vuln003_poc/OUTSIDE /tmp/vuln003_poc/packet/proof
$ xtask review-packet-check --dir /tmp/vuln003_poc/packet --json report.json
Review packet OK: 6 artifact(s) in `/tmp/vuln003_poc/packet`
EXIT=0
```

The receipt claimed `"ok": true` and `"hashes_verified": 6`, including
`proof/proof-run-summary.json`, whose bytes are physically at
`/tmp/vuln003_poc/OUTSIDE/proof-run-summary.json`.

Negative control, same fixture with the symlink replaced by a real directory:

```
Error: review packet check failed:
- artifact `proof-run-summary` path `proof/proof-run-summary.json` is missing: No such file or directory (os error 2)
EXIT=1
```

**Impact:** The verifier receipt (`ok: true`, `hashes_verified: N`) certifies
evidence bytes that are not packet-local, defeating the manifest-hash contract
that the hosted PR comment reports as `Manifest hashes: valid`. A party that can
place a directory symlink inside the packet directory before verification
controls which bytes are certified.

**Fix Applied:**

```diff
+    // Resolve the packet root once. A symlinked packet root is supported; the
+    // resolved root is what every artifact must stay inside.
+    let packet_root = dir
+        .canonicalize()
+        .with_context(|| format!("resolve review packet directory {}", dir.display()))?;
...
-        let artifact_path = dir.join(&relative_path);
+        let artifact_path = packet_root.join(&relative_path);
         match fs::symlink_metadata(&artifact_path) {
             Ok(metadata) if metadata.file_type().is_file() => {
+                // A regular leaf can still be reached through a directory link.
+                // Resolve it before reading bytes to establish packet containment.
+                match artifact_path.canonicalize() {
+                    Ok(resolved) if resolved.starts_with(packet_root) => {}
+                    Ok(_) => {
+                        errors.push(format!(
+                            "artifact `{}` path `{display_path}` resolves outside packet directory",
+                            artifact.id
+                        ));
+                        continue;
+                    }
+                    Err(err) => { /* could not be resolved */ }
+                }
                 verify_artifact_hash(&artifact_path, &display_path, artifact, &mut errors);
             }
```

Regression tests added: `review_packet_rejects_proof_directory_symlink_escape`,
`review_packet_accepts_internal_proof_directory_symlink` (an internal directory
link must keep working), and `review_packet_accepts_symlinked_packet_root`.

Post-fix reproduction:

```
$ xtask review-packet-check --dir /tmp/vuln003_poc/packet
Error: review packet check failed:
- artifact `proof-run-summary` path `proof/proof-run-summary.json` resolves outside packet directory
EXIT=1
```

---

### VULN-004: Review-packet writer emits artifacts outside the requested packet directory

| Attribute | Value |
|-----------|-------|
| **Severity** | MEDIUM |
| **STRIDE Category** | Tampering |
| **CWE** | CWE-59 (Improper Link Resolution Before File Access), CWE-22 (Path Traversal) |
| **File** | `crates/tokmd-cockpit/src/render/review_packet.rs:65` (pre-fix) |
| **Status** | Patched — commit `2787b0ea` |

**Description:**

VULN-003 is the verifier side of the review packet. The writer side had no
containment check at all. `write_review_packet` created the packet directory and
wrote every artifact through it:

```rust
std::fs::create_dir_all(dir)?;
...
std::fs::write(dir.join("cockpit.json"), &cockpit_json)?;
...
std::fs::write(dir.join(&artifact.path), &artifact.content)?;
```

`create_dir_all` succeeds when the target already exists as a symlink to a
directory, and `fs::write` then follows that symlink. If the packet directory
path is itself a symlink — or contains a symlinked `proof` subdirectory — every
artifact is written **outside** the requested packet directory.

**Evidence:**

Reproduced end to end with the built `tokmd` binary, in a repository where
`.tokmd/review` is committed as a symlink:

```
$ git ls-files -s .tokmd
120000 61ae6dc686b6f217704a8659ae4ed268f5069c5f 0	.tokmd/review

$ tokmd cockpit --base HEAD~1 --head HEAD --review-packet-dir .tokmd/review --format json
exit=0

$ ls -la /tmp/wr_test/outside/
-rw-r--r-- 1 runner runner 1373 cockpit.json
-rw-r--r-- 1 runner runner  456 comment.md
-rw-r--r-- 1 runner runner 2442 evidence.json
-rw-r--r-- 1 runner runner 2420 manifest.json
-rw-r--r-- 1 runner runner 1628 review-map.json
-rw-r--r-- 1 runner runner  896 review-map.md
```

All six packet artifacts landed outside the requested directory, and the
command reported success. The escaped packet then passed the verifier:

```
$ xtask review-packet-check --dir .tokmd/review
Review packet OK: 5 artifact(s) in `.tokmd/review`
```

**Reachability:** `.tokmd/` is not gitignored in this repository — only
`.tokmd/cache/` is (`.gitignore:34`), and `git check-ignore .tokmd/review`
returns "not ignored". Git tracks a symlink as mode `120000` and restores it as
a symlink on checkout. A pull-request author can therefore commit
`.tokmd/review` as a symlink and the checkout presents it as a symlink to any
job that later runs `tokmd cockpit --review-packet-dir .tokmd/review`. That is
the path `action.yml` uses by default (`review_packet_dir=".tokmd/review"`).

**Impact:** Packet artifacts are written outside the directory the caller
requested. Combined with VULN-003, the produced packet and its verification
receipt both report success while referring to files outside the packet
directory.

**Fix Applied:**

```diff
-    std::fs::create_dir_all(dir)?;
+    let packet_root = prepare_packet_root(dir)?;
...
-    std::fs::write(dir.join("cockpit.json"), &cockpit_json)?;
+    write_packet_artifact(&packet_root, "cockpit.json", &cockpit_json)?;
```

`prepare_packet_root` refuses a symlinked packet root and returns the canonical
root. `write_packet_artifact` checks the relative path component-wise,
re-resolves the parent directory, and rejects any target whose resolved parent
lies outside the packet root, so a symlinked intermediate directory cannot
redirect a write.

Regression tests added: `rejects_symlinked_packet_root`,
`rejects_symlinked_intermediate_directory`,
`rejects_artifact_path_escaping_the_packet_root`,
`writes_artifact_inside_the_packet_root`, `creates_missing_packet_directory`.

Post-fix reproduction:

```
$ tokmd cockpit --base HEAD~1 --head HEAD --review-packet-dir .tokmd/review --format json
Error: review packet directory must not be a symlink: .tokmd/review
EXIT=1

$ ls -la /tmp/wr_test/outside/     # empty — nothing escaped
```

The non-symlinked path is unaffected: `tokmd cockpit --review-packet-dir
.tokmd/review` still exits 0 and writes all six artifacts inside the requested
directory.

---

### VULN-005: Git option injection through unvalidated revision arguments

| Attribute | Value |
|-----------|-------|
| **Severity** | MEDIUM |
| **STRIDE Category** | Tampering |
| **CWE** | CWE-88 (Improper Neutralization of Argument Delimiters in a Command) |
| **File** | `xtask/src/tasks/affected.rs:91` (pre-fix), `xtask/src/tasks/ci_plan.rs:528`, `xtask/src/tasks/mutation_scope.rs:85` |
| **Status** | Patched — commit `8ea34506` |

**Description:**

`changed_files`, `git_diff_names`, and `git_diff_rust_files` passed the
`--base`/`--head` revisions straight to `git diff` as positional arguments:

```rust
let output = Command::new("git")
    .args(["diff", "--name-only", base, head, "--"])
    .output()
```

Git parses a revision that begins with `-` as an **option**, not as a revision.
`git diff --output=<file>` writes the diff to that path instead of stdout, so a
revision value of `--output=<path>` made `git diff` write to a caller-chosen
path. The trailing `--` separator protects the *pathspec* side of the command
only; it does not stop a revision argument from being parsed as an option.

**Evidence:**

```
$ git diff --no-renames --name-only -z "--output=/tmp/gitinject/IMPORTANT.txt" HEAD --
$ cat /tmp/gitinject/IMPORTANT.txt
existing important content          # before
                                   # after: truncated to 0 bytes
```

The workspace already applies the correct discipline in the runtime code path:
`crates/tokmd-git/src/refs.rs::env_base_ref_is_safe` rejects empty values,
leading `-`, whitespace, control characters, and backslashes, and
`crates/tokmd-git/src/refs.rs` invokes `git rev-parse --verify --quiet
--end-of-options`. The `xtask` control-plane paths did not follow that pattern.

**Impact:** A revision value under the caller's control reaches `git diff` as an
option, allowing git to write to, and truncate, a file at a caller-chosen path.

**Reachability:** The CLI layer (clap) already rejects a leading-dash `--base`
value, so the injection is not reachable from an untrusted pull request
through the command line. It remains reachable for any other caller of the
`xtask` library functions, and the missing `--end-of-options` separator means
the invariant depends entirely on the argument parser. All in-repository CI
call sites are additionally safe: `ci.yml` and `proof-executor.yml` pass an
`origin/`-prefixed ref (an argument starting with `origin/` is never parsed as
an option), and `release-preflight.yml` validates both refs against
`^[0-9a-f]{40}$|^[0-9a-f]{64}$` before use. The finding is reported at MEDIUM
as a defense-in-depth gap in a control-plane tool, not as a directly
exploitable pull-request path.

**Fix Applied:**

```diff
+/// Reject a revision that Git would parse as an option.
+pub(crate) fn ensure_ref_is_not_option_like(label: &str, value: &str) -> Result<()> {
+    if value.starts_with('-') {
+        bail!("{label} revision must not look like a git option: {value:?}");
+    }
+    Ok(())
+}
+
 pub(crate) fn changed_files(base: &str, head: &str) -> Result<Vec<String>> {
+    ensure_ref_is_not_option_like("base", base)?;
+    ensure_ref_is_not_option_like("head", head)?;
+
     let output = Command::new("git")
-        .args(["diff", "--name-only", base, head, "--"])
+        .args([
+            "diff",
+            "--name-only",
+            "--end-of-options",
+            base,
+            head,
+            "--",
+        ])
```

The same guard and `--end-of-options` separator were added to
`xtask/src/tasks/ci_plan.rs::git_diff_names` and
`xtask/src/tasks/mutation_scope.rs::git_diff_rust_files`.

Regression tests added: `option_like_revisions_are_rejected_before_reaching_git`,
`ordinary_revisions_are_accepted`, and
`option_like_base_revision_does_not_write_through_git_diff` (which asserts the
marker file is not created).

## Low Findings

*None.*

## Observations (Below Threshold — Not Reported As Findings)

### OBS-001: `xtask affected` and `xtask proof` do not use NUL-framed diff output

The publication repository's `main` predates swarm commit `07d878be` (#771,
"fix(ci): retain renamed paths in affected selectors"), which moved
`xtask/src/tasks/affected.rs` and `xtask/src/tasks/ci_plan.rs` to NUL-framed
`git diff --name-only -z` parsing and stopped rewriting `\` to `/`. The
publication copy still splits on newlines and normalizes separators, so a Git
filename containing a newline is split into two paths and a literal `\`
is treated as a directory separator.

**Why not a finding:** This is a correctness and path-identity defect, not an
injection or containment defect: the affected-scope output selects which proof
scopes run, and the swarm fix already fails closed on unknown files
(`fail_on_unknown_non_rust`). It is a publication-lag gap, not a vulnerability
introduced by the in-window changes. The swarm fix (PR #771, merged into swarm
`main` on 2026-10-26) is the upstream remediation; it should reach the
publication repository through the normal swarm import path.

### OBS-002: FFI JSON payload size is not bounded

`run_json(mode, args_json)` accepts a JSON string of arbitrary size. Individual
in-memory `inputs[].path` is bounded to 4096 bytes, but the outer envelope is
not. **Carried forward unchanged** from the 2026-07-27 scan and the threat
model. Caller controls the input; no algorithmic blowup. Remains below the
`medium` threshold.

### OBS-003: Transitive `RUSTSEC-2020-0163` advisory

`term_size` is a transitive dependency of `tokei` and carries an unmaintained
advisory. Already documented in `deny.toml` with rationale; out of scope per
`SECURITY.md`. **Carried forward unchanged.**

### OBS-004: GitHub Actions pinning is mixed (tag + SHA)

Droid-related workflows pin third-party actions by SHA; other workflows pin
first-party actions by tag. The threat model claims SHA pinning workspace-wide,
which is no longer strictly accurate. Tag-pinned first-party actions are a
well-accepted practice with low residual risk, and the custom Droid action — the
highest-privilege third-party surface — **is** SHA-pinned. **Carried forward
unchanged**; flagged for the next threat-model refresh.

### OBS-005: `action.yml` interpolates the `version` input into a download URL

The composite Action builds a release download URL from a user-supplied `version`
input without an explicit format check, and verifies the downloaded binary
against `checksums.txt` (sha256) without a signature on the checksum file. The
value is bounded to a `MAJOR.MINOR.PATCH`-style string by the `${ver#v}` prefix
logic, and `curl -fsSL` rejects HTTP errors. Build provenance is separately
attested in `release.yml`. **Carried forward unchanged.**

### OBS-006: `tokmd cockpit --review-packet-dir` is a user-facing CLI surface

The containment rules added by this scan live in the library, so every caller —
including library consumers embedding `tokmd-core` — gets them. Consumers that
point `--review-packet-dir` at a directory they do not control are unaffected;
consumers that point it at a directory an untrusted party can write to should
be aware the writer now **fails closed** on a symlinked root rather than
silently writing elsewhere. This is the intended behavior change.

## Standing Defenses Verified (No Regression)

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `todo`, `unimplemented` denied | `Cargo.toml` | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs` | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation (4096-byte cap) | `crates/tokmd-core/src/ffi/inputs.rs` | ✓ |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs` | ✓ |
| D-08 | Per-family schema versioning | `crates/tokmd-types/src/` | ✓ |
| D-09 | SHA-pinned Droid-related actions; tag-pinned first-party actions | `.github/workflows/droid*.yml` | ✓ |
| D-10 | Branch protection policy on `main` | `.github/settings.yml` | ✓ |
| D-11 | `cargo-deny` advisory check + license allowlist | `deny.toml` | ✓ |
| D-12 | BLAKE3 redaction with extension allowlist | `crates/tokmd-format/src/redact/` | ✓ |
| D-13 | Content reads bounded by `ContentLimits` | `crates/tokmd-analysis/src/content/` | ✓ |
| D-14 | PyO3 FFI invariants (no panic, GIL release, error translation) | `crates/tokmd-python/src/lib.rs` | ✓ |
| D-15 | WASM uses `MemFs` (no host fs) | `crates/tokmd-wasm/` | ✓ |
| D-16 | `web/runner` uses `textContent` (no `innerHTML`/`eval`/`new Function`) | `web/runner/main.js` | ✓ |
| D-17 | `web/runner` token in `sessionStorage` (not `localStorage`) | `web/runner/auth.js` | ✓ |
| D-18 | `web/runner` worker protocol allowlists modes & presets | `web/runner/messages.js` | ✓ |
| D-19 | Composite action installs tokmd with sha256 checksum verification | `action.yml` | ✓ |
| D-20 | Strict no-panic policy (new findings must be allowlisted) | `xtask/src/tasks/no_panic.rs` | ✓ |
| D-21 | `cargo audit` invoked with structured `--json`; malformed JSON treated as Pending | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-22 | `run_json` top-level JSON must be an object | `crates/tokmd-core/src/ffi/mod.rs` | ✓ |
| D-23 | Proof policy schema and scope routing validated | `ci/proof.toml`, `xtask proof-policy --check` | ✓ |

## Appendix

### Threat Model

- **Status:** Current (generated 2026-08-02; within the 90-day window)
- **Location:** `.factory/threat-model/threat-model.md`
- **Methodology:** STRIDE
- **No regeneration this scan.** The threat model is 92 days old as of
  2026-11-02, so it is now *past* the 90-day freshness window and is due for
  refresh. It was not regenerated because the scan window introduced no new
  external surface, no new subprocess invocation, and no trust-boundary shift
  — the three in-window commits touch the CI control plane and documentation
  only. **Action for the next scan:** regenerate
  `.factory/threat-model/threat-model.md`. The scan findings above (packet
  containment on both the writer and verifier sides; revision-argument
  neutralization in the control plane) should be folded into the standing
  defenses when that refresh happens.

### Scan Metadata

- **Commits Scanned:** 0 on the publication repository `main`; 3 on
  `EffortlessMetrics/tokmd-swarm:main` (`3e3dc2e`, `e79d82d`, `07d878be`),
  all dated 2026-10-26.
- **Unmerged in-window PRs reviewed:** swarm #775 (head `42d2a87`), swarm #776
  (head `ad9fedb`).
- **Findings:** 3 (all MEDIUM), all reproduced against built binaries, all
  fixed on this branch.
- **Scan Duration:** ~75m.
- **Skills Used:** threat-model-generation (reviewed, not regenerated),
  commit-security-scan, vulnerability-validation, security-review.
- **False Positive Filter:** applied — see OBS-001 through OBS-006.

### Scan Coverage

| Area | Files reviewed | Findings |
|------|----------------|----------|
| Review packet verifier | `xtask/src/tasks/review_packet_check.rs` | 1 (VULN-003) |
| Review packet writer | `crates/tokmd-cockpit/src/render/review_packet.rs` | 1 (VULN-004) |
| Git revision handling | `xtask/src/tasks/affected.rs`, `ci_plan.rs`, `mutation_scope.rs`, `repo_graph.rs`, `publish.rs` | 1 (VULN-005) |
| Affected-file discovery | `xtask/src/tasks/affected.rs` (swarm #771 NUL framing) | 0 |
| CI planning | `xtask/src/tasks/ci_plan.rs` (swarm #771 rename retention) | 0 |
| Build guard / gate | `xtask/src/tasks/build_guard.rs`, `gate.rs` (swarm #770) | 0 |
| Analysis scope selection | `crates/tokmd-analysis/src/analysis/files.rs` (swarm #775) | 0 |
| Model aggregation | `crates/tokmd-model/src/aggregate.rs` (swarm #776) | 0 |
| Path handling | `crates/tokmd-scan/src/path/`, `crates/tokmd-scan/src/walk/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/` | 0 |
| Content reads | `crates/tokmd-analysis/src/content/` | 0 |
| GitHub workflows | `.github/workflows/*.yml`, `action.yml`, `.github/settings.yml` | 0 |
| Build / lint policy | `Cargo.toml`, `deny.toml`, `clippy.toml`, `.cargo/config.toml` | 0 |
| Threat model | `.factory/threat-model/threat-model.md` | refresh due |

### Commit-level Analysis

The 7-day window (2026-10-26 → 2026-11-02) contains **no commits on `main`**
of the publication repository. `main` is unchanged since the 2026-08-05 import
commit `ff05889`. `git log origin/main --since="7 days ago" --oneline` returns
no output.

Development occurs in `EffortlessMetrics/tokmd-swarm` per the dual-repo
workbench boundary in `AGENTS.md`. Swarm `main` received three commits inside
the window, all on 2026-10-26:

| Commit | Subject | Security assessment |
|--------|---------|---------------------|
| `3e3dc2e` | docs: clarify same-session review and evidence reuse (#769) | Documentation only. No finding. |
| `e79d82d` | fix(gate): preserve other runs' target directories (#770) | Removes the age-based cross-run `/tmp` target sweep and the `find /tmp … -exec rm -rf` step in `ci.yml`. This is a **security improvement**, not a regression: the removed sweep used an age heuristic to decide that another run's target directory was abandoned, so it could delete a concurrently active run's multi-GiB target. `ScopedTempDir` still removes only the current invocation's directory on drop, and the disk-pressure refusal with its floor and override is preserved. `ScopedTempDir` uses a per-process unique suffix, so there is no prefix-collision or predictable-name issue. No finding. |
| `07d878b` | fix(ci): retain renamed paths in affected selectors (#771) | Security-positive. Moves diff parsing to NUL framing (`git diff --no-renames --name-only -z`) so a filename containing a newline is no longer split, stops treating a literal `\` in a Git filename as a directory separator, and adds `escape_debug` to the human-readable affected report so dynamic fields cannot inject terminal control sequences. The trailing `--` pathspec separator is present. No finding. |

Two pull requests were opened inside the window and remain unmerged at scan
time. Both were reviewed because they are in-window work:

- **swarm #775** (`42d2a87`, "fix(analysis): select bounded files across all
  scopes") changes `crates/tokmd-analysis/src/analysis/files.rs`. The scope
  resolution path `resolve_scope` canonicalizes the scope and the root and
  rejects any scope that does not `strip_prefix` the root, so a symlinked
  scope is caught; `tokmd_scan::walk::list_files` builds on `ValidatedRoot` and
  sets `follow_links(false)`. The `max_files` bound is enforced by popping the
  largest element whenever `files.len()` exceeds the limit. No finding.
- **swarm #776** (`ad9fedb`, "fix(model): count embedded module contributions
  once") changes `crates/tokmd-model/src/aggregate.rs` to skip a child row when
  a parent row with the same path is present. This is a conservation fix for
  embedded-language double counting. It changes reported counts, not access
  control, path handling, or subprocess behavior. No finding.

### Patches Generated

Three patches, one commit each:

| Commit | Finding | Files |
|--------|---------|-------|
| `2787b0ea` | VULN-003 + VULN-004 | `xtask/src/tasks/review_packet_check.rs`, `crates/tokmd-cockpit/src/render/review_packet.rs`, `docs/review-packet.md`, `docs/specs/review-packet.md` |
| `8ea34506` | VULN-005 | `xtask/src/tasks/affected.rs`, `xtask/src/tasks/ci_plan.rs`, `xtask/src/tasks/mutation_scope.rs` |

`crates/tokmd-cockpit/src/render/evidence.rs` also carries a one-line clippy
autofix (`format!("{DOC_ARTIFACTS_PACKET_PATH}")` → `DOC_ARTIFACTS_PACKET_PATH.to_string()`).
This is a pre-existing clippy finding that was keeping `cargo xtask gate
--check` red on `main`; it was applied by the repository's own
`cargo xtask lint-fix` while formatting the security changes, and it is
behavior-preserving.

### Validation

| Check | Result |
|-------|--------|
| `cargo build --locked --workspace --all-features` | pass |
| `cargo test --locked --all-features` | pass — 21070 passed, 0 failed |
| `cargo test --locked -p xtask --all-features` | 1 pre-existing failure (below) |
| `cargo test --locked -p tokmd-cockpit --all-features` | pass — all suites green |
| `cargo xtask gate --check` | pass — 4/4 steps |
| `cargo xtask lint-fix` | pass — all steps |
| `cargo fmt --all -- --check` | pass |
| `cargo xtask proof-policy --check` | pass — 89 scopes, 1 allowlist |
| `cargo xtask docs --check` | pass — documentation up to date |

**Pre-existing failure, verified identical at base commit `ff05889` with the
security changes stashed (not caused by this change):** `xtask` test
`publishable_internal_dev_dependencies_use_loose_versions`
(`xtask/tests/publish_w71.rs:341`). This is the same failure the 2026-10-26
scan recorded.

### Follow-ups (recorded, not actioned here)

1. **Regenerate the threat model.** `.factory/threat-model/threat-model.md` is
   dated 2026-08-02 and is now past the 90-day window. Fold in the packet
   containment defenses (writer and verifier) and the control-plane revision
   neutralization.
2. **Merge swarm PR #750** (`bc3220b`), "fix(xtask): constrain resolved review
   packet artifact paths". It is CI-qualified and addresses the same verifier
   side as VULN-003. The fix on this branch is functionally equivalent for the
   escape case; when #750 merges, prefer its version to avoid two competing
   implementations of the same check.
3. **Land swarm #771** in the publication repository so the NUL-framed diff
   parsing and the `\` path-identity correction reach `main` (OBS-001).
4. **Fix `publishable_internal_dev_dependencies_use_loose_versions`**, which
   keeps one `xtask` test red.
5. **Consider `cargo xtask` argument hardening at the parser layer**, so the
   revision invariant does not depend on each call site remembering to call
   `ensure_ref_is_not_option_like`.
6. **Confirm the `Codex Review Gate` required status check has a producer**, or
   remove it from `.github/settings.yml` (carried from the 2026-10-26 scan).

## References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Previous scans: `.factory/security/reports/security-report-2026-07-27.md`,
  `.factory/security/reports/security-report-2026-10-26.md` (publication),
  and swarm `droid/security-report-2026-10-26` (PR #772)
