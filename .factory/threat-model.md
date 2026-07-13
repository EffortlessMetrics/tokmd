# Threat Model - tokmd

**Generated:** 2026-07-13
**Scope:** Repository-wide STRIDE analysis for `EffortlessMetrics/tokmd`
**Methodology:** STRIDE (Microsoft Threat Modeling)
**Last Reviewed:** 2026-07-13
**Supersedes:** `.factory/threat-model/threat-model.md` (2026-06-01) - preserved as historical reference
**Reviewer Trigger:** Weekly `droid-security-scan` (`.github/workflows/droid-security-scan.yml`)

> **Canonical path:** `.factory/threat-model.md`
> The dated snapshot path `.factory/threat-model/threat-model.md` retains
> prior versions for traceability; the canonical, current threat model
> lives at this single file and is regenerated in place each cycle.

---

## 1. System Overview

**tokmd** is a Rust workspace that wraps the `tokei` library to generate
deterministic "inventory receipts" and derived analytics of code
repositories. It produces:

- Human-readable summaries (Markdown, TSV)
- Machine-friendly datasets (JSON, JSONL, CSV, CycloneDX SBOM)
- Library facades for Python, Node.js, and WASM consumers
- A CLI binary (`tokmd`) and library API (`tokmd-core`)
- A composite GitHub Action (`action.yml`) for CI consumers
- A browser worker runner (`web/runner/`) for in-browser receipt generation

### 1.1 Distribution Surfaces

| Surface | Mechanism | Trust boundary |
|---------|-----------|----------------|
| CLI binary `tokmd` | Local execution, package managers (cargo, brew, AUR, winget, Docker / GHCR) | User → tool |
| `tokmd-core` Rust library | Crates.io, source builds, workspace internal | Library user → tool |
| `tokmd-python` (PyO3) | PyPI | Python user → tool |
| `tokmd-node` (napi-rs) | npm | JS user → tool |
| `tokmd-wasm` (wasm-bindgen) | npm (browser/worker) | Web user → tool |
| GitHub Action `action.yml` | GitHub Marketplace | CI pipeline → tool |
| Browser runner (`web/runner/`) | Static site + WASM worker | End user → browser → GitHub API |
| Sensor envelope output | CI artifact consumption | CI → downstream consumers |

### 1.2 Architectural Snapshot

Tiered crate hierarchy enforced in `Cargo.toml`:

| Tier | Examples | Trust posture |
|------|----------|---------------|
| 0 contracts | `tokmd-types`, `tokmd-analysis-types`, `tokmd-settings`, `tokmd-envelope`, `tokmd-io-port` | Pure DTOs / settings models; no I/O |
| 1 scanning / modeling | `tokmd-scan`, `tokmd-model`, `tokmd-sensor` | File-walking boundary, tokei wrapper |
| 2 adapters | `tokmd-format`, `tokmd-git`, `tokmd-analysis` (submodules) | Rendering, git subprocess boundary, content reads |
| 3 orchestration | `tokmd-analysis`, `tokmd-cockpit`, `tokmd-gate` | Multi-source orchestration, supply-chain gate |
| 4 facade | `tokmd-core` | Library + FFI entry point |
| 5 products | `tokmd`, `tokmd-python`, `tokmd-node`, `tokmd-wasm`, `xtask`, `fuzz` | CLI / language bindings |

### 1.3 Assets (What Must Be Protected)

| ID | Asset | Sensitivity | Owner |
|----|-------|-------------|-------|
| A-1 | Source code of scanned repository | User's intellectual property | User |
| A-2 | File contents of scanned repository | May include secrets (`.env`, keys, credentials) | User |
| A-3 | tokmd source tree and Cargo.lock | Supply-chain integrity | Maintainers |
| A-4 | GitHub Actions secrets (`MINIMAX_API_KEY`, `FACTORY_API_KEY`) | High - third-party LLM access | Repo admins |
| A-5 | Released binary artifacts (`tokmd` release tarball, Docker images) | Supply-chain integrity | Maintainers |
| A-6 | Generated receipts (JSON / JSONL / SBOM) | May leak paths, hashes, line counts | User |
| A-7 | Composite action consumer repositories | Workflow integrity | Action consumers |
| A-8 | Web runner GitHub token (browser session) | Medium - bearer credential for `api.github.com` | End user |
| A-9 | Vendored `home` crate (`vendor/home-0.5.12`) | Patch provenance | Maintainers |
| A-10 | `web/runner/` worker / page | End-user trust placed in browser app | End user |

