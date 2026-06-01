# Threat Model for tokmd

**Version:** 2026-06-01
**Generated:** 2026-06-01 (Initial model based on architecture review)
**Repository:** EffortlessMetrics/tokmd

## Asset Inventory

### Trust Anchors
- FFI envelope contract (`.factory/threat-model.md`)
- blake3 hashing for integrity verification
- Schema version constants for receipt versioning

### Critical Components
| Component | Tier | Sensitivity |
|-----------|------|-------------|
| tokmd-core FFI | 4 | HIGH - External JSON interface |
| tokmd-git shell-out | 3 | HIGH - User-controlled git refs |
| tokmd-scan file walking | 1 | MEDIUM - Filesystem access |
| Python/Node/WASM bindings | 5 | HIGH - Cross-language FFI |

## Trust Boundaries

### 5 Trust Boundaries Identified:

1. **CLI → Process**: Human operator to tokmd binary (trusted operator)
2. **FFI → tokmd-core**: Untrusted JSON payload via `run_json()` (hardened boundary)
3. **tokmd-git shell-out**: User-controlled git refs passed to `git` binary
4. **tokmd-scan → tokei**: Filesystem walking boundary via `ignore` crate
5. **Cross-language FFI**: Python (PyO3), Node.js (napi-rs), WASM (wasm-bindgen) bindings

## STRIDE Analysis

### Spoofing
- **Unknown modes**: `run_json` with invalid mode strings
- **Wrong repo detection**: Path context confusion

### Tampering
- **Path traversal**: User-supplied paths with `..` components
- **Base64 injection**: Decoded content without size limits
- **Git command injection**: Refs not validated for `-` prefix
- **JSON nesting**: Deeply nested payloads causing stack overflow

### Repudiation
- **No audit trail**: Receipts lack cryptographic signing
- **Receipt forgery**: BTreeMap ordering only, no HMAC

### Information Disclosure
- **Path leakage**: Error messages reveal filesystem structure
- **Redaction gaps**: Secret redaction in output fields
- **Git author emails**: Email addresses in git analysis

### Denial of Service
- **Large inputs**: Files exceeding reasonable size limits
- **Deep git history**: `git log` on large repositories
- **Memory exhaustion**: Large in-memory scans

### Elevation of Privilege
- **Malicious config**: `.tokeignore` with crafted patterns
- **Path traversal escapes**: Absolute path escapes via symlinks
- **WASM sandbox**: Capabilities advertised vs actual

## Key Mitigations

1. **FFI path validation**: 4096-byte limit, control char rejection, no `..` traversal
2. **`run_json` envelope**: Always returns valid JSON, never panics
3. **Rust `Command::args()`**: Git arguments passed safely without shell expansion
4. **Python `?` operator**: No `.expect()` in production code
5. **WASM capability advert**: Correctly advertises rootless capability surface

## Identified Gaps (Medium/Low Priority)

1. No per-input byte limit on decoded base64 content
2. Git refs not validated for `-` prefix (potential git flag injection)
3. Error messages may leak path fragments

## Next Review Date

Review by: 2026-09-01 (90 days from generation)
