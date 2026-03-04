//! Wave 48 deep tests for tokmd-redact.
//!
//! Covers BLAKE3 hash determinism, RedactMode-style path redaction,
//! lexical path normalization, forward-slash normalization, property tests
//! for irreversibility and hex validity, edge cases, and collision resistance.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// ============================================================================
// Strategies
// ============================================================================

/// Arbitrary multi-segment path (1–8 segments, alphanumeric + underscore).
fn arb_segments() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_]{1,12}", 1..=8).prop_map(|parts| parts.join("/"))
}

/// Arbitrary path with a known extension.
fn arb_with_ext() -> impl Strategy<Value = (String, String)> {
    (
        arb_segments(),
        prop_oneof![
            Just("rs"),
            Just("py"),
            Just("go"),
            Just("js"),
            Just("ts"),
            Just("json"),
            Just("toml"),
            Just("yaml"),
            Just("c"),
            Just("h"),
        ],
    )
        .prop_map(|(base, ext)| {
            let full = format!("{}.{}", base, ext);
            (full, ext.to_string())
        })
}

/// Arbitrary Windows-style path (backslash separators).
fn arb_win_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9_]{1,10}", 1..=6).prop_map(|parts| parts.join("\\"))
}

// ============================================================================
// 1. BLAKE3 hash determinism: same input → same hash
// ============================================================================

#[test]
fn blake3_determinism_basic() {
    assert_eq!(short_hash("hello"), short_hash("hello"));
}

#[test]
fn blake3_determinism_empty() {
    assert_eq!(short_hash(""), short_hash(""));
}

#[test]
fn blake3_determinism_1000_iterations() {
    let baseline = short_hash("determinism/check.rs");
    for _ in 0..1000 {
        assert_eq!(short_hash("determinism/check.rs"), baseline);
    }
}

#[test]
fn blake3_determinism_redact_path() {
    let r1 = redact_path("crates/tokmd-redact/src/lib.rs");
    let r2 = redact_path("crates/tokmd-redact/src/lib.rs");
    assert_eq!(r1, r2);
}

// ============================================================================
// 2. Path redaction modes (None / Paths / All simulated)
// ============================================================================

#[test]
fn mode_none_no_redaction_preserves_original() {
    // When "no redaction" is desired, caller skips redact_path entirely.
    let original = "src/secrets/config.json";
    assert_eq!(original, "src/secrets/config.json");
}

#[test]
fn mode_paths_redaction_hides_directory_structure() {
    let redacted = redact_path("src/secrets/config.json");
    assert!(!redacted.contains("src"));
    assert!(!redacted.contains("secrets"));
    assert!(!redacted.contains("config"));
    assert!(redacted.ends_with(".json"));
}

#[test]
fn mode_all_redaction_short_hash_hides_everything() {
    let h = short_hash("src/secrets/config.json");
    assert!(!h.contains("src"));
    assert!(!h.contains("json"));
    assert_eq!(h.len(), 16);
}

// ============================================================================
// 3. Lexical path normalization before hashing
// ============================================================================

#[test]
fn normalize_leading_dot_slash() {
    assert_eq!(short_hash("src/lib.rs"), short_hash("./src/lib.rs"));
}

#[test]
fn normalize_double_leading_dot_slash() {
    assert_eq!(short_hash("src/lib.rs"), short_hash("././src/lib.rs"));
}

#[test]
fn normalize_interior_dot_segment() {
    assert_eq!(
        short_hash("crates/foo/src/lib.rs"),
        short_hash("crates/./foo/./src/lib.rs")
    );
}

#[test]
fn normalize_trailing_dot() {
    assert_eq!(short_hash("src"), short_hash("src/."));
}