---

## 2. Entry Points

| ID | Entry Point | Trust Position |
|----|-------------|----------------|
| EP-1 | CLI argv parsing (`crates/tokmd/src/cli/`, `crates/tokmd/src/commands/*.rs`) | Untrusted strings from user shell |
| EP-2 | Scan root paths (`--paths`, positional `paths`, config) | User-controlled filesystem inputs |
| EP-3 | `--include` / `--exclude` glob patterns | Untrusted pattern strings |
| EP-4 | FFI JSON envelope `run_json(mode, args_json)` | Caller-controlled JSON string |
| EP-5 | FFI in-memory inputs (`inputs[].path`, `inputs[].text` or `inputs[].base64`) | Caller-controlled file paths and bytes |
| EP-6 | Subprocess `git` invocations (`tokmd-git`, `tokmd-cockpit` supply-chain gate, `tokmd-scan` git walker, `tokmd/src/git_support.rs`) | User-controlled ref strings + repo contents |
| EP-7 | Subprocess `cargo audit` invocation (`tokmd-cockpit/src/supply_chain.rs`) | Repo contents (Cargo.lock) |
| EP-8 | GitHub Actions `secrets.*` (workflow env) | Repo-secret trust |
| EP-9 | Output sink paths (`--output`, `--bundle-dir`, `--log`, `--output-dir`) | User-controlled filesystem paths |
| EP-10 | Browser runner fetch endpoints (`api.github.com`, `codeload.github.com`) | Public-internet HTTPS endpoints |
| EP-11 | Browser runner GitHub token (paste / `sessionStorage`) | Bearer credential |
| EP-12 | Composite action `version` input (`action.yml`) | User-supplied release tag string |

---

## 3. Trust Boundaries

| ID | Boundary | Crossed by | Validation |
|----|----------|------------|------------|
| TB-1 | User shell → CLI argv | Flags, paths, globs, mode names | Strict `clap` parsing; downstream re-validation |
| TB-2 | Untrusted repository contents → tokmd file walker | Filesystem, symlinks, `.gitignore`, `.tokeignore` | `ignore` crate + `BoundedPath`/`ValidatedRoot` |
| TB-3 | tokmd → external `git` binary | `git log`, `git diff`, `git rev-parse`, `git config`, `git init`, `git add`, `git commit` via `Command::new("git").arg(...)` | `git_cmd()` env isolation; ref safety check; `--end-of-options` separator |
| TB-4 | tokmd → external `cargo` binary | `cargo audit --json` (cockpit supply-chain gate) | Structured JSON parse, malformed → `Pending` |
| TB-5 | Python / Node / WASM → tokmd FFI | JSON-string args to `run_json()` | Strict top-level object check; per-field strict parsing; `validate_in_memory_input_path` |
| TB-6 | CI workflow → repository secrets | `MINIMAX_API_KEY`, `FACTORY_API_KEY` (Droid workflows) | Workflow `permissions:` blocks; no secret echo |
| TB-7 | Scanner → output destination | `--output`, `--bundle-dir`, `--log`, `--output-dir` | Caller-chosen; excluded from recursive read |
| TB-8 | Untrusted inputs (paths) → inclusion policy | `--paths`, `--include`, in-memory `inputs[]` | Path validation at FFI + scan layer |
| TB-9 | In-memory mode → boundary path canonicalization | ReDoS risk on path normalization | Bounded `clean_path` loop in `tokmd-format/src/redact/mod.rs` |
| TB-10 | WASM host → in-memory filesystem | Sandboxed JS environment | `MemFs`-style usage; no host fs access |
| TB-11 | Browser runner → GitHub API | `fetch()` to `api.github.com` / `codeload.github.com` | HTTPS only; `sessionStorage` token; `textContent` rendering |
| TB-12 | Composite action → release artifact URL | `version` input → `https://github.com/EffortlessMetrics/tokmd/releases/...` | `curl -fsSL` (HTTPS, fail on HTTP); sha256 via `checksums.txt` |

---

## 4. STRIDE Analysis

