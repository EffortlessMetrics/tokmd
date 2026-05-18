# Threat Model for tokmd

**Last Updated:** 2026-05-18
**Version:** 1.0.0
**Methodology:** STRIDE + Natural Language Analysis

---

## 1. System Overview

### Architecture Description

tokmd is a Rust CLI tool and library that wraps the `tokei` library to generate code inventory receipts and derived analytics of code repositories. It scans source code, counts lines by language, and produces outputs in JSON/JSONL/CSV/Markdown/TSV formats. The system is built entirely in Rust using a tiered microcrate workspace architecture.

The project ships multiple product surfaces over the same deterministic receipt model:

1. **CLI binary** (`tokmd`) - Command-line interface with subcommands (lang, module, export, run, analyze, badge, diff, cockpit, gate, tools, context, init, check-ignore, completions)
2. **Rust Library** (`tokmd-core`) - Facade with workflow functions for embedding
3. **FFI Layer** (`tokmd-core::ffi`) - JSON-based FFI entrypoint via `run_json(mode, args_json)`
4. **Python Bindings** (`tokmd-python`) - PyO3-based Python module
5. **Node.js Bindings** (`tokmd-node`) - napi-rs based async bindings
6. **WASM Bindings** (`tokmd-wasm`) - wasm-bindgen for browser/worker environments

### Crate Hierarchy

| Tier | Crates | Purpose |
|------|--------|---------|
| 0 (Contracts) | `tokmd-types`, `tokmd-analysis-types`, `tokmd-settings`, `tokmd-envelope`, `tokmd-io-port` | Pure data DTOs, no business logic |
| 1 (Core) | `tokmd-scan`, `tokmd-model`, `tokmd-sensor` | Tokei wrapper, aggregation, sensor substrate |
| 2 (Adapters) | `tokmd-format`, `tokmd-git` | Output rendering, git history analysis |
| 3 (Orchestration) | `tokmd-analysis`, `tokmd-cockpit`, `tokmd-gate` | Analysis presets, PR metrics, policy evaluation |
| 4 (Facade) | `tokmd-core` | Library facade with FFI layer |
| 5 (Products) | `tokmd`, `tokmd-python`, `tokmd-node`, `tokmd-wasm` | CLI and language bindings |

### Key Components

| Component | Purpose | Security Criticality | Attack Surface |
|-----------|---------|---------------------|----------------|
| `tokmd-core::ffi::run_json` | Single JSON entrypoint for all modes | HIGH | FFI boundary, JSON parsing |
| `tokmd-scan` | Filesystem traversal and tokei scanning | HIGH | Path handling, symlink resolution |
| `tokmd-format` | Output rendering (JSON/XML/CSV export) | MEDIUM | Content injection in outputs |
| `tokmd-git` | Git history via shell `git log` | HIGH | Command injection in git arguments |
| `tokmd-python` | Python FFI with GIL management | HIGH | Exception propagation, GIL safety |
| `tokmd-node` | Node.js async bindings | HIGH | Promise rejection, JSON serialization |
| `tokmd-wasm` | Browser WASM bindings | MEDIUM | JS value boundary, JSON parsing |
| `tokmd-envelope` | Response envelope parsing/extraction | MEDIUM | JSON envelope validation |

### Data Flow

```
User Input (CLI/FFI/Python/Node/WASM)
         │
         ▼
┌─────────────────────────────────────────────────────┐
│  tokmd-core::ffi::run_json(mode, args_json)          │
│  - Parse JSON arguments                             │
│  - Validate mode (lang/module/export/analyze/diff)  │
│  - Dispatch to workflow                             │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│  tokmd-scan (Tier 1)                                │
│  - Walk filesystem with ignore patterns             │
│  - Bounded path validation (no traversal escape)    │
│  - Execute tokei for LOC counting                   │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│  tokmd-model (Tier 1)                               │
│  - Aggregate tokei results                          │
│  - BTreeMap for deterministic ordering              │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│  tokmd-format (Tier 2)                             │
│  - Render JSON/JSONL/CSV/Markdown/TSV/CycloneDX     │
│  - Redaction (BLAKE3 path hashing)                  │
└─────────────────────────────────────────────────────┘
         │
         ▼
Response Envelope: {"ok": true, "data": {...}} or {"ok": false, "error": {...}}
```