#[test]
fn normalize_only_dots_and_slashes() {
    let h = short_hash("./././.");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn normalize_redact_path_leading_dot() {
    assert_eq!(redact_path("src/main.rs"), redact_path("./src/main.rs"));
}

#[test]
fn normalize_redact_path_interior_dot() {
    assert_eq!(redact_path("a/b/c.rs"), redact_path("a/./b/./c.rs"));
}

// ============================================================================
// 4. Forward-slash normalization on Windows paths
// ============================================================================

#[test]
fn win_to_unix_short_hash() {
    assert_eq!(short_hash("src\\lib.rs"), short_hash("src/lib.rs"));
}

#[test]
fn win_to_unix_deep_path() {
    assert_eq!(
        short_hash("crates\\tokmd\\src\\commands\\run.rs"),
        short_hash("crates/tokmd/src/commands/run.rs")
    );
}

#[test]
fn win_to_unix_mixed_separators() {
    assert_eq!(
        short_hash("crates/foo\\src/bar\\baz.rs"),
        short_hash("crates/foo/src/bar/baz.rs")
    );
}

#[test]
fn win_to_unix_redact_path() {
    assert_eq!(redact_path("src\\main.rs"), redact_path("src/main.rs"));
}

#[test]
fn win_to_unix_redact_preserves_extension() {
    let r = redact_path("crates\\tokmd\\src\\lib.rs");
    assert!(r.ends_with(".rs"));
}

// ============================================================================
// 5. Property: redacted paths never contain original components
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn prop_redacted_hides_components(
        segments in prop::collection::vec("[a-zA-Z]{3,10}", 2..=5)
    ) {
        let path = format!("{}.rs", segments.join("/"));
        let redacted = redact_path(&path);
        for seg in &segments {
            prop_assert!(
                !redacted.contains(seg.as_str()),
                "Redacted '{}' leaks component '{}' from '{}'",
                redacted, seg, path
            );
        }
    }

    #[test]
    fn prop_short_hash_hides_input(input in "[a-zA-Z]{4,20}") {
        let h = short_hash(&input);
        prop_assert!(
            !h.contains(&input),
            "Hash '{}' contains original input '{}'",
            h, input
        );
    }
}

// ============================================================================
// 6. Property: hash output is always valid hex
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_short_hash_valid_hex(input in "\\PC*") {
        let h = short_hash(&input);
        prop_assert_eq!(h.len(), 16);
        prop_assert!(
            h.chars().all(|c| c.is_ascii_hexdigit()),
            "Not hex: '{}' for input '{:?}'", h, input
        );
    }

    #[test]
    fn prop_short_hash_lowercase(input in "\\PC*") {
        let h = short_hash(&input);
        prop_assert_eq!(h.clone(), h.to_lowercase());
    }

    #[test]
    fn prop_redact_path_hex_prefix(path in arb_with_ext()) {
        let (full, _ext) = path;
        let r = redact_path(&full);
        prop_assert!(r.len() >= 16);
        prop_assert!(
            r[..16].chars().all(|c| c.is_ascii_hexdigit()),
            "Hash prefix not hex in '{}'", r
        );
    }
}

// ============================================================================
// 7. Property: same path always produces same hash
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_short_hash_deterministic(input in "\\PC*") {
        prop_assert_eq!(short_hash(&input), short_hash(&input));
    }

    #[test]
    fn prop_redact_path_deterministic(path in arb_with_ext()) {
        let (full, _) = path;
        prop_assert_eq!(redact_path(&full), redact_path(&full));
    }

    #[test]
    fn prop_triple_invocation_stable(input in "[a-z]{1,30}") {
        let a = short_hash(&input);
        let b = short_hash(&input);
        let c = short_hash(&input);
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(&b, &c);
    }
}

// ============================================================================
// 8. Edge cases: empty, root, deeply nested, Unicode
// ============================================================================

