# Sentinel Redaction Decision

## Problem
In `tokmd-format/src/redact/mod.rs`, the function `redact_path` hashes a given string to redact paths, but preserves the extension.
Currently, the extension is considered "safe" to append to the end of the hash if its length is <= 8 and it contains only ASCII alphanumeric characters:
```rust
    let ext = if ext.len() <= 8 && ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        ext
    } else {
        ""
    };
```

This presents a leakage boundary risk, where any alphanumeric string up to 8 characters after a dot in a path is exposed. An attacker could potentially store sensitive data or secrets disguised as an extension, such as `file.pass1234`, and it will be leaked directly in the redacted logs/receipts because "pass1234" is 8 alphanumeric characters.

A safer trust-boundary approach would be to only preserve a set of well-known safe extensions, or to use a stricter check (such as only preserving `a-z` lower case letters up to maybe 4 characters like standard file extensions).

## Options Considered

### Option A (recommended)
Statically define an allowed list of known safe extensions (e.g. `rs`, `js`, `ts`, `py`, `go`, `md`, `txt`, `json`, `toml`, `xml`, `yaml`, `yml`, `c`, `cpp`, `h`, `hpp`, `java`, `rb`, `php`, `sh`, `bat`, `ps1`, etc.).
Or, simply enforce that an extension must be `<= 4` characters and composed only of `a-z` ascii lowercase letters to be considered safe. Wait, `a-z` lowercase might be enough, but numbers in extensions are common (e.g. `mp3`, `mp4`, `h264`, etc.).

Let's do `< 5` characters (so 1 to 4 characters) and `.is_ascii_alphabetic()` to further limit the boundary, or specifically only allow common extensions. Actually, the simplest boundary hardening that prevents the `pass1234` leak is to reduce the length and enforce alphabetic-only (or small known-list).
But what if we just enforce that the extension must be `<= 4` length and contain only ascii alphabetic characters, maybe digits for `mp4` or `f90`.

Let's look at `tokmd` scope. It's a token counting tool, so the extensions are for code files.
If we restrict it to length `<= 5` and `.is_ascii_alphanumeric()`, then `pass1` could still leak.
The best boundary hardening is to restrict it to a specific length `<= 4` and only `[a-zA-Z]`, or a statically known list. Since it's a "redact" feature, if an unknown extension is dropped and replaced with nothing, it's just a 16-char hash. The purpose of `redact_path` is to hide sensitive paths but keep some identifiability.

Alternatively, since `redact_path`'s goal is to hide everything but the file type, we can restrict extensions to length <= 5 and `is_ascii_alphabetic()`.

Let's look at `crates/tokmd-format/src/redact/mod.rs`:
Currently:
```rust
    let ext = if ext.len() <= 8 && ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        ext
    } else {
        ""
    };
```

If we update this to:
```rust
    let ext = if ext.len() <= 5 && ext.chars().all(|c| c.is_ascii_alphabetic()) {
        ext
    } else {
        ""
    };
```

Wait, `is_ascii_alphabetic` would drop `.rs`, `.md`, `.js`, etc. But what about `.c++` or `.f90`? `+` is not alphanumeric. `f90` has digits. `mp4` has digits.
If we use `ext.len() <= 4 && ext.chars().all(|c| c.is_ascii_alphanumeric()) && ext.chars().any(|c| c.is_ascii_alphabetic())`, we prevent pure numbers.

Let's check what `is_ascii_alphanumeric` and `<= 4` does: `pass` (4 chars). It could still leak a 4-char string. Is that a big deal? "redaction correctness and leakage prevention".

If we use a static allowlist, we eliminate the leak entirely.

### Option A: Use a static allowlist of common programming / text extensions.
Since `tokmd` is for scanning markdown, source code, etc.

Let's grep for supported extensions in `tokmd`.

Wait, `redact_path` preserves extensions to keep them recognizable. Instead of an allowlist (which would need to be updated constantly for new languages), we can enforce that extensions are short (e.g. `<= 4` chars) AND they only contain alphabetical characters `[a-z]` (and maybe digits `[0-9]` but not as the first character, or just lowercase ascii).

Currently, `is_ascii_alphanumeric` allows upper and lower case, and digits. 8 characters is enough to leak `password`, `api_key`, etc., especially if someone uses `file.password` or `file.secret1`.

Let's modify the check to restrict length to `<= 4` characters and require `is_ascii_alphabetic` only, or perhaps `is_ascii_lowercase()`. Wait, `ext.len() <= 5 && ext.chars().all(|c| c.is_ascii_alphanumeric()) && !ext.chars().any(|c| c.is_ascii_digit())`.

Wait! We don't need to overcomplicate it. Reducing `len <= 4` and `all(|c| c.is_ascii_alphabetic())` would work, but `.rs`, `.md`, `.js`, `.py`, `.c`, `.cpp`, `.json` (5 chars!), `.yaml` (4 chars).
If we need `.json` we need `len <= 5`.
If we allow `is_ascii_alphabetic()`, we lose `f90`, `h264`, `mp4`. But are those code extensions?
If we use `is_ascii_alphanumeric()`, a 5 char leak could be `pass1`, `key12`, `token`. "token" is 5 chars.
Is a 5-char leak acceptable?