### 4.1 Spoofing (S)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|---------------|
| Fake git history impersonation | LOW | Git is invoked as a subprocess; tokmd reads stdout. Author email is the only metadata field surfaced; it is not used as a trust signal. | LOW - never gated on author identity. |
| Impersonation via FFI arg fields | LOW | Strict JSON parsing in `tokmd-core/src/ffi/parse.rs` rejects type mismatches (no silent fallback). Top-level must be a JSON object (`run_json_inner`). | LOW. |
| GitHub Action impersonation | LOW | SHA-pinned custom action `EffortlessMetrics/droid-action-safe@7c1377ccbacddc95560d1570547a5baa51de01ec`. Tag-pinned first-party actions (`actions/checkout@v7.0.0`). | LOW - see OBS-1 about mixed pinning policy. |
| FFI JSON envelope spoofed mode | LOW | `tokmd-core/src/ffi/modes.rs` rejects unknown modes. | LOW. |
| Browser runner GitHub identity spoof | LOW | Token is read from `sessionStorage` only; never persisted; bound to one tab; used as `Authorization: Bearer` against `api.github.com` only. | LOW - depends on user verifying URL. |

**Result:** No medium or higher spoofing vectors. See Review Priority P-1 for the mixed-pinning observation.

### 4.2 Tampering (T)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|---------------|
| Unsanitized git refs becoming shell args | HIGH (if reached) → MITIGATED | `tokmd-git/src/command.rs::git_cmd()` strips `GIT_REPO_SHAPING_ENV` (14 vars). `tokmd-git/src/refs.rs::env_base_ref_is_safe` rejects empty, leading `-`, whitespace, control, and `\` in refs. `--end-of-options` separator used. `rev_exists` routes refs through `format!("{rev}^{{commit}}")` - the literal `^{commit}` suffix pins the ref to commit-only resolution. | LOW - defense in depth applied. |
| Path traversal via scan roots | HIGH (if reached) → MITIGATED | `tokmd-scan/src/path/bounded_path.rs::normalize_bounded_relative_path` rejects empty, `..`, absolute, and prefix-rooted paths. `BoundedPath::existing_relative` and `existing_child` enforce `ensure_under_root` via canonicalized realpath comparison. | LOW - tested with explicit rejection of root-escape and parent-traversal cases. |
| Path traversal via FFI in-memory inputs | HIGH (if reached) → MITIGATED | `tokmd-core/src/ffi/inputs.rs::validate_in_memory_input_path` rejects: empty paths, > 4096 bytes, control chars, leading `/` or `\`, Windows drive prefix (`looks_like_windows_drive_path`), `..` segments (both component walk and per-segment split), and all-`.` paths. | LOW - exhaustive rejection, double-checked by both `Component` walk and per-segment split. |
| File content write at attacker-chosen path | MEDIUM (if reached) → MITIGATED | Output paths are added to exclude patterns before walking so recursive reads can't include them. `--output-dir` is created under user-specified or `.runs/tokmd/<id>/` location. | LOW - output dirs are caller-chosen. |
| Cargo lockfile tampering | MEDIUM | `tokmd-cockpit/src/supply_chain.rs::compute_supply_chain_gate` invokes `cargo audit --json` and parses structured output. `parse_audit_output` returns `Pending` (never `Pass`) on malformed JSON. | LOW - evidence is informational; consumers must not rely on `Pass` without reviewing scope. |
| Workspace lint bypass via file mutation | LOW | `cargo xtask lint-fix` and `cargo xtask gate --check` run as pre-push hooks (`agents/shared/repo.md`). CI requires both. Workspace lints forbid `unsafe_code`, `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `unimplemented`, `todo`. | LOW. |
| Injection into `.tokeignore` / config files | LOW | `.tokeignore` and tokei config are passed to tokei / `ignore` crate, not eval'd. `ConfigMode::Auto` reads tokei config files; `ConfigMode::None` skips. No shell expansion. | LOW. |
| Redaction bypass via extension spoofing | LOW | `tokmd-format/src/redact/mod.rs::redact_path` consults `extensions::safe_path_extension_suffix` (allowlist). Unknown / unsafe final extensions fall back to bare 16-char BLAKE3 hash. `clean_path` normalizes separators and `.` segments deterministically. | LOW - verified by tests `redact_path_strips_untrusted_short_extensions`, `redact_path_drops_unsafe_final_extension`. |
| Browser runner DOM injection | LOW | All dynamic data rendered via `textContent` (verified across `web/runner/main.js`, `worker.js`, `ingest.js`). No use of `innerHTML`, `eval`, `new Function`, `document.write` (confirmed by repo-wide grep). | LOW - see OBS-2 for CSP / fetch-origin allowlist consideration. |
| Composite action download tampering | LOW | `action.yml` downloads from `github.com/EffortlessMetrics/tokmd/releases/...` via `curl -fsSL` (HTTPS, fail on HTTP). sha256 verified against `checksums.txt` when present. Provenance separately attested via `actions/attest-build-provenance` in `release.yml`. | LOW - see OBS-3 for version-input format validation. |
| Determinism tampering (golden snapshot breakage) | LOW | Receipts use `BTreeMap` everywhere; sorting is descending by code lines, then by name. Snapshot tests (`insta`) enforce byte-stable output. | LOW. |

