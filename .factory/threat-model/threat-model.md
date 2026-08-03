# Threat Model — tokmd-swarm

**Generated:** 2026-08-02
**Scope:** Repository-wide STRIDE analysis for `EffortlessMetrics/tokmd-swarm`
**Methodology:** STRIDE (Microsoft Threat Modeling)
**Last Reviewed:** 2026-08-02

## 1. System Overview

**tokmd** is a Rust workspace that wraps the `tokei` library to generate
"inventory receipts" and analytics of code repositories. It produces:

- Human-readable summaries (Markdown, TSV)
- Machine-friendly datasets (JSON, JSONL, CSV, CycloneDX SBOM)
- Library facades for Python, Node.js, and WASM consumers
- A CLI binary (`tokmd`) and library API (`tokmd-core`)

**Distribution surfaces:**

| Surface | Mechanism | Trust boundary |
|---------|-----------|----------------|
| CLI binary `tokmd` | Local execution, package managers (cargo, brew, AUR, Docker) | User → tool |
| `tokmd-core` Rust library | Crates.io, source builds | Library user → tool |
| `tokmd-python` (PyO3) | PyPI | Python user → tool |
| `tokmd-node` (napi-rs) | npm | JS user → tool |
| `tokmd-wasm` (wasm-bindgen) | npm (browser/worker) | Web user → tool |
| GitHub Action `action.yml` | GitHub Marketplace | CI pipeline → tool |
| Sensor envelope output | CI artifact consumption | CI → downstream consumers |

## 2. Trust Boundaries

| ID | Boundary | Crossed by |
|----|----------|------------|
| TB-1 | User shell → CLI argv | CLI flags, paths, globs |
| TB-2 | Untrusted repository contents → tokmd file walker | Filesystem, symlinks, `.gitignore` |
| TB-3 | tokmd → external `git` binary | `git log`, `git diff`, `git rev-parse` (via `std::process::Command`) |
| TB-4 | tokmd → external `cargo` binary | `cargo audit` (cockpit supply-chain gate) |
| TB-5 | Python/Node → tokmd FFI | JSON-string args to `run_json()` |
| TB-6 | CI workflow → repository secrets | `secrets.*` (e.g., `MINIMAX_API_KEY`, `FACTORY_API_KEY`) |
| TB-7 | Scanner → output destination | User-specified `--output`, `--bundle-dir`, `--log` paths |
| TB-8 | Untrusted inputs (paths) → inclusion policy | `--paths`, `--include`, in-memory `inputs[]` |
| TB-9 | In-memory mode → boundary path canonicalization | ReDoS on path normalization |
| TB-10 | WASM host → in-memory filesystem | Sandboxed JS environment |

## 3. STRIDE Analysis

### Spoofing (S)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|---------------|
| Fake git history impersonation | LOW | Git is invoked as a subprocess; tokmd reads stdout, not metadata. No trust placed in commit author fields beyond display. | Author field displayed in receipts; do not gate on it. |
| Impersonation via FFI arg fields | LOW | Strict JSON parsing with explicit type validation in `tokmd-core/src/ffi/parse.rs` rejects type mismatches. | Low — caller controls input. |
| GitHub Action impersonation | LOW | Actions pinned by SHA (`actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2`). | Low — supply chain still trusted. |

**Result:** Low residual risk. No `medium` or higher spoofing vectors.

### Tampering (T)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|---------------|
| Unsanitized git refs becoming shell args | HIGH (if reached) → MITIGATED | `tokmd-git/src/command.rs` uses `Command::new("git").arg()` (not shell). `tokmd-git/src/refs.rs::env_base_ref_is_safe` rejects `--`, whitespace, control, and `\\` in refs. `--end-of-options` separator used. `GIT_REPO_SHAPING_ENV` is `env_remove`'d. | LOW — defense in depth applied. |
| Path traversal via scan roots | HIGH (if reached) → MITIGATED | `tokmd-scan/src/path/bounded_path.rs` rejects empty paths, `..` segments, absolute paths. `ValidatedRoot` requires canonicalization. `BoundedPath::existing_relative` enforces `ensure_under_root`. | LOW — tested. |
| Path traversal via FFI in-memory inputs | HIGH (if reached) → MITIGATED | `tokmd-core/src/ffi/inputs.rs::validate_in_memory_input_path` rejects: empty, >4096 bytes, control chars, leading `/` or `\\`, Windows drive prefixes, `..` segments, all-`.` paths. | LOW — exhaustive rejection. |
| File content write at attacker-chosen path | MEDIUM (if reached) → MITIGATED | `tokmd/src/commands/run.rs` writes to user-provided `--output-dir` or `.runs/tokmd/<id>/` (no traversal). `--output`, `--bundle-dir`, `--log` are added to exclude patterns to prevent recursive read. | LOW — output dirs are user-chosen. |
| Cargo lockfile tampering | MEDIUM | `tokmd-cockpit/src/supply_chain.rs` invokes `cargo audit --json` and parses structured output. `parse_audit_output` tolerates malformed JSON by returning `Pending`, never `Pass`. | LOW — supply-chain evidence is informational. |
| Workspace lint bypass via file mutation | LOW | `cargo xtask lint-fix` and `cargo xtask gate --check` run as pre-push hooks. CI requires both. | LOW. |
| Injection into .tokeignore / config files | LOW | `.tokeignore` patterns are passed to `tokei`/`ignore` crate, not eval'd. `ConfigMode::Auto` reads tokei config files; `ConfigMode::None` skips. | LOW. |

