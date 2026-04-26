# Sentinel Decision: Redaction Boundary Hardening

## Option A: Redact `module_roots` in ExportData/ModuleReport
Currently `module_roots` are strings passed into the `redact_module_roots` function.
If we look at `crates/tokmd-format/src/lib.rs`, `redact_module_roots` uses `short_hash` directly instead of matching path semantics. Module roots should be redacted via `short_hash` correctly as they are just prefixes, but the paths they apply to are redacted via `redact_path`.
However, the `redact_path` behavior is quite important: it hashes the path but leaves the extension, using `clean_path` and `short_hash` internally.

Wait, looking at `crates/tokmd-format/src/lib.rs` line 735:
```rust
    if redact == RedactMode::All {
        final_args.module_roots = redact_module_roots(&final_args.module_roots, redact);
        final_report.module_roots = redact_module_roots(&final_report.module_roots, redact);
        for row in &mut final_report.rows {
            row.module = short_hash(&row.module);
```
In `redact_module_roots`:
```rust
fn redact_module_roots(roots: &[String], redact: RedactMode) -> Vec<String> {
    if redact == RedactMode::All {
        roots.iter().map(|r| short_hash(r)).collect()
    ...
```

Wait, what about `scan_args` redaction?
In `crates/tokmd-format/src/scan_args/mod.rs`:
```rust
    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
...
    if should_redact {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }
```
Wait, `args.paths` is initialized as a string and then modified again?
```rust
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
```
Then later:
```rust
    if should_redact {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }
```
If `redact_path` takes `&str` and returns `String`, `args.paths.iter().map(|p| redact_path(p))` works because `args.paths` is `Vec<String>`.

But what about the security / leak issue?
What if `redact_path` is called on something that isn't a path, or leaks something?
Let's look at `redact_path`:
```rust
pub fn redact_path(path: &str) -> String {
    let cleaned = clean_path(path);
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&cleaned);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}
```
If `ext` is really long or contains sensitive info (e.g. `file.secret_extension`), it's preserved. But more concerningly, what if the path is an absolute path?
Actually, what about `redact_rows` in `crates/tokmd-format/src/lib.rs`:
```rust
fn redact_rows(rows: &[FileRow], mode: RedactMode) -> impl Iterator<Item = Cow<'_, FileRow>> {
    rows.iter().map(move |r| match mode {
        RedactMode::None => Cow::Borrowed(r),
        RedactMode::Paths => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: r.module.clone(),
            lang: r.lang.clone(),
            kind: r.kind,
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
        }),
        RedactMode::All => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: short_hash(&r.module),
            lang: r.lang.clone(),
            kind: r.kind,
            code: r.code,
            comments: r.comments,
            blanks: r.blanks,
            lines: r.lines,
            bytes: r.bytes,
            tokens: r.tokens,
        }),
    })
}
```

Wait! In `RedactMode::Paths`, the `module` is NOT redacted!
If `module` is derived from the directory path (which it usually is, e.g. `src/secret_path`), then `RedactMode::Paths` LEAKS the directory paths through the `module` field!
Let's check `crates/tokmd-format/src/lib.rs` redaction tests.
```rust
    fn redact_rows_paths_mode() {
        let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();
        // Paths should be redacted (16 char hash + extension)
        assert_ne!(redacted[0].path, "src/lib.rs");
        // Module should NOT be redacted
        assert_eq!(redacted[0].module, "src");
```
Wait, if `RedactMode::Paths` is meant to redact paths, but `module` (which is a path prefix) is kept in plaintext, that's a leak. BUT `RedactMode::Paths` explicitly says "Module should NOT be redacted" in the test. So that might be intentional for `Paths` vs `All`.

What about `ScanArgs`?
In `crates/tokmd-format/src/scan_args/mod.rs`:
```rust
    let mut args = ScanArgs {
        paths: paths.iter().map(|p| normalize_scan_input(p)).collect(),
        excluded: if should_redact {
            global.excluded.iter().map(|p| short_hash(p)).collect()
        } else {
            global.excluded.clone()
        },
```
Wait, `excluded` uses `short_hash`. What about `paths`?
```rust
    if should_redact {
        args.paths = args.paths.iter().map(|p| redact_path(p)).collect();
    }
```
If `redact` is `RedactMode::Paths`, `global.excluded` is hashed with `short_hash`, but `args.paths` is hashed with `redact_path`.