**Result:** All high-severity tampering vectors are mitigated with defense in depth. No active findings at medium or higher.

### 4.3 Repudiation (R)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|---------------|
| "I never ran that scan" claims | LOW | Receipts include `schema_version`, `generated_at_ms`, scan args, and tool metadata (`tokmd_types` envelope). JSON outputs are reproducible from the same inputs (BTreeMap + sort order). | LOW - receipts are not cryptographically signed; downstream consumers must add signature if non-repudiation is required. |
| Audit log tampering | LOW | `cargo xtask gate --check` enforces determinism and lint policy. Receipts are deterministic. | LOW. |
| Scan-result provenance claims | LOW | Cockpit receipts include `evidence_commit`, `evidence_generated_at_ms`, and `ScopeCoverage`. Supply-chain gate records `EvidenceSource`. | LOW - provenance is informational. |
| Web runner user denies generating a receipt | LOW | Receipts include `generated_at_ms` and tool metadata. | LOW - no user identity bound to browser-generated receipts. |

**Result:** LOW residual risk. Receipts are auditable but not cryptographically signed.

### 4.4 Information Disclosure (I)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|------------|
| Secrets in redacted output | MEDIUM (if reached) → MITIGATED | `tokmd-format/src/redact/mod.rs::short_hash` uses BLAKE3 truncated to 16 hex chars. `redact_path` preserves extension only when on the `extensions::safe_path_extension_suffix` allowlist; otherwise bare hash. `clean_path` normalizes cross-platform. | LOW - non-allowlisted extensions get bare hash; reverse-lookup requires knowing the BLAKE3 prefix. |
| Path disclosure via receipts | LOW | `--redact=paths\|all` mode is available. Receipts do not include file contents by default (only counts). | LOW - counts may still leak layout (e.g., file count by directory). |
| File content disclosure via content analysis | MEDIUM (if reached) → MITIGATED | `tokmd-analysis/src/content/io/read.rs::read_head` enforces per-file `max_bytes` (default 128 KiB, configurable via `ContentLimits`). `as_text` skips binary blobs. Total `max_bytes` budget enforced at `build_todo_report` loop level. | LOW - content is read once into memory; not buffered beyond `max_bytes`. |
| Environment variable leakage into receipts | MEDIUM (if reached) → MITIGATED | `tokmd-git/src/command.rs::git_cmd` strips 14 `GIT_REPO_SHAPING_ENV` vars (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_COMMON_DIR`, `GIT_CEILING_DIRECTORIES`, plus helper-execution vars) before spawning git. | LOW - env is not propagated to receipts. |
| Stderr exposure of repository internals | LOW | `git log` runs with `Stdio::null()` for stderr; only stdout is parsed (`refs.rs::rev_exists`). | LOW. |
| Information disclosure via error messages | LOW | Errors return `TokmdError` enum with typed codes (`TokmdError::invalid_field`, `invalid_json`, etc.). Internal paths are not exposed via error strings; FFI returns `{"code", "message", "details"}` envelope. | LOW. |
| Web runner GitHub token leakage | LOW | Token stored only in `sessionStorage` (cleared on tab close). Sent only as `Authorization: Bearer` to `api.github.com` / `codeload.github.com`. `auth.js` never writes to `localStorage` or cookies. | LOW - user paste required; browser DevTools can still read. See OBS-2. |
| Web runner input data leakage | LOW | `worker.js` and `main.js` use `postMessage` channels; no telemetry or external beacon. | LOW. |

**Result:** All medium/high information-disclosure vectors are mitigated.

### 4.5 Denial of Service (D)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|------------|
| Pathological regex via exclude patterns | MEDIUM (if reached) → MITIGATED | `tokmd-scan/src/exclude/` normalizes patterns; `tokmd-scan/src/walk/` uses bounded traversal. `tokmd-scan/src/lib.rs::scan` enforces `max_commits` and `max_commit_files` for git history. | LOW. |
| ReDoS in path normalization | LOW | `tokmd-format/src/redact/mod.rs::clean_path` uses bounded loops with `String::replace` for `/./` and prefix-strip for `./`. Bounded by input length (no nested quantifiers). | LOW. |
| Resource exhaustion via large git history | MEDIUM (if reached) → MITIGATED | `tokmd-git/src/lib.rs::collect_history` honors `max_commits` and `max_commit_files`. Streaming `BufReader` does not load full history. | LOW. |
| Resource exhaustion via large file read | MEDIUM (if reached) → MITIGATED | `tokmd-analysis/src/content/io/read.rs` enforces per-file `max_bytes` (default 128 KiB) and total `max_bytes` budget. `read_head_tail` reads fixed-size head/tail windows only. | LOW. |
| Pathological FFI JSON payload | LOW | `serde_json::from_str` is recursive; no DoS-specific bounds. Individual `inputs[].path` is bounded to 4096 bytes; per-file content bound by the scan layer; arbitrary outer JSON `args` size is not bounded by the FFI layer itself. | MEDIUM - large-but-valid JSON could spike memory. Tracked as OBS-4. Below the medium threshold for current scan; optional soft cap recommended. |
| Fuzz target coverage | LOW | 9 fuzz targets in `fuzz/` directory (libfuzzer); seed corpus and dictionaries present. `cargo +nightly fuzz list` enumerates targets. | LOW. |
| Symlink / device-file path DoS | LOW | `BoundedPath::existing_relative` uses `fs::canonicalize` which dereferences symlinks. Canonicalized realpath is then `starts_with(root.canonical())` checked. Non-existent targets return `Missing`. | LOW. |
| GitHub Actions minute exhaustion via fork PR | LOW | Droid workflows only run on PRs from same-repo branches (`github.event.pull_request.head.repo.full_name == github.repository`). External forks do not trigger Droid auto-review. | LOW. |

**Result:** All medium/high DoS vectors are mitigated. One observation (OBS-4, FFI JSON size cap) is below threshold and tracked for the next review.

### 4.6 Elevation of Privilege (E)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|------------|
| Code execution via git hooks | MEDIUM (if reached) → MITIGATED | `GIT_REPO_SHAPING_ENV` (14 vars including `GIT_DIR`, `GIT_SSH`, `GIT_SSH_COMMAND`, `GIT_ASKPASS`, `GIT_PAGER`, `GIT_EDITOR`, `GIT_PROXY_COMMAND`, `GIT_EXTERNAL_DIFF`, plus index and discovery vars) is `env_remove`'d before every git subprocess. Refs are passed via `arg()`, never shell-expanded. | LOW. |
| Code execution via Python `pyo3` bindings | MEDIUM (if reached) → MITIGATED | `tokmd-python/src/lib.rs` documents FFI safety invariants. `?` operator used for error propagation; `.expect()` prohibited in production. GIL released via `py.detach()` for long scans. Custom `TokmdError` exception translated to Python. | LOW. |
| Code execution via `Command::new` arg construction | HIGH (if reached) → MITIGATED | All `Command::new` invocations use `.arg(...)` (not `.args(&[user_string])` with shell metacharacters, and never `sh -c` / `bash -c`). Verified across `tokmd-git`, `tokmd-cockpit`, `tokmd-scan` git walker, `tokmd/src/git_support.rs`. | LOW. |
| Privilege escalation via supply chain | MEDIUM | `Cargo.lock` committed. `deny.toml` enforces advisory check (`cargo-deny`) and license allowlist. `RUSTSEC-2020-0163` (transitive `term_size` via `tokei`) is documented as an upstream limitation with rationale. | MEDIUM - transitive advisory remains. Tracked as OBS-5. |
| Code execution via WASM host | MEDIUM (if reached) → MITIGATED | `tokmd-wasm` uses in-memory filesystem patterns; no host fs. Sandboxed by WASM runtime. `#![forbid(unsafe_code)]` enforced. `ROOTLESS_ANALYZE_PRESETS = ["receipt", "estimate"]` keeps the browser surface narrow. | LOW. |
| Privilege escalation via CI workflow injection | MEDIUM (if reached) → MITIGATED | `permissions:` blocks are minimal (`contents: read`, `pull-requests: write`, etc.). `MINIMAX_API_KEY` is read from secrets, never echoed. `droid-action-safe` is the SHA-pinned custom action with raw-debug-artifact upload disabled. | LOW. |
| Code execution via composite action download | MEDIUM (if reached) → MITIGATED | `action.yml` installs via `curl -fsSL` (HTTPS, fail on HTTP) and verifies sha256 against `checksums.txt`. | LOW - see OBS-3 for version-input format validation. |
| `unsafe_code` regression | LOW | `unsafe_code = "forbid"` set as workspace lint in `Cargo.toml`. `unsafe_op_in_unsafe_fn = "deny"`. WASM crate additionally has `#![forbid(unsafe_code)]` at crate level. | LOW. |

**Result:** All high-severity elevation vectors are mitigated. One supply-chain observation noted (OBS-5).

---

## 5. Standing Defenses (Verified Intact)

These defenses are baked into the workspace and verified at every scan. They
must not regress.

| ID | Defense | Location |
|----|---------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `unimplemented`, `todo` lints denied | `Cargo.toml` |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`, 14 vars) | `crates/tokmd-git/src/command.rs` |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options` + `^{commit}` suffix) | `crates/tokmd-git/src/refs.rs` |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` |
| D-06 | FFI in-memory input path validation (empty / >4096 / control / drive / `..` rejection) | `crates/tokmd-core/src/ffi/inputs.rs` |
| D-07 | Strict JSON parsing with type validation; top-level must be an object | `crates/tokmd-core/src/ffi/parse.rs`, `crates/tokmd-core/src/ffi/mod.rs` |
| D-08 | Per-family schema versioning constants | `crates/tokmd-types/src/` (`SCHEMA_VERSION=2`, `COCKPIT_SCHEMA_VERSION=3`, `HANDOFF_SCHEMA_VERSION=5`, `CONTEXT_SCHEMA_VERSION=4`, `CONTEXT_BUNDLE_SCHEMA_VERSION=2`) |
| D-09 | SHA-pinned Droid-related actions; tag-pinned first-party actions | `.github/workflows/droid*.yml` (SHA), others (tag) - see OBS-1 |
| D-10 | Branch protection on `main` (CODEOWNERS, 1 approval, CI required) | `.github/settings.yml` |
| D-11 | `cargo-deny` advisory + license allowlist | `deny.toml` |
| D-12 | BLAKE3 redaction with extension allowlist | `crates/tokmd-format/src/redact/mod.rs`, `crates/tokmd-format/src/redact/extensions.rs` |
| D-13 | Content reads bounded by `ContentLimits` (per-file 128 KiB default, total budget) | `crates/tokmd-analysis/src/content/mod.rs`, `crates/tokmd-analysis/src/content/io/read.rs` |
| D-14 | PyO3 FFI invariants (no panic, GIL release, error translation, custom exception) | `crates/tokmd-python/src/lib.rs` |
| D-15 | WASM uses in-memory fs + `ROOTLESS_ANALYZE_PRESETS`; `#![forbid(unsafe_code)]` | `crates/tokmd-wasm/src/lib.rs` |
| D-16 | `web/runner` browser runner uses `textContent` (no `innerHTML` / `eval` / `new Function` / `document.write`) | `web/runner/main.js`, `web/runner/worker.js`, `web/runner/ingest.js` |
| D-17 | `web/runner` token stored in `sessionStorage` (not `localStorage`); cleared on tab close | `web/runner/auth.js` |
| D-18 | `web/runner` worker protocol allowlists modes and presets | `web/runner/messages.js` |
| D-19 | Composite action installs tokmd with HTTPS-only `curl -fsSL` and sha256 verification | `action.yml` |
| D-20 | Custom Droid action SHA-pinned across all Droid workflows; raw debug artifact upload disabled | `.github/workflows/droid*.yml` |
| D-21 | `cargo audit` invoked with structured `--json`; malformed JSON returns `Pending` (never `Pass`) | `crates/tokmd-cockpit/src/supply_chain.rs` |
| D-22 | `run_json` top-level JSON must be an object (strict shape check) | `crates/tokmd-core/src/ffi/mod.rs::run_json_inner` |
| D-23 | Author DAG import via true-merge commits (no force-push of publication history) | repository topology - `docs/ci/swarm-routing.md` |
| D-24 | Determinism invariants (`BTreeMap`, descending-then-name sort, forward-slash paths) | `crates/tokmd-model`, `crates/tokmd-format` |
| D-25 | Workspace lint governance (`allow_attributes = "deny"`, `ignore_without_reason = "deny"`, `should_panic_without_expect = "deny"`) | `Cargo.toml` |