---

## 2. Trust Boundaries & Security Zones

### Trust Boundary Definition

tokmd has **2 primary trust zones**:

1. **External Zone** (Untrusted) - User input from all binding surfaces
   - CLI arguments and file paths
   - JSON arguments via FFI
   - Python function arguments
   - Node.js function arguments
   - WASM JS value arguments
   - Assumes: All input is potentially malicious
   - Entry Points: All public API functions accepting user data

2. **Internal Zone** (Trusted) - Rust processing after validation
   - Tokei library execution
   - In-memory data aggregation
   - Receipt construction
   - Assumes: Input has been validated at boundary

### Authentication & Authorization

tokmd does not implement authentication or authorization. It operates on local filesystems only.

**Critical Security Controls:**

- **Path Bounding**: All filesystem access is constrained to user-specified roots via `canonicalize_bounded_path()` which resolves symlinks and rejects paths that escape the scan root
- **Input Validation**: JSON parsing with strict schema validation at FFI boundary
- **Error Propagation**: All errors propagate via structured `TokmdError` with error codes, never panicking across FFI
- **GIL Safety**: Python bindings release GIL during long operations
- **Deterministic Output**: BTreeMap and stable sorting prevent timing-based information leaks

---

## 3. Attack Surface Inventory

### External Interfaces

#### CLI Commands

| Command | Input | Validation | Risk |
|---------|-------|------------|------|
| `tokmd lang` | Paths, glob patterns | Path bounding, ignore patterns | Path traversal |
| `tokmd module` | Paths, module roots | Path bounding | Path traversal |
| `tokmd export` | Paths, format (json/jsonl/csv/cyclonedx) | Path bounding, output format | Path traversal, XXE in XML |
| `tokmd run` | Paths, output options | Path bounding | Path traversal |
| `tokmd analyze` | Paths, preset, git flags | Path bounding | Path traversal |
| `tokmd diff` | Two paths or receipt files | Path bounding | Path traversal |
| `tokmd cockpit` | Git refs, baseline | Git range validation | Git injection |
| `tokmd gate` | Policy file, receipt | JSON pointer rules | JSON injection |
| `tokmd context` | Paths, token budget | Path bounding | Path traversal |
| `tokmd badge` | SVG template, lang input | Input sanitization | XSS in SVG |
| `tokmd tools` | Format (openai/anthropic/schema) | None | None |
| `tokmd init` | None | None | None |
| `tokmd check-ignore` | Paths | Path bounding | None |
| `tokmd completions` | Shell name | None | None |

#### FFI Entry Points

- **`run_json(mode: &str, args_json: &str) -> String`** (tokmd-core)
  - Input: Mode string, JSON arguments
  - Validation: Mode whitelist, JSON schema validation
  - Risk: Invalid JSON, unknown mode, injection

#### Python Bindings (tokmd-python)

- **`lang(paths, top, files, children, redact, excluded, hidden)`**
- **`module(paths, top, module_roots, module_depth, ...)`**
- **`export(paths, format, min_code, max_rows, ...)`**
- **`analyze(paths, preset, window, git, ...)`**
- **`diff(from_path, to_path)`**
- **`cockpit(base, head, range_mode, baseline)`**
  - Validation: Each argument validated before release from GIL
  - Risk: Exception translation, GIL deadlock

#### Node.js Bindings (tokmd-node)

- Same API surface as Python, all async (returns Promises)
- Uses `tokio::task::spawn_blocking()` for non-blocking event loop
- Risk: Promise rejection handling, JSON serialization

#### WASM Bindings (tokmd-wasm)

- **`run_json(mode, args_json)`** - Raw JSON string
- **`run(mode, args)`** - JS object to WASM
- **`run_lang(args)`, `run_module(args)`, `run_export(args)`**, **`run_analyze(args)`**
- **`capabilities()`** - Returns supported modes
- Validation: JS value to JSON conversion, preset validation for rootless mode
- Risk: JS value boundary, JSON serialization errors