**Result:** All `high`/medium tampering vectors are mitigated with defense in depth.

### Repudiation (R)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|---------------|
| "I never ran that scan" claims | LOW | Receipts include `schema_version`, `generated_at_ms`, scan args, and tool metadata. JSON outputs are signed by inclusion in audit pipelines. | LOW — receipts are not cryptographically signed. |
| Audit log tampering | LOW | `cargo xtask gate --check` is the gate; receipts are deterministic. | LOW. |

**Result:** LOW residual risk. Receipts are auditable but not signed.

### Information Disclosure (I)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|
| Secrets in redacted output | MEDIUM (if reached) → MITIGATED | `tokmd-format/src/redact/mod.rs` uses BLAKE3, 16-char prefix, with extension-preserving allowlist (`tokmd-format/src/redact/extensions.rs`). Reverts to bare hash for unknown/unsafe compound suffixes. | LOW — non-allowlisted extensions get bare hash. |
| Path disclosure via receipts | LOW | `--redact=paths|all` mode available. Receipts do not include file contents by default (only counts). | LOW. |
| File content disclosure via `--content` analysis | MEDIUM (if reached) → MITIGATED | `tokmd-analysis/src/content/mod.rs` uses `ContentLimits` (default `max_file_bytes=128KiB`, total `max_bytes`) to bound read. `is_text_like` check skips binary blobs. | LOW — content is read once into memory. |
| Environment variable leakage into receipts | MEDIUM (if reached) → MITIGATED | `tokmd-git/src/command.rs` strips `GIT_DIR`, `GIT_WORK_TREE`, `GIT_SSH`, `GIT_SSH_COMMAND`, `GIT_ASKPASS`, `GIT_PAGER`, `GIT_EDITOR`, `GIT_PROXY_COMMAND`, `GIT_EXTERNAL_DIFF` before spawning git. | LOW — env is not propagated to receipts. |
| Stderr exposure of repository internals | LOW | `git log` runs with `Stdio::null()` for stderr; only stdout is parsed. | LOW. |
| Information disclosure via error messages | LOW | Errors return `TokmdError` enum with typed codes; internal paths not exposed. | LOW. |

**Result:** All `medium`/high I-disclosure vectors mitigated.

### Denial of Service (D)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|------------|
| Pathological regex via exclude patterns | MEDIUM (if reached) → MITIGATED | `tokmd-scan/src/exclude/` normalizes patterns. `tokmd-scan/src/walk/` uses bounded traversal. `tokmd-scan/src/lib.rs::scan` enforces `max_commits` / `max_commit_files` for git history. | LOW. |
| ReDoS in path normalization | LOW | `tokmd-format/src/redact/mod.rs::clean_path` uses bounded loops with `String::replace` for `/./`. Bounded by input length. | LOW. |
| Resource exhaustion via large git history | MEDIUM (if reached) → MITIGATED | `tokmd-git/src/lib.rs::collect_history` honors `max_commits` and `max_commit_files`. Streaming `BufReader` does not load full history. | LOW. |
| Resource exhaustion via large file read | MEDIUM (if reached) → MITIGATED | `tokmd-analysis/src/content/mod.rs` enforces `DEFAULT_MAX_FILE_BYTES = 128 KiB` and total `max_bytes` limit. | LOW. |
| Pathological FFI JSON payload | LOW | `serde_json::from_str` is recursive; no DoS-specific bounds. The 4 KiB path limit bounds `inputs[]`; arbitrary JSON `args` size is not bounded by the FFI layer itself. | MEDIUM — large-but-valid JSON could spike memory. Observed but not at severity threshold for a finding. |
| Fuzz target coverage | LOW | 9 fuzz targets in `fuzz/` directory. | LOW. |