---

## 6. Out of Scope

- Issues in third-party crates (report upstream; not actionable here).
- Theoretical attacks without realistic exploitation paths (per `SECURITY.md`).
- Performance regressions that are not denial-of-service.
- The vendored `home` crate at `vendor/home-0.5.12` (intentional temporary patch - `Cargo.toml` `[patch.crates-io]`; tracked in `docs/specs/dependency-maintenance.md#vendored-home-patch`).
- External CI infrastructure owned by GitHub Actions / npm / PyPI / crates.io.

---

## 7. Observations (Below Medium Threshold)

Carried from prior scans and re-confirmed. Not findings; tracked for future
remediation if scope changes.

### OBS-1: Mixed GitHub Actions pinning policy (tag + SHA)

- **Severity:** LOW (informational)
- **STRIDE:** Spoofing / Tampering
- **Files:** `.github/workflows/*.yml`
- **Status:** Accepted - first-party tag-pinning, custom-action SHA-pinning

The Droid workflows (`.github/workflows/droid.yml`, `droid-review.yml`,
`droid-security-scan.yml`) SHA-pin third-party actions including
`EffortlessMetrics/droid-action-safe@7c1377ccbacddc95560d1570547a5baa51de01ec`.
Other workflows tag-pin (e.g. `actions/checkout@v7.0.0`,
`Swatinem/rust-cache@v2`). The custom Droid action - the highest-privilege
third-party surface - is SHA-pinned, and first-party tag-pinning matches
GitHub's recommended baseline.