### Data Input Vectors

1. **Filesystem paths** - User-specified directories to scan
2. **Git refs** - Branch/tag names for diff and cockpit commands
3. **JSON configuration** - Settings passed via FFI
4. **Environment variables** - Git executable location, home directory
5. **Shell completions** - Generated but not executed

---

## 4. Critical Assets & Data Classification

### Data Classification

#### PII

- **None collected** - tokmd is a code inventory tool that processes source files
- No user accounts, no personal data
- No network transmission of user data

#### Credentials & Secrets

- **Git credentials** - If present on filesystem, git operations may read them
- **Home directory paths** - Used for config file discovery via `home` crate
- **PATH environment** - Used to locate `git` executable

#### Business-Critical Data

- **Source code content** - Read during analysis but only metadata extracted
- **Git history** - Analyzed for churn metrics
- **Receipt outputs** - JSON/JSONL/CSV containing code statistics

---

## 5. Threat Analysis (STRIDE Framework)

### S - Spoofing Identity

#### Threat: FFI Mode String Injection

**Scenario:** An attacker passes a malicious mode string to `run_json()` that causes unexpected behavior or information disclosure.

**Vulnerable Components:**
- `tokmd-core/src/ffi/mod.rs` - `run_json_inner()`
- `tokmd-core/src/ffi/modes.rs` - Mode dispatch

**Attack Vector:**
1. Attacker calls `run_json("../../../etc/passwd", "{}")`
2. Mode validation may be insufficient
3. Error handling exposes file system structure

**Code Pattern to Look For:**
```rust
// SAFE: Mode is validated against whitelist
let valid_modes = ["lang", "module", "export", "analyze", "diff", "version"];
if !valid_modes.contains(mode) {
    return Err(TokmdError::unknown_mode(mode));
}
```

**Existing Mitigations:**
- Mode is validated against known modes in `modes.rs`
- Unknown mode returns `TokmdError::UnknownMode`

**Gaps:**
- None identified; mode validation is in place

**Severity:** MEDIUM | **Likelihood:** LOW

---

#### Threat: Path Identity Spoofing via Symlinks

**Scenario:** A malicious repository contains symlinks that escape the scan root, potentially causing tokmd to read files outside the intended directory.

**Vulnerable Components:**
- `tokmd-scan/src/path/mod.rs` - `canonicalize_bounded_path()`
- `tokmd-scan/src/walk/mod.rs` - Filesystem walker

**Attack Vector:**
1. Attacker creates a repo with symlink: `ln -s /etc/passwd repo/leaked`
2. User runs `tokmd lang repo/`
3. Without bounding, would read `/etc/passwd`
4. Bounded path rejects symlinks that escape root

**Code Pattern to Look For:**
```rust
// SAFE: Canonicalize and verify path stays under root
fn canonicalize_bounded_path(root: impl AsRef<Path>, relative: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let root = ValidatedRoot::new(root)?;
    let bounded = BoundedPath::existing_relative(&root, relative.as_ref())?;
    Ok(bounded.canonical().to_path_buf())
}

// In BoundedPath::canonical():
// if !canonical_path.starts_with(&self.root().canonical()) {
//     return Err(PathViolation::EscapesRoot(...).into());
// }
```

**Existing Mitigations:**
- `canonicalize_bounded_path()` resolves symlinks and verifies they don't escape
- `normalize_bounded_rel_path()` rejects parent traversal (`..`) at lexical level
- Tests verify symlink escape rejection

**Gaps:**
- None identified for filesystem access

**Severity:** HIGH | **Likelihood:** LOW (mitigations in place)

---

### T - Tampering with Data

#### Threat: CycloneDX XML Output Tampering

**Scenario:** Export to CycloneDX XML format could contain unsanitized file paths, allowing XML injection if paths contain special characters.

**Vulnerable Components:**
- `tokmd-format/src/export/cyclonedx.rs`