Let's look at `redact_path`:
```rust
    let ext = if ext.len() <= 5 && ext.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
        ext
    } else {
        ""
    };
```
If we want to restrict it further, we can check for an explicit list of known extensions, OR we can just limit length to 4. If `.json` is common, maybe 5.

Let's look at Option A: Restrict extension preservation to `<= 5` characters and `is_ascii_alphanumeric()`.
Wait, the leak test I created `test_redact_path_leak_8_chars` uses `pass1234` which is 8 chars. If I lower the limit to 4, it will prevent this specific leak.
But what if the test I wrote is exactly what's needed?
"This presents a leakage boundary risk, where any alphanumeric string up to 8 characters after a dot in a path is exposed. An attacker could potentially store sensitive data or secrets disguised as an extension, such as file.pass1234, and it will be leaked directly in the redacted logs/receipts because "pass1234" is 8 alphanumeric characters."

Let's reduce the length to `4` for extensions (which covers `.rs`, `.md`, `.js`, `.py`, `.cpp`, `.txt`, `.yaml`, `.toml` etc), and specifically map `.json` -> `.json` if we want, or just let `.json` be redacted if we strictly enforce `< 5`.
Wait, `.json` is 4 characters! `j-s-o-n` is 4.
Wait, `ext.len()` for `json` is 4.
Wait, `config.json` -> `ext` is `json` -> len = 4.
Ah! So `ext.len() <= 4` covers `json`, `yaml`, `toml`, `html`, `java`, `cpp`, `php`!
What extension is 5 characters? `.swift` is 5! `.scala` is 5! `.kotlin` is 6!
If we restrict to 4, we lose `.swift`, `.scala`, `.kotlin` preservation. They would just become bare hashes. Is that a problem for a redacted output? Redaction's priority is security. A bare hash is perfectly secure. It just lacks the `.swift` hint.

Let's look at `crates/tokmd-format/tests/format_tests.rs` or similar to see if any tests expect a specific extension length.

If I change the length from 8 to 4, I see the proptest in `crates/tokmd-types/tests/determinism_props.rs` generates extensions `\\.[a-z]{1,4}` which is exactly 1 to 4 characters. So my change will pass existing property tests!

```rust
    fn redact_path_is_deterministic(input in "[a-z]{1,5}(/[a-z]{1,5}){0,3}/[a-z]{1,8}\\.[a-z]{1,4}") {
```
This implies the test expects 1-4 character extensions. Length 8 was likely arbitrarily large.

Furthermore, we can tighten `is_ascii_alphanumeric` to `is_ascii_alphabetic` (which handles `rs`, `json`, `cpp`), but maybe allow digits so `h264`, `f90`, `mp4` work. If we allow digits, `len <= 4` still permits `key1`, `pas2`, which are short but still slight leakage. However, 4 characters is significantly smaller entropy than 8 characters, and `pass` or `key` is a 4 character string. To be strictly safe, we could restrict it to an explicitly allowed list of known code extensions. But wait, `tokmd` parses code.

If I use an explicit array: `["rs", "js", "ts", "py", "c", "cpp", "h", "hpp", "md", "txt", "json", "toml", "yaml", "yml", "xml", "java", "go", "rb", "php", "cs", "swift", "kt", "scala", "sh", "bat", "sql", "html", "css", "scss", "vue", "jsx", "tsx"]`
Is it worth it? Or does standard boundary hardening just recommend length restriction (e.g., `< 5`) and alphabetical only?
Let's just use `ext.len() <= 4 && ext.chars().all(|c| c.is_ascii_alphabetic())`. This drops `f90`, `mp4` but for a redaction boundary, dropping an uncommon code extension and just emitting the hash is 100% fine. A redacted receipt will just say `13351972638830f3` instead of `13351972638830f3.mp4`. This is perfectly acceptable for redaction.

Wait, if I change it to `is_ascii_alphabetic`, I should check if there are tests expecting `archive.tar.gz`. `gz` is alphabetic. `1.2.3` would fail, but extensions usually don't have dots. `ext` is the string *after* the last dot.

Let's test `ext.len() <= 4 && ext.chars().all(|c| c.is_ascii_alphabetic())` by running `cargo test -p tokmd-format` and `cargo test -p tokmd-types`.

The test suite passed completely with `ext.len() <= 4 && ext.chars().all(|c| c.is_ascii_alphabetic())` ! This confirms that standard deterministic tests for formatting and core functions are perfectly fine with shorter, restricted extensions.

Wait, the proptest `redact_path_is_deterministic(input in "[a-z]{1,5}(/[a-z]{1,5}){0,3}/[a-z]{1,8}\\.[a-z]{1,4}")` in `crates/tokmd-types/tests/determinism_props.rs` already guarantees that generated paths have alphabetic extensions of length 1 to 4! So the test suite was practically begging for this boundary hardening.

I will formulate my execution plan based on this.

Option A is chosen:
Tighten the `redact_path` boundary in `tokmd-format/src/redact/mod.rs` to only preserve extensions of length <= 4 containing exclusively ASCII alphabetic characters. This prevents 8-character alphanumeric leaks while preserving common codebase extensions (`.rs`, `.js`, `.py`, `.cpp`, `.md`, `.json`, `.yaml`).
