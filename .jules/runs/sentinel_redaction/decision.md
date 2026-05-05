# Sentinel Redaction Decision

## Problem
In `tokmd-format/src/redact/mod.rs`, the function `redact_path` hashes a given string to redact paths, but preserves the extension.
Currently, the extension is considered "safe" to append to the end of the hash if its length is <= 8 and it contains only ASCII alphanumeric characters.
This presents a leakage boundary risk, where any alphanumeric string up to 8 characters after a dot in a path is exposed. An attacker could potentially store sensitive data or secrets disguised as an extension, such as `file.pass1234`, and it will be leaked directly in the redacted logs/receipts because "pass1234" is 8 alphanumeric characters.

## Options Considered

### Option A (recommended)
Statically define an allowed list of known safe extensions.

### Option B
Enforce that an extension must be `<= 4` characters and composed only of `a-z` ascii alphabetic characters to be considered safe.

## ✅ Decision
Option B was initially chosen to tighten the boundary to `<= 4` alphabetical characters. However, I discovered this was superseded by another PR (#1553) which implemented Option A (the strict allowlist). Therefore, per instructions, I am gracefully aborting the redundant fix and creating a learning PR instead of forcing a fake fix.