**Attack Vector:**
1. Files have paths like `"><script>alert(1)</script>.rs`
2. Exported as CycloneDX XML
3. If XML is served in browser without encoding, XSS possible

**Code Pattern to Look For:**
```rust
// VULNERABLE: Raw path in XML without escaping
write_element!("component", "name", &file_row.path)?;

// SAFE: XML entity escaping
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
```

**Existing Mitigations:**
- XML library handles escaping automatically
- CycloneDX is typically machine-consumed, not browser-rendered

**Gaps:**
- Verify XML library handles all edge cases

**Severity:** MEDIUM | **Likelihood:** LOW

---

#### Threat: CSV Injection via File Paths

**Scenario:** CSV export contains file paths that could contain formula injection characters (`=`, `+`, `-`, `@`).

**Vulnerable Components:**
- `tokmd-format/src/export/csv.rs`

**Attack Vector:**
1. File path: `=cmd|' /C calc'!A0`
2. CSV opened in Excel
3. Formula executed on open

**Code Pattern to Look For:**
```rust
// VULNERABLE: Raw CSV field without escaping
writeln!(w, "{},{},{}", path, code, blank)?;

// SAFE: CSV field quoting/escaping
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.starts_with('=') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
```

**Existing Mitigations:**
- Need to verify CSV export handles formula injection

**Gaps:**
- Should prefix paths starting with `=` or `@` with single quote

**Severity:** MEDIUM | **Likelihood:** MEDIUM

---

#### Threat: Git Argument Injection

**Scenario:** User-controlled git ref arguments could inject extra git commands or flags.

**Vulnerable Components:**
- `tokmd-git/src/command.rs` - Git command execution
- `tokmd-git/src/lib.rs` - Git operations

**Attack Vector:**
1. User provides ref: `--base=main; cat /etc/passwd`
2. Git command: `git log main; cat /etc/passwd..HEAD`
3. If shell execution is used, arbitrary command runs

**Code Pattern to Look For:**
```rust
// SAFE: Git args passed as array, not shell string
Command::new("git")
    .args(["log", "--format=full", "--follow"])
    .arg(format!("{}..{}", base, head))
    .output()

// VULNERABLE: Shell string interpolation
let cmd = format!("git log {}..{}", base, head);
std::process::Command::new("sh").arg("-c").arg(&cmd)
```

**Existing Mitigations:**
- Git operations use `std::process::Command` with array args
- No shell interpolation

**Gaps:**
- None identified

**Severity:** CRITICAL | **Likelihood:** LOW (proper API usage confirmed)

---

### R - Repudiation

#### Threat: Missing Audit Trail for Scans

**Scenario:** Users could deny scanning certain repositories because tokmd doesn't log operations by default.

**Vulnerable Components:**
- `tokmd` CLI binary
- `tokmd-core` workflows

**Attack Vector:**
1. User scans private repo
2. No logs generated by default
3. Cannot prove what was scanned

**Code Pattern to Look For:**
- No logging implementation found in core

**Existing Mitigations:**
- Receipts include `generated_at_ms` timestamp
- Schema version in all outputs provides provenance

**Gaps:**
- No command-level audit logging
- No correlation between CLI invocations and receipts

**Severity:** LOW | **Likelihood:** LOW (tool nature, not security critical)

---

### I - Information Disclosure

#### Threat: Path Redaction Bypass

**Scenario:** Redaction mode should hash file paths to hide sensitive directory names, but implementation bugs could leak paths.

**Vulnerable Components:**
- `tokmd-format/src/redact/mod.rs` - Redaction logic
- `tokmd-format/src/redact/extensions.rs` - Path hashing

**Attack Vector:**
1. User requests redaction with `redact=paths`
2. Output should show only hashed paths
3. Bug causes original paths to appear

**Code Pattern to Look For:**
```rust
// SAFE: BLAKE3 hash of path with truncation
use blake3::Hasher;
fn redact_path(path: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(path.as_bytes());
    let hash = hasher.finalize();
    format!("{:.8}", hash.to_hex())
}
```