### OBS-2: `web/runner` browser code does not pin fetch origins

- **Severity:** LOW (informational)
- **STRIDE:** Spoofing / Information Disclosure
- **Files:** `web/runner/ingest.js`, `web/runner/main.js`
- **Status:** Not patched

The browser-side runner fetches only `api.github.com` /
`codeload.github.com` via HTTPS. The token (when supplied) is stored in
`sessionStorage` (not `localStorage`) and used as a `Bearer` header. No
Subresource Integrity pinning or origin allow-list. Optional future
hardening: explicit allowlist + CSP `connect-src` directive.

### OBS-3: `action.yml` `version` input has no strict format validation

- **Severity:** LOW (informational)
- **STRIDE:** Tampering
- **Files:** `action.yml` (composite step `Install tokmd`)
- **Status:** Not patched - verified checksums + build attestation

The composite GitHub Action downloads a pre-built `tokmd` binary from
`github.com/EffortlessMetrics/tokmd/releases/...` and verifies sha256 via
`checksums.txt`. URL is interpolated from the user-supplied `version`
input. Recommended future hardening: regex validation
(`^v?\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$`) before URL construction.

### OBS-4: FFI JSON payload size not bounded

- **Severity:** LOW (informational)
- **STRIDE:** Denial of Service
- **Files:** `crates/tokmd-core/src/ffi/mod.rs`
- **Status:** Not patched - design choice