Let's look at `RedactMode` in `crates/tokmd-types/src/lib.rs` (if it's there) or `crates/tokmd-settings/src/lib.rs`.
Actually, `RedactMode::Paths` means "redact file paths". `RedactMode::All` means "redact paths AND module names".

Wait, look at `crates/tokmd-format/src/lib.rs` line 737:
```rust
    if redact == RedactMode::All {
        final_args.module_roots = redact_module_roots(&final_args.module_roots, redact);
```
What about `RedactMode::Paths`? If `RedactMode::Paths` redacts paths, should it redact `module_roots`? No, because `module_roots` are modules. But `module_roots` are also paths!
Wait, what if `args.paths` contains absolute paths or sensitive information?
Let's consider `tokmd_format::redact::redact_path`:
```rust
pub fn redact_path(path: &str) -> String {
    let cleaned = clean_path(path);
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&cleaned);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}
```
If `ext` is something like `secret_file_name_that_is_long`, wait, what is an extension? `Path::new(...).extension()` gets everything after the last dot. If the file is `config.secret_db_password`, the extension is `secret_db_password`, which is leaked!
Should `redact_path` restrict the length of the extension or only allow known extensions?
"Redact a path by hashing it while preserving the file extension. This allows redacted paths to still be recognizable by file type while hiding the actual path structure."

Let's check `crates/tokmd-types/src/lib.rs`: `RedactMode`
```rust
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactMode {
    #[default]
    None,
    Paths,
    All,
}
```

Wait, look at `short_hash`:
```rust
pub fn short_hash(s: &str) -> String {
    let cleaned = clean_path(s);
    let mut hex = blake3::hash(cleaned.as_bytes()).to_hex().to_string();
    hex.truncate(16);
    hex
}
```
If `clean_path` is:
```rust
fn clean_path(s: &str) -> String {
    let mut normalized = s.replace('\\', "/");
    // Strip leading ./
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }
    // Remove interior /./
    while normalized.contains("/./") {
        normalized = normalized.replace("/./", "/");
    }
    // Remove trailing /.
    if normalized.ends_with("/.") {
        normalized.truncate(normalized.len() - 2);
    }
    normalized
}
```
Is `clean_path` safe against `../`? No, it doesn't resolve `../`! So `src/../secret/file.rs` hashes differently than `secret/file.rs`, which is fine.

Let's check another place in `crates/tokmd-format/src/lib.rs` that might be leaking paths:
```rust
pub fn write_export_report_to<W: Write>(
    out: &mut W,
    export: &ExportData,
    global: &ScanOptions,
    args: &ExportArgs,
) -> Result<()> {
...
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| p.display().to_string().replace('\\', "/"))
                },
```
Wait, if `strip_prefix` is a path prefix like `/home/user/my_secret_project`, `redact_path` hashes it and appends the extension (if it has one). E.g. `/home/user/project.git` -> `[hash].git`.
This seems fine.

But what about CycloneDX export?
```rust
fn write_export_cyclonedx_impl<W: Write>(
    out: &mut W,
    export: &ExportData,
    redact: RedactMode,
    serial_number: Option<&str>,
    timestamp: Option<&str>,
) -> Result<()> {
    // Apply redaction to rows before generating components
    let components: Vec<CycloneDxComponent> = redact_rows(&export.rows, redact)
        .map(|row| {
            // Group is the module, name is the file name
            // If redaction is Paths or All, the path is already hashed in `row.path`
            // If redaction is All, the module is already hashed in `row.module`
...
            CycloneDxComponent {
                type_: "file".to_string(),
                group: if row.module.is_empty() {
                    None
                } else {
                    Some(row.module.to_string())
                },
                name: Path::new(&row.path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
...
```
Wait! `Path::new(&row.path).file_name()`!
If `row.path` is redacted, what does it look like?
`redact_path("src/secret/config.json")` returns `1234567890abcdef.json` (a 16 char hash + `.json`).
Wait, `redact_path` returns ONLY the hash + extension! It does NOT preserve the directory structure!
Let's check `redact_path` again:
```rust
pub fn redact_path(path: &str) -> String {
    let cleaned = clean_path(path);
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&cleaned);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}
```
Ah! `redact_path("src/secret/config.json")` returns something like `a1b2c3d4e5f6g7h8.json`. There are no slashes in the output of `redact_path`!

If there are no slashes, `Path::new(&row.path).file_name()` is just the whole redacted string.
Wait, if `row.path` is just the hash + extension, then we've lost the directory structure entirely in the exported data! This is correct for redaction (we want to hide the structure).