**Existing Mitigations:**
- Uses BLAKE3 for cryptographic hashing
- Redaction is deterministic (same input = same hash)

**Gaps:**
- None identified

**Severity:** MEDIUM | **Likelihood:** LOW

---

#### Threat: Sensitive Data in Error Messages

**Scenario:** Error messages might expose sensitive paths or system information.

**Vulnerable Components:**
- `tokmd-core/src/error.rs` - Error formatting
- `tokmd-envelope/src/ffi.rs` - Error message formatting

**Attack Vector:**
1. User provides invalid path
2. Error message echoes the path back
3. If path contains sensitive info, leaks in error output

**Code Pattern to Look For:**
```rust
// SAFE: Error includes path in message
return Err(TokmdError::path_not_found_with_suggestions(&path));

// BUT: Error messages should not leak system internals
// Check: Does error message expose .git directory structure?
```

**Existing Mitigations:**
- Error codes are structured, messages are user-facing
- Path-not-found errors include the path for user benefit

**Gaps:**
- User benefits from path in error message
- Trade-off between debuggability and information hiding

**Severity:** LOW | **Likelihood:** LOW

---

#### Threat: WASM Browser Memory Exposure

**Scenario:** WASM module running in browser could expose memory contents if not properly isolated.

**Vulnerable Components:**
- `tokmd-wasm/src/lib.rs`
- Browser JavaScript runtime

**Attack Vector:**
1. Malicious page loads tokmd WASM
2. Attacker accesses WASM memory directly
3. Could read uninitialized memory

**Code Pattern to Look For:**
- WASM memory is isolated by browser security model
- No `unsafe` code in WASM bindings (`#![forbid(unsafe_code)]`)

**Existing Mitigations:**
- `#![forbid(unsafe_code)]` in tokmd-wasm
- WASM memory model provides isolation
- No `web-sys` bindings to DOM access

**Gaps:**
- WASM binary itself could contain secrets in strings
- Review: Are any secrets compiled into WASM binary?

**Severity:** MEDIUM | **Likelihood:** LOW

---

### D - Denial of Service

#### Threat: Pathological Git History

**Scenario:** Repository with extremely large git history causes tokmd to hang or exhaust memory.

**Vulnerable Components:**
- `tokmd-git/src/command.rs` - Git log parsing
- `tokmd-cockpit/src/gates/diff_coverage/` - Coverage gate

**Attack Vector:**
1. Attacker provides repo with 1M+ commits
2. `git log` returns massive output
3. tokmd buffers entire output in memory
4. OOM or hang

**Code Pattern to Look For:**
```rust
// VULNERABLE: Read entire git output into memory
let output = Command::new("git")
    .args(["log", "--format=full", &format!("{}..{}", base, head)])
    .output()?;
let git_output = String::from_utf8_lossy(&output.stdout);

// SAFE: Stream processing with limits
// But tokei output is inherently bounded by file count
```

**Existing Mitigations:**
- Git history is parsed but only metadata extracted
- No streaming parser found for git output
- `max_commits` setting limits git scanning

**Gaps:**
- No hard limits on git output size
- Consider adding `--max-commits` enforcement

**Severity:** HIGH | **Likelihood:** MEDIUM

---

#### Threat: Malformed CycloneDX XML

**Scenario:** CycloneDX output could be malformed XML if input contains unusual characters.

**Vulnerable Components:**
- `tokmd-format/src/export/cyclonedx.rs`

**Attack Vector:**
1. File contains null bytes or invalid UTF-8
2. XML serializer fails
3. Output is truncated or invalid

**Code Pattern to Look For:**
- XML library handles encoding
- Null bytes in file paths could cause issues

**Existing Mitigations:**
- Path normalization ensures UTF-8
- XML library (quick-xml or similar) handles encoding

**Gaps:**
- None identified

**Severity:** LOW | **Likelihood:** LOW

---

#### Threat: Recursive Symbolic Links

**Scenario:** Repository contains recursive symlinks causing infinite traversal.