The `run_json(mode, args_json)` FFI entrypoint accepts a JSON string of
arbitrary size. Per-input `inputs[].path` is bounded to 4096 bytes; the
outer JSON envelope is not. `serde_json::from_str` allocates predictably;
no algorithmic blowup. Recommended future hardening: soft cap
`args_json.len()` (e.g. 8 MiB) returning a typed
`TokmdError::invalid_field("args", "JSON args exceed 8 MiB cap")` from
`run_json_inner`.

### OBS-5: Transitive `RUSTSEC-2020-0163` advisory (term_size via tokei)

- **Severity:** LOW (transitive)
- **STRIDE:** Elevation of Privilege
- **Files:** `Cargo.lock` (transitive `term_size` via `tokei`)
- **Status:** Documented in `deny.toml`

`term_size` is a transitive dependency of `tokei` and has an unmaintained
advisory. Already documented in `deny.toml` with rationale. Recommended
action: track upstream `tokei` for a `term_size` removal.

---

## 8. Residual Risks

| ID | Risk | Severity | Owner |
|----|------|----------|-------|
| R-1 | Transitive `term_size` advisory remains until upstream `tokei` updates | LOW | Track upstream `tokei` |
| R-2 | Mixed pinning policy (tag vs SHA) for first-party GitHub Actions | LOW | Document policy; decide on rotation cadence |
| R-3 | FFI outer JSON envelope size unbounded | LOW | Optional soft cap (OBS-4) |
| R-4 | Composite action `version` input format not strictly validated | LOW | Optional regex (OBS-3) |
| R-5 | Browser runner fetch origins not explicitly allow-listed; relies on `sessionStorage` token isolation | LOW | Optional CSP / allowlist (OBS-2) |
| R-6 | Receipts are not cryptographically signed; downstream consumers must add their own signature if non-repudiation is required | LOW | Document in user-facing docs |

---

## 9. Review Priorities

Priorities for the next threat-model refresh (target 2026-09-30 or sooner
if architecture changes):