#[test]
fn edge_empty_path() {
    let h = short_hash("");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn edge_empty_redact() {
    let r = redact_path("");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn edge_root_path() {
    let h = short_hash("/");
    assert_eq!(h.len(), 16);
}

#[test]
fn edge_single_char() {
    let h = short_hash("x");
    assert_eq!(h.len(), 16);
}

#[test]
fn edge_deeply_nested_50_levels() {
    let deep = (0..50)
        .map(|i| format!("d{}", i))
        .collect::<Vec<_>>()
        .join("/");
    let h = short_hash(&deep);
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn edge_deeply_nested_redact_preserves_ext() {
    let deep = format!(
        "{}/file.py",
        (0..50)
            .map(|i| format!("d{}", i))
            .collect::<Vec<_>>()
            .join("/")
    );
    let r = redact_path(&deep);
    assert!(r.ends_with(".py"));
    assert_eq!(r.len(), 16 + 1 + 2); // hash + '.' + "py"
}

#[test]
fn edge_unicode_chinese() {
    let h = short_hash("项目/源代码/主.rs");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn edge_unicode_emoji() {
    let h = short_hash("🦀/🔥/lib.rs");
    assert_eq!(h.len(), 16);
}

#[test]
fn edge_unicode_arabic() {
    let h = short_hash("مشروع/ملف.rs");
    assert_eq!(h.len(), 16);
}

#[test]
fn edge_unicode_mixed_scripts() {
    let h1 = short_hash("café/naïve.rs");
    let h2 = short_hash("cafe/naive.rs");
    // Different Unicode => different hash
    assert_ne!(h1, h2);
}

#[test]
fn edge_unicode_redact_preserves_extension() {
    let r = redact_path("données/résumé.json");
    assert!(r.ends_with(".json"));
}

#[test]
fn edge_dotfile_has_no_extension() {
    let r = redact_path(".gitignore");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn edge_hidden_dir_dotfile() {
    let r = redact_path(".config/.env");
    assert_eq!(r.len(), 16);
    assert!(!r.contains('.'));
}

#[test]
fn edge_trailing_dot() {
    let r = redact_path("file.");
    assert_eq!(r.len(), 16);
}

#[test]
fn edge_double_extension() {
    let r = redact_path("archive.tar.gz");
    assert!(r.ends_with(".gz"));
    assert!(!r.contains("tar"));
}

#[test]
fn edge_spaces_in_path() {
    let h = short_hash("path with spaces/file name.rs");
    assert_eq!(h.len(), 16);
}

#[test]
fn edge_null_byte() {
    let h = short_hash("before\0after");
    assert_eq!(h.len(), 16);
}

#[test]
fn edge_newlines() {
    let h = short_hash("line1\nline2\n");
    assert_eq!(h.len(), 16);
}

// ============================================================================
// 9. Collision resistance: different paths → different hashes
// ============================================================================

#[test]
fn collision_single_char_diff() {
    assert_ne!(short_hash("a"), short_hash("b"));
}

#[test]
fn collision_prefix_vs_full() {
    assert_ne!(short_hash("src"), short_hash("src/lib.rs"));
}

#[test]
fn collision_case_sensitive() {
    assert_ne!(short_hash("Src/Lib.rs"), short_hash("src/lib.rs"));
}

#[test]
fn collision_trailing_slash() {
    // "src/" and "src" are different inputs after normalization
    assert_ne!(short_hash("src/"), short_hash("src"));
}

#[test]
fn collision_100_sequential_paths() {
    let mut hashes = std::collections::HashSet::new();
    for i in 0..100 {
        let h = short_hash(&format!("path/to/file_{}.rs", i));
        assert!(hashes.insert(h), "Collision at index {}", i);
    }
}

#[test]
fn collision_similar_paths_differ() {
    assert_ne!(short_hash("src/foo.rs"), short_hash("src/foO.rs"));
    assert_ne!(short_hash("src/bar.rs"), short_hash("src/baz.rs"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_collision_resistance(
        a in "[a-zA-Z0-9_]{1,25}",
        b in "[a-zA-Z0-9_]{1,25}"
    ) {
        prop_assume!(a != b);
        prop_assert_ne!(
            short_hash(&a),
            short_hash(&b),
            "Collision between '{}' and '{}'", a, b
        );
    }

    #[test]
    fn prop_collision_paths(
        a in arb_segments(),
        b in arb_segments()
    ) {
        prop_assume!(a != b);
        prop_assert_ne!(
            short_hash(&a),
            short_hash(&b),
            "Path collision: '{}' vs '{}'", a, b
        );
    }
}

// ============================================================================
// 10. Cross-platform normalization property tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn prop_win_unix_equivalence(path in arb_win_path()) {
        let unix = path.replace('\\', "/");
        prop_assert_eq!(
            short_hash(&path),
            short_hash(&unix),
            "Win/Unix mismatch: '{}' vs '{}'", path, unix
        );
    }

    #[test]
    fn prop_leading_dot_slash_transparent(path in arb_segments()) {
        prop_assert_eq!(
            short_hash(&path),
            short_hash(&format!("./{}", path))
        );
    }

    #[test]
    fn prop_redact_no_separators(path in arb_with_ext()) {
        let (full, _) = path;
        let r = redact_path(&full);
        prop_assert!(!r.contains('/'), "Contains / in '{}'", r);
        prop_assert!(!r.contains('\\'), "Contains \\ in '{}'", r);
    }

    #[test]
    fn prop_redact_extension_preserved(pair in arb_with_ext()) {
        let (full, ext) = pair;
        let r = redact_path(&full);
        prop_assert!(
            r.ends_with(&format!(".{}", ext)),
            "Extension '.{}' not preserved in '{}' for input '{}'",
            ext, r, full
        );
        prop_assert_eq!(r.len(), 16 + 1 + ext.len());
    }
}
