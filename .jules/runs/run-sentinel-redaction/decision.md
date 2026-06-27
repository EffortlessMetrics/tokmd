## 💡 Summary
Fixing a security vulnerability where path redaction does not properly normalize paths containing parent directory segments (`..`), which can lead to different hashes being generated for paths that logically point to the same location, potentially leaking directory structure information.

## 🎯 Why
Redaction currently normalizes separators (`/` and `\`) and `.`/`./` segments. However, it fails to normalize `..` segments. For example, `a/b/../c/secret.txt` and `a/c/secret.txt` point to the same file but will produce different redacted hashes. This violates the guarantee that logically identical paths hash identically, and risks leaking directory structure information through hash comparisons or side channels.

## 🔎 Evidence
```rust
fn main() {
    let p1 = tokmd_format::redact::redact_path("a/b/../c/secret.txt");
    let p2 = tokmd_format::redact::redact_path("a/c/secret.txt");
    println!("p1: {}", p1);
    println!("p2: {}", p2);
}
```
Outputs:
```
p1: a6fb4284d72856f6.txt
p2: e09e20db9035f498.txt
```
This shows the hashes are different.

## 🧭 Options considered

### Option A (recommended)
- Update `clean_path` in `crates/tokmd-format/src/redact/mod.rs` to correctly resolve `..` segments during normalization.
- Using standard component-based path normalization is robust and fits well into the existing function logic.
- Trade-offs: Increases the cost of `clean_path` slightly as it requires iterating over components rather than just string operations, but ensures correctness. Fits exactly the shard scope (formatting pipeline, redaction correctness).

### Option B
- Modify `scan_args` to canonicalize paths using `std::fs::canonicalize` before redaction.
- Trade-offs: Requires filesystem access during formatting which is a heavy side-effect and fails if the file doesn't exist (e.g. during testing or if the path is virtual). We shouldn't rely on the filesystem for redaction of a string.

## ✅ Decision
Option A. It's the robust and correct way to fix redaction leakage without relying on the filesystem.