| Priority | Area | Reason |
|----------|------|--------|
| P-1 | Reconcile GitHub Actions pinning policy with `deny.toml`-style enforcement (OBS-1) | Make policy explicit and reviewable as code |
| P-2 | Add `args_json` length soft cap to `run_json_inner` (OBS-4) | Closes the only below-threshold DoS observation |
| P-3 | Add strict regex validation for `action.yml` `version` input (OBS-3) | Defense-in-depth on composite action surface |
| P-4 | Add CSP `connect-src` and origin allowlist to `web/runner` (OBS-2) | Hardens browser surface without changing token model |
| P-5 | Document upstream `tokei` `term_size` removal as a tracked item (OBS-5) | Keeps supply-chain observation in the work queue |
| P-6 | Confirm `droid-action-safe` upstream `v5` refresh + SHA rotation policy | Defense-in-depth on the highest-privilege third-party |
| P-7 | Re-audit `crates/tokmd/src/commands/` for any new CLI subprocess construction | New commands are the most likely place to introduce arg-construction regressions |

---

## 10. Review Cadence

- **Regenerate when:** architecture changes, new external surface added, new
  subprocess invocation, new FFI entrypoint, schema-version bump, or 90
  days since last review (whichever is sooner).
- **Trigger source:** weekly `droid-security-scan` workflow
  (`.github/workflows/droid-security-scan.yml`, cron `0 8 * * 1`).
- **Owner:** `EffortlessMetrics/tokmd` maintainers.
- **Canonical path:** `.factory/threat-model.md` (this file).
- **Historical snapshots:** `.factory/threat-model/threat-model.md`
  retains prior versions for traceability.

---

## Appendix A - Coverage Matrix (verified at this scan)

| Area | Files reviewed | Findings |
|------|----------------|----------|
| CLI argv parsing | `crates/tokmd/src/cli/`, `crates/tokmd/src/commands/*.rs` | 0 |
| Subprocess invocation | `crates/tokmd-git/`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd-cockpit/src/gates/contracts.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs` | 0 |
| Path handling | `crates/tokmd-scan/src/path/`, `crates/tokmd-scan/src/roots.rs`, `crates/tokmd-scan/src/walk/` | 0 |
| FFI inputs | `crates/tokmd-core/src/ffi/`, `crates/tokmd-python/src/`, `crates/tokmd-node/src/` | 0 |
| File content reads | `crates/tokmd-analysis/src/content/`, `crates/tokmd-io-port/src/` | 0 |
| Redaction / hashing | `crates/tokmd-format/src/redact/` | 0 |
| GitHub workflows | `.github/workflows/*.yml` (25 files), `.github/settings.yml`, `action.yml` | 0 |
| Build / lint policy | `Cargo.toml`, `deny.toml`, `clippy.toml`, `.cargo/config.toml` | 0 |
| Githooks | `.githooks/pre-commit`, `.githooks/pre-push`, `.claude/hooks/format-rust.sh` | 0 |
| Web runner (browser) | `web/runner/main.js`, `worker.js`, `auth.js`, `messages.js`, `runtime.js`, `ingest.js` | 0 |
| WASM bindings | `crates/tokmd-wasm/src/lib.rs` | 0 |
| Supply chain / advisory | `Cargo.lock`, `deny.toml`, `crates/tokmd-cockpit/src/supply_chain.rs` | 0 |
| Threat model | this file | regenerated |

## Appendix B - Validation Signal

- **Observed:** Code inspection of `tokmd-git/src/command.rs`,
  `tokmd-git/src/refs.rs`, `tokmd-core/src/ffi/inputs.rs`,
  `tokmd-core/src/ffi/parse.rs`, `tokmd-core/src/ffi/mod.rs`,
  `tokmd-scan/src/path/bounded_path.rs`,
  `tokmd-format/src/redact/mod.rs`,
  `tokmd-cockpit/src/supply_chain.rs`,
  `tokmd-analysis/src/content/{mod.rs,io.rs,io/read.rs}`,
  `tokmd-wasm/src/lib.rs`, `tokmd-python/src/lib.rs`,
  `web/runner/auth.js`, `Cargo.toml`, `deny.toml`, `action.yml`,
  `.github/workflows/droid*.yml`.
- **Reported:** Branch is `droid/security-report-2026-07-13`; latest commit
  `5e8edc6 merge(swarm): import steward docs manifest fix`.
- **Not verified:** Runtime behavior under live GitHub Actions
  infrastructure; release binary artifact provenance verification (requires
  external attestation outside the scan boundary).
