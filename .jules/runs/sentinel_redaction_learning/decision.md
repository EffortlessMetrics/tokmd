# Sentinel Redaction Decision

## Problem
The `tokmd-format` crate provides redaction functions like `redact_path`, which hashes paths to protect sensitive information but preserves short, alphanumeric file extensions (e.g., `src/main.rs` -> `<hash>.rs`).
However, because it uses any short (<= 8 chars) alphanumeric extension, it can leak sensitive data if a file happens to be named something like `file.passwd` or `file.secret`. This violates the security boundary guarantee of complete redaction for arbitrary sensitive paths.
The test `test_redact_path_leak` currently attempts to check for this leak but uses `"super_secret_password_123"`, which is longer than 8 chars, and thus doesn't trigger the bug. If we change it to `"passwd"`, the test fails.

## Option A (recommended)
**Use an explicit whitelist of safe extensions**
- What it is: Instead of blindly preserving any short alphanumeric string, we use a hardcoded list of common source/data extensions (e.g., `"rs", "js", "ts", "json", "toml", "md", "csv", "yml", "yaml", "xml", "txt"`).
- Why it fits: It fully closes the leakage vector. Only explicitly safe, known file extensions are preserved in redacted output, guaranteeing no sensitive data from arbitrary extensions can escape.
- Trade-offs: Might redact some obscure, but benign, language extensions. This is a safe trade-off for a security-boundary profile.

## Option B
**Only redact paths fully (no extensions)**
- What it is: Remove extension preservation entirely and just output the hash.
- Why it fits: Simplest solution, guarantees zero leakage.
- Trade-offs: Redacted output becomes significantly less useful for debugging or analysis because you can't tell what types of files were affected.

## Decision
Choose Option A. It provides the strongest security guarantee without losing the utility of recognizing file types in redacted SBOM/receipt outputs. We will define a `SAFE_EXTENSIONS` array and check against it in `redact_path`. We'll also update `test_redaction_leak.rs` to actually test a short leaked string like `"passwd"` to ensure it's caught.