**Vulnerable Components:**
- `tokmd-scan/src/walk/mod.rs` - Filesystem walker
- `ignore` crate for traversal

**Attack Vector:**
1. Create recursive symlink: `mkdir a; ln -s a b/a`
2. Walk traverses forever
3. CPU spins, disk fills with path cache

**Code Pattern to Look For:**
- `ignore` crate should detect cycles
- Need to verify cycle detection

**Existing Mitigations:**
- `ignore` crate used for traversal (handles cycles)
- Path bounding adds secondary protection

**Gaps:**
- None identified

**Severity:** MEDIUM | **Likelihood:** LOW

---

#### Threat: JSON/JSONL Output Size

**Scenario:** Massive repository produces enormous JSON output causing memory exhaustion.

**Vulnerable Components:**
- `tokmd-format/src/export/json.rs`
- `tokmd-format/src/export/jsonl.rs`

**Attack Vector:**
1. User scans repo with 10M files
2. JSON output is 10GB
3. tokmd OOMs trying to serialize

**Code Pattern to Look For:**
```rust
// VULNERABLE: Buffer entire JSON in memory
let mut buf = Vec::new();
let mut ser = Serializer::new(&mut buf);
receipt.serialize(&mut ser)?;
std::fs::write("output.json", &buf)?;

// SAFE: Stream JSONL line by line
// But receipt is a single JSON object
```

**Existing Mitigations:**
- `ExportReceipt` is inherently bounded by file count
- `max_rows` setting limits output
- `min_code` filter reduces output size

**Gaps:**
- No hard limit on JSON output size
- Consider streaming for large outputs

**Severity:** MEDIUM | **Likelihood:** LOW

---

### E - Elevation of Privilege

#### Threat: FFI Panics Crash Host Process

**Scenario:** Rust panic crosses FFI boundary and crashes Python/Node process.

**Vulnerable Components:**
- `tokmd-python/src/lib.rs`
- `tokmd-node/src/lib.rs`
- `tokmd-core/src/ffi/mod.rs`

**Attack Vector:**
1. tokmd encounters unexpected input
2. Rust code panics
3. Panic propagates across FFI
4. Python interpreter crashes

**Code Pattern to Look For:**
```rust
// SAFE: No panics across FFI, errors return Result
#[no_mangle]
pub extern "C" fn run_json(mode: *const c_char, args: *const c_char) -> *mut c_char {
    // Use Result<T, E> and convert to error envelope
    // Never let panic unwind
}

// In Python bindings:
#[cfg_attr(not(test), pyfunction)]
fn lang(...) -> PyResult<Py<PyAny>> {
    // All operations use ? operator for error propagation
    // No .expect(), .unwrap(), or panics
}
```

**Existing Mitigations:**
- All FFI boundaries use `Result` return, no panics
- Python bindings explicitly prohibit `.expect()` in production
- `tokmd-core` workspace lints: `panic = "deny"`, `unwrap_used = "deny"`
- Extensive documentation of FFI safety invariants

**Gaps:**
- None identified; strict panic policy

**Severity:** CRITICAL | **Likelihood:** LOW (mitigations comprehensive)

---

#### Threat: Python GIL Not Released During Long Operations

**Scenario:** Python bindings hold GIL during long scan, blocking other Python threads.

**Vulnerable Components:**
- `tokmd-python/src/runtime.rs` - GIL management

**Attack Vector:**
1. Python calls `tokmd.lang()`
2. Scan takes 10 minutes
3. GIL held entire time
4. Other Python threads hang

**Code Pattern to Look For:**
```rust
// SAFE: GIL released before long operation
#[pyfunction]
fn lang(...) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        // Build args with GIL
        let args = build_args(py, ...)?;
        // Release GIL for long operation
        let result = std::thread::spawn(move || {
            run_blocking(|| tokmd_core::ffi::run_json("lang", &args_json))
        }).join()?;
        // GIL reacquired for return
    })
}
```

**Existing Mitigations:**
- Runtime uses `std::thread::spawn` for blocking operations
- GIL released during scan
- PyO3 handles GIL management