**Result:** All `medium`/high DoS vectors mitigated or have explicit resource limits. One observation noted but not at severity threshold.

### Elevation of Privilege (E)

| Threat | Severity | Mitigation | Residual risk |
|--------|----------|------------|------------|
| Code execution via git hooks | MEDIUM (if reached) → MITIGATED | `GIT_REPO_SHAPING_ENV` (`GIT_DIR`, `GIT_SSH`, `GIT_SSH_COMMAND`, `GIT_ASKPASS`, `GIT_PAGER`, `GIT_EDITOR`, `GIT_PROXY_COMMAND`, `GIT_EXTERNAL_DIFF`) is `env_remove`'d before every git subprocess. | LOW. |
| Code execution via Python `pyo3` bindings | MEDIUM (if reached) → MITIGATED | `tokmd-python/src/lib.rs` documents FFI safety invariants. `?` operator used for error propagation. `.expect()` prohibited in production. GIL released via `py.detach()` for long scans. | LOW. |
| Code execution via `Command::new` arg construction | HIGH (if reached) → MITIGATED | All `Command::new` invocations use `arg()` (not `args(&[user_string])` with shell metacharacters). No use of `sh -c` or `bash -c`. Verified across `tokmd-git`, `tokmd-cockpit`. | LOW. |
| Privilege escalation via supply chain | MEDIUM | `Cargo.lock` is committed. `deny.toml` enforces advisory check (cargo-deny). `RUSTSEC-2020-0163` (transitive `term_size`) is documented as an upstream limitation. | MEDIUM — transitive advisory remains. Observed but not at severity threshold for a finding. |
| Code execution via WASM host | MEDIUM (if reached) → MITIGATED | `tokmd-wasm` uses `MemFs` (no host fs). Sandboxed by the WASM runtime. | LOW. |

**Result:** All `high` elevation vectors mitigated. One supply-chain observation noted.

## 4. Out-of-Scope

- Issues in third-party crates (report upstream; not actionable here).
- Theoretical attacks without realistic exploitation paths.
- Performance regressions that are not denial-of-service.
- The `home` crate vendored at `vendor/home-0.5.12` (intentional temporary patch — `Cargo.toml` `[patch.crates-io]`).

## 5. Standing Defenses

These defenses are baked into the workspace and should not regress:

1. **Workspace lints** (`Cargo.toml`): `unsafe_code = "forbid"`, `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`, `unreachable = "deny"`, `dbg_macro = "deny"`, `unimplemented = "deny"`, `todo = "deny"`, plus a long list of correctness lints (numerical, file/process/path, async, etc.).
2. **Git subprocess isolation**: `git_cmd()` constructor strips `GIT_REPO_SHAPING_ENV`.
3. **FFI input validation**: `validate_in_memory_input_path` rejects every known bypass path.
4. **Bounded path handling**: `BoundedPath` enforces canonical under-root invariant.
5. **Strict JSON parsing**: `parse.rs` rejects type mismatches (no silent fallback to defaults).
6. **Receipt schema versioning**: Per-family constants in `tokmd-types` (`SCHEMA_VERSION`, `COCKPIT_SCHEMA_VERSION`, etc.) — see `CLAUDE.md`.
7. **Pinned GitHub Actions**: All third-party actions pinned by SHA with a comment naming the upstream version.
8. **License allowlist + advisory check**: `deny.toml` configures `cargo-deny`.
9. **Branch protection policy**: the checked-in settings require 1 PR approval with CODEOWNERS review and the `Tokmd Rust Result` status check. Live protection is repository-specific control-plane state and must be queried separately; the settings file is not proof that public `main` is currently protected.
10. **Constrained merge policy**: normal swarm and public development PRs use squash merges; the publication repository also permits deliberate two-parent merge commits for swarm imports so the shared graph is preserved. Direct pushes, unrelated merge commits, and force-pushes remain prohibited by operating policy and branch controls. `allow_merge_commit = true` and `required_linear_history = false` are topology requirements for that import path, not standing security defenses.

## 6. Review Cadence

- **Regenerate when:** architecture changes, new external surface added, new subprocess invocation, or 90 days since last review.
- **Source:** weekly `droid-security-scan` workflow (`.github/workflows/droid-security-scan.yml`).
- **Owner:** EffortlessMetrics/tokmd-swarm maintainers.