But wait, look at `CycloneDxComponent` `name`:
```rust
                name: Path::new(&row.path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
```
If `row.path` has no slashes, `file_name()` is the whole string.

Wait, is there a vulnerability in `redact_path` regarding the file extension?
```rust
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
```
If I have a file named `my_secret_password.txt.super_secret_leak`, `ext` is `super_secret_leak`. This leaks information.
But more importantly, look at how `redact_path` handles paths with no extensions, like `.env`.
For `.env`, what is `extension()`?
`Path::new(".env").extension()` is `None`! Wait, no, `Path::new(".env").extension()` is `None` in Rust! Wait, let's verify.
In Rust, `Path::new(".env").extension()` returns `None`. `file_name()` is `.env`.
If it returns `None`, `ext` is `""`.
So `.env` is redacted to just a 16-character hash. That seems correct (it hides `.env`).
What about `.git/config`? `extension()` is `None`. It hashes to 16 chars.

Wait, look at `strip_prefix` in `write_export_report_to`:
```rust
                strip_prefix: if should_redact {
                    args.strip_prefix
                        .as_ref()
                        .map(|p| redact_path(&p.display().to_string().replace('\\', "/")))
                } else {
```
If `args.strip_prefix` is `/home/user/project`, its redacted form is `hash`. Wait!
If `strip_prefix` in the output receipt is compared to the redacted paths, it will NEVER match if it was a directory path, because `redact_path` output has no slashes, so the client might not be able to reconstruct or correlate anything, but that's a semantic issue, not a security leak.

What is the actual security leak?
Let's check the Sentinel assignment:
"Land one security-significant hardening improvement.
Target ranking
1) redaction correctness and leakage prevention
2) FFI parsing / trust boundaries
3) subprocess / environment / path boundary hardening
4) receipt/schema trust and deterministic safety"

Let's look at `redact_path` again:
```rust
pub fn redact_path(path: &str) -> String {
    let cleaned = clean_path(path);
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&cleaned);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}
```
If a user creates a file named `a.my_secret_token_is_12345`, the extension is `my_secret_token_is_12345`.
`redact_path` will append this entire string to the hash, leaking the token!
Is there a way to limit the extension length or character set to prevent arbitrary data leakage via extensions?
Common extensions are short (1-4 chars) and alphanumeric.
What if we enforce that `ext` only contains alphanumeric characters and is no longer than, say, 8 characters? Or we just sanitize it?
Wait, if someone names a file `script.sh?token=secret`, the OS allows `?` in file names (except on Windows). `Path::new` will see `sh?token=secret` as the extension!

Let's check `crates/tokmd-format/src/redact/mod.rs`:
```rust
pub fn redact_path(path: &str) -> String {
    let cleaned = clean_path(path);
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&cleaned);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}
```
Wait! What if we restrict the extension to a known safe length?
```rust
    let mut ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    // Hardening: limit extension length to prevent data leakage through long extensions
    if ext.len() > 8 {
        ext = "";
    } else if !ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        ext = "";
    }
```
Actually, many test files use `[a-z]{1,4}` for extensions.
Wait, look at `crates/tokmd-format/src/lib.rs` property tests:
```rust
        fn redact_rows_paths_end_with_extension(ext in "[a-z]{1,4}") {
            let redacted: Vec<_> = redact_rows(&rows, RedactMode::Paths).collect();
            prop_assert!(redacted[0].path.ends_with(&format!(".{}", ext)),
                "Redacted path '{}' should end with .{}", redacted[0].path, ext);
```
So up to 4 chars is tested.

Let's see if there's any other leakage.
What about `LangArgs`?
In `write_lang_report_to`:
```rust
pub fn write_lang_report_to<W: Write>(
    out: &mut W,
    report: &LangReport,
    global: &ScanOptions,
    args: &LangArgs,
) -> Result<()> {
    let should_redact = matches!(args.format, TableFormat::Json | TableFormat::Jsonld)
        && false; // Wait, is there redaction for LangReport?
```
Wait, let's look at `write_lang_report_to` in `crates/tokmd-format/src/lib.rs`!

Looking closer at `tokmd-types/src/lib.rs` and the redact tests, if `redact == RedactMode::Paths`, the path is hashed, but what about the module? `RedactMode::Paths` leaves `module` intact! If `module` is just a directory path from the repo root (e.g. `crates/tokmd-types`), then this *leaks the directory structure*. The prompt mentions: `redaction correctness and leakage prevention`.