**Gaps:**
- None identified

**Severity:** MEDIUM | **Likelihood:** LOW

---

## 6. Vulnerability Pattern Library

### How to Use This Section

This section contains code patterns specific to tokmd's Rust implementation. When analyzing code for security issues, look for these patterns and verify mitigations are in place.

---

### Path Traversal Patterns

```rust
// PATTERN 1: User-controlled path without bounding
fn scan(path: &str) -> Result<Languages> {
    tokei::Languages::from_path(path)  // VULNERABLE: No bounds check
}

// SAFE ALTERNATIVE:
fn scan_bounded(root: &Path, relative: &Path) -> Result<Languages> {
    let bounded = canonicalize_bounded_path(root, relative)?;
    tokei::Languages::from_path(&bounded)
}
```

**tokmd Implementation:** `tokmd-scan/src/path/mod.rs::canonicalize_bounded_path()`

---

### Command Injection Patterns

```rust
// PATTERN 1: Shell string interpolation
let cmd = format!("git log {}..{}", base, head);
std::process::Command::new("sh").arg("-c").arg(&cmd);  // VULNERABLE

// PATTERN 2: Array args (SAFE)
std::process::Command::new("git")
    .args(["log", &format!("{}..{}", base, head)])  // SAFE
    .output()?;
```

**tokmd Implementation:** `tokmd-git/src/command.rs` uses array args

---

### JSON Injection Patterns

```rust
// PATTERN 1: User string in JSON without escaping
fn to_json_error(code: &str, msg: &str) -> String {
    format!(r#"{{"error": {{"code": "{}", "message": "{}"}}}}"#, code, msg)
    // VULNERABLE: msg could contain "}}
}                                              //

// SAFE ALTERNATIVE:
fn to_json_error(code: &str, msg: &str) -> String {
    serde_json::json!({
        "error": {
            "code": code,
            "message": msg
        }
    }).to_string()
}
```

**tokmd Implementation:** `tokmd-core/src/error.rs` uses `serde_json::json!()` macro

---

### FFI Panic Patterns

```rust
// PATTERN 1: Unchecked .unwrap() across FFI
#[no_mangle]
pub extern "C" fn run_json(args: *const c_char) -> *mut c_char {
    let args = std::ffi::CStr::from_ptr(args).to_str().unwrap()  // VULNERABLE: panic
}

// SAFE ALTERNATIVE:
#[no_mangle]
pub extern "C" fn run_json(args: *const c_char) -> *mut c_char {
    let args = match std::ffi::CStr::from_ptr(args).to_str() {
        Ok(s) => s,
        Err(_) => return error_json("invalid_utf8"),
    }
}
```

**tokmd Implementation:** All FFI uses `Result` types, workspace lint `panic = "deny"`

---

### XXE in XML Generation

```rust
// PATTERN 1: XML generation without entity escaping
fn write_xml_field(w: &mut Writer, name: &str, value: &str) {
    write_element!(w, name, value);  // VULNERABLE if value contains < > & "
}

// SAFE ALTERNATIVE:
// Use XML library's built-in escaping
let mut xml = XmlWriter::new();
xml.write_text_content(name, TextType::Escaped, value)?;
```

**tokmd Implementation:** CycloneDX uses serde-based XML serialization which handles escaping

---

## 7. Security Testing Strategy

### Automated Testing

| Tool | Purpose | Frequency |
|------|---------|-----------|
| `cargo clippy` | Static analysis, lints | Every commit |
| `cargo test` | Unit and integration tests | Every commit |
| `cargo xtask gate` | Full quality gate | Pre-commit |
| `cargo audit` | Vulnerability scanning in deps | Daily (CI) |
| `cargo outdated` | Outdated dependencies | Weekly |
| Mutation testing (`cargo-mutants`) | Test quality | PR review |

### Manual Security Reviews

Human review is required for:

- HIGH/CRITICAL findings from automated scans
- Changes to FFI boundary code (Python, Node, WASM bindings)
- Changes to path handling logic
- Changes to git command execution
- New binding surface additions
- Changes to error handling that could leak information

