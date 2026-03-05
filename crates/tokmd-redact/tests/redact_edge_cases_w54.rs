//! W54 – Comprehensive redaction edge-case tests.

use tokmd_redact::{redact_path, short_hash};

// ── Determinism ────────────────────────────────────────────────

#[test]
fn short_hash_deterministic_multiple_calls() {
    let input = "src/secrets/api_key.rs";
    let results: Vec<String> = (0..100).map(|_| short_hash(input)).collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn redact_path_deterministic_multiple_calls() {
    let input = "config/database.yml";
    let results: Vec<String> = (0..100).map(|_| redact_path(input)).collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

// ── Hash output format ─────────────────────────────────────────

#[test]
fn short_hash_always_16_hex_chars() {
    let inputs = ["", "a", "a/b/c/d/e", "very long path with spaces"];
    for input in inputs {
        let h = short_hash(input);
        assert_eq!(h.len(), 16, "failed for input: {input}");
        assert!(
            h.chars().all(|c| c.is_ascii_hexdigit()),
            "non-hex in hash for: {input}"
        );
    }
}

#[test]
fn short_hash_is_lowercase() {
    let h = short_hash("SomeInput");
    assert_eq!(h, h.to_lowercase());
}

// ── Special characters ─────────────────────────────────────────

#[test]
fn hash_of_path_with_spaces() {
    let h = short_hash("my project/src dir/file name.rs");
    assert_eq!(h.len(), 16);
}

#[test]
fn hash_of_path_with_special_chars() {
    let h = short_hash("a@b#c$d%e^f&g*h");
    assert_eq!(h.len(), 16);
}

#[test]
fn redact_path_with_parentheses() {
    let r = redact_path("dir(1)/file(2).txt");
    assert!(r.ends_with(".txt"));
    assert_eq!(r.len(), 16 + 1 + 3); // hash + dot + "txt"
}

#[test]
fn hash_with_null_byte() {
    let h = short_hash("before\0after");
    assert_eq!(h.len(), 16);
}

#[test]
fn hash_with_newline() {
    let h = short_hash("line1\nline2");
    assert_eq!(h.len(), 16);
}

#[test]
fn hash_with_tab() {
    let h = short_hash("col1\tcol2");
    assert_eq!(h.len(), 16);
}

// ── Unicode ────────────────────────────────────────────────────

#[test]
fn hash_of_cjk_path() {
    let h = short_hash("项目/源码/主.rs");
    assert_eq!(h.len(), 16);
}

#[test]
fn hash_of_emoji_path() {
    let h = short_hash("🚀/launch/🌍.rs");
    assert_eq!(h.len(), 16);
}

#[test]
fn redact_unicode_preserves_extension() {
    let r = redact_path("パッケージ/ファイル.json");
    assert!(r.ends_with(".json"));
}

// ── Empty and minimal input ────────────────────────────────────

#[test]
fn short_hash_empty_string() {
    let h = short_hash("");
    assert_eq!(h.len(), 16);
}

#[test]
fn redact_empty_path() {
    let r = redact_path("");
    // Empty path has no extension
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn redact_single_char_path() {
    let r = redact_path("x");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn redact_single_dot_file() {
    // A file named "." has no extension
    let r = redact_path(".");
    assert!(!r.is_empty());
}

// ── Cross-platform stability ───────────────────────────────────

#[test]
fn short_hash_backslash_forward_equivalent() {
    assert_eq!(
        short_hash("crates/tokmd/src/lib.rs"),
        short_hash(r"crates\tokmd\src\lib.rs")
    );
}

#[test]
fn redact_path_backslash_forward_equivalent() {
    assert_eq!(
        redact_path("crates/tokmd/src/main.rs"),
        redact_path(r"crates\tokmd\src\main.rs")
    );
}

#[test]
fn dot_prefix_normalization_in_hash() {
    assert_eq!(short_hash("src/lib.rs"), short_hash("./src/lib.rs"));
    assert_eq!(short_hash("a/b/c"), short_hash("./a/b/c"));
}

#[test]
fn interior_dot_normalization_in_hash() {
    assert_eq!(
        short_hash("crates/foo/src/lib.rs"),
        short_hash("crates/./foo/./src/lib.rs")
    );
}

#[test]
fn trailing_dot_normalization_in_hash() {
    assert_eq!(short_hash("crates/foo"), short_hash("crates/foo/."));
}

// ── One-way property ───────────────────────────────────────────

#[test]
fn hash_does_not_contain_original_path() {
    let path = "src/super_secret/private_config.rs";
    let h = short_hash(path);
    assert!(!h.contains("src"));
    assert!(!h.contains("secret"));
    assert!(!h.contains("private"));
}

#[test]
fn redacted_path_does_not_contain_original_dirs() {
    let path = "internal/credentials/token.json";
    let r = redact_path(path);
    assert!(!r.contains("internal"));
    assert!(!r.contains("credentials"));
    assert!(!r.contains("token"));
    // Only extension leaks through (by design)
    assert!(r.ends_with(".json"));
}

// ── Different inputs produce different hashes ──────────────────

#[test]
fn different_inputs_different_hashes() {
    let pairs = [
        ("alpha", "beta"),
        ("src/lib.rs", "src/main.rs"),
        ("a/b/c", "a/b/d"),
        ("file.rs", "file.py"),
    ];
    for (a, b) in pairs {
        assert_ne!(short_hash(a), short_hash(b), "collision: {a} vs {b}");
    }
}

#[test]
fn different_paths_different_redactions() {
    let r1 = redact_path("src/config.json");
    let r2 = redact_path("src/secrets.json");
    // Both end with .json but the hash portion differs
    assert!(r1.ends_with(".json"));
    assert!(r2.ends_with(".json"));
    assert_ne!(r1, r2);
}

// ── Extension handling ─────────────────────────────────────────

#[test]
fn double_extension_preserves_last() {
    let r = redact_path("data.tar.gz");
    assert!(r.ends_with(".gz"));
    assert!(!r.contains("tar"));
}

#[test]
fn dotfile_no_extension() {
    // ".gitignore" is treated as a file with no extension by std::path::Path
    let r = redact_path(".gitignore");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn dotfile_in_dir_no_extension() {
    // "project/.env" — ".env" has no extension per std::path
    let r = redact_path("project/.env");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn extension_with_numbers() {
    let r = redact_path("model.h5");
    assert!(r.ends_with(".h5"));
}

// ── Very long paths ────────────────────────────────────────────

#[test]
fn very_long_path_still_16_char_hash() {
    let long_path = (0..100)
        .map(|i| format!("segment{i}"))
        .collect::<Vec<_>>()
        .join("/")
        + "/file.txt";
    let r = redact_path(&long_path);
    assert!(r.ends_with(".txt"));
    // hash part is always 16 chars
    let hash_part = &r[..16];
    assert!(hash_part.chars().all(|c| c.is_ascii_hexdigit()));
}

// ── Mixed normalization scenarios ──────────────────────────────

#[test]
fn all_normalizations_combined() {
    // backslash + leading ./ + interior /./
    let canonical = short_hash("crates/foo/src/lib.rs");
    let variant = short_hash(r".\crates\.\foo\src\lib.rs");
    assert_eq!(canonical, variant);
}