Wait, in `RedactMode::Paths`, what IS the module?
The module is typically a directory prefix. E.g., `src/foo`.
If a user asks for `tokmd export --redact paths`, they expect file paths to be redacted. But the `module` column in the CSV/JSON contains the plaintext directory structure!
Wait, the prompt says "Target ranking 1) redaction correctness and leakage prevention".

Let's look at `RedactMode` documentation or its use in `crates/tokmd-format/src/lib.rs`.
```rust
    rows.iter().map(move |r| match mode {
        RedactMode::None => Cow::Borrowed(r),
        RedactMode::Paths => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: r.module.clone(),
            lang: r.lang.clone(),
            ...
        }),
        RedactMode::All => Cow::Owned(FileRow {
            path: redact_path(&r.path),
            module: short_hash(&r.module),
            ...
        }),
    })
```
If `Paths` mode is used, `module` is deliberately left unredacted. That seems to be the intended behavior of `Paths` (vs `All`). So maybe that's not a bug but a feature?

Wait, what about `redact_path` file extensions?
```rust
pub fn redact_path(path: &str) -> String {
    let cleaned = clean_path(path);
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&cleaned);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}
```
If I have a file named `my_bank_account_password_is_hunter2.secret`, the extension is `secret`, which is fine. If the file is `config.my_bank_account_password_is_hunter2`, the extension is `my_bank_account_password_is_hunter2`. This is an arbitrary string that leaks through redaction!

Let's test this in `cargo`:

So the `redact_path` function leaks the extension, and the extension could be arbitrarily long and contain sensitive data.

Is there anything else?
What about CycloneDX component names?
In `write_export_cyclonedx_impl`:
```rust
            CycloneDxComponent {
                type_: "file".to_string(),
                group: if row.module.is_empty() {
                    None
                } else {
                    Some(row.module.to_string())
                },
                name: Path::new(&row.path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
```
Wait!
If `row.path` was redacted using `RedactMode::Paths`, it looks like `a1b2c3d4e5f6g7h8.ext`.
`Path::new(&row.path).file_name()` is `a1b2c3d4e5f6g7h8.ext`.
BUT what if it was NOT redacted?
`Path::new("src/foo/bar.rs").file_name()` is `bar.rs`.
Wait, CycloneDX `name` is just the file name!
BUT the `path` property or `properties` list?
```rust
                properties: vec![
                    CycloneDxProperty {
                        name: "tokmd:lang".to_string(),
                        value: row.lang.clone(),
                    },
```
Wait, the actual path is lost in CycloneDX format?
If `name` is `file_name()`, then CycloneDX component `name` is just the base name. Is the full path stored anywhere in CycloneDX?
Let's check `crates/tokmd-format/src/lib.rs` CycloneDX export.

Wait, `name: row.path.clone()`. It does not use `file_name()`.

So the main redaction leak is in `redact_path`.

```rust
pub fn redact_path(path: &str) -> String {
    let cleaned = clean_path(path);
    let ext = Path::new(&cleaned)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mut out = short_hash(&cleaned);
    if !ext.is_empty() {
        out.push('.');
        out.push_str(ext);
    }
    out
}
```

If we sanitize `ext` to only allow standard short alphanumeric extensions, we close the data leakage vector.
For example, we can bound the extension length to `<= 8` and require it to be entirely ASCII alphanumeric. If it violates this, we treat it as having no extension and just use the hash.

This matches the `Target ranking 1) redaction correctness and leakage prevention` and `security-boundary` gate profile perfectly.

Let's write a targeted unit test to prove the leak and then fix it.
## ✅ Decision

I have chosen Option A: Hardening `redact_path` to sanitize the file extension.

`redact_path` is the canonical function for redacting paths. It hashed the file path but explicitly preserved the file extension (so that users can still identify the file type) via `Path::new(&cleaned).extension()`. However, `Path::extension()` returns an arbitrary string after the last dot. A file named `a.my_secret_token_is_12345` would result in the extension being `my_secret_token_is_12345`, which `redact_path` appends to the hash, leaking the token in plaintext.

To fix this, I restricted the extension to alphanumeric ASCII characters only, and set a hard length limit of 8 characters. Any extension failing this check is entirely redacted.

This effectively prevents arbitrary data leakage through the extension while preserving the utility of standard short extensions (e.g. `.rs`, `.md`, `.json`, `.toml`).

The decision perfectly aligns with the `Sentinel` persona and `security-boundary` gate profile, targeting redaction correctness and leakage prevention.