### Specific Security Tests to Implement

1. **Path Traversal Test Suite**
   - Symlink escape attempts
   - Parent traversal (`../`) in paths
   - Absolute path attempts
   - Null byte injection

2. **FFI Boundary Tests**
   - Invalid JSON input
   - Invalid UTF-8 in arguments
   - Null pointer handling
   - Memory leak detection

3. **Output Injection Tests**
   - CSV formula injection characters
   - XML special characters in paths
   - JSON special characters in paths

4. **DoS Resistance Tests**
   - Large git history
   - Recursive symlinks
   - Large number of files
   - Deep directory structures

---

## 8. Assumptions & Accepted Risks

### Security Assumptions

1. **Filesystem integrity** - User running tokmd has appropriate permissions to scan intended directories and tokmd is not used to attack systems without authorization

2. **Git executable integrity** - `git` on PATH is the genuine git implementation, not a malicious wrapper

3. **Trusted code input** - tokmd processes source code files; users are responsible for the security of code they scan

4. **No network attack surface** - tokmd does not make outbound network requests (except via git for history)

5. **WASM isolation** - Browser WASM runtime provides memory isolation between the WASM module and JavaScript

### Accepted Risks

1. **Git history DoS** - Extremely large git histories could cause OOM; accepted because users control their repositories and `max_commits` setting provides mitigation

2. **Formula injection in CSV** - CSV exports could contain formula injection if opened in spreadsheets; accepted because CSV is machine-readable format and users are expected to handle appropriately

3. **Path disclosure in error messages** - Path-not-found errors include the path for usability; accepted because this helps legitimate users and sensitive paths are user-controlled

4. **No authentication/authorization** - tokmd has no auth; accepted because it's a local tool for personal repositories

5. **Python/Node crash on panic** - If Rust code panics despite mitigations, it could crash the host process; accepted because workspace linting and FFI safety practices make this extremely unlikely

---

## 9. Threat Model Changelog

### Version 1.0.0 (2026-05-18)

- Initial threat model created
- STRIDE analysis completed for all crate tiers (0-5)
- FFI boundary analysis for Python, Node.js, and WASM bindings
- Path bounding and traversal mitigations documented
- Git command injection analysis completed
- Output format security (CSV, XML, JSON) reviewed
- Vulnerability pattern library established for Rust-specific patterns
- Security testing strategy defined

---

## Appendix: Key Security-Critical Code Locations

| Component | File | Key Functions |
|-----------|------|---------------|
| Path Bounding | `crates/tokmd-scan/src/path/mod.rs` | `canonicalize_bounded_path`, `normalize_bounded_rel_path`, `BoundedPath::canonical` |
| FFI Entry | `crates/tokmd-core/src/ffi/mod.rs` | `run_json`, `run_json_inner` |
| Python Bindings | `crates/tokmd-python/src/lib.rs` | `lang`, `module`, `export`, `analyze`, `diff`, `cockpit` |
| Node Bindings | `crates/tokmd-node/src/lib.rs` | `run`, `lang`, `module`, `export_fn`, `analyze`, `cockpit`, `diff` |
| WASM Bindings | `crates/tokmd-wasm/src/lib.rs` | `run_json`, `run`, `run_lang`, `run_module`, `run_export`, `run_analyze` |
| Git Execution | `crates/tokmd-git/src/command.rs` | `git_log`, `git_diff`, `git_blame` |
| Error Handling | `crates/tokmd-core/src/error.rs` | `TokmdError`, `ResponseEnvelope` |
| Envelope FFI | `crates/tokmd-envelope/src/ffi.rs` | `parse_envelope`, `extract_data`, `extract_data_json` |
| Redaction | `crates/tokmd-format/src/redact/mod.rs` | `redact_path`, `redact_all` |
| CSV Export | `crates/tokmd-format/src/export/csv.rs` | `render_csv` |
| CycloneDX Export | `crates/tokmd-format/src/export/cyclonedx.rs` | `render_cyclonedx` |
