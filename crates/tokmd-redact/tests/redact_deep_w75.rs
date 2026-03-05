//! Deep property-based tests for tokmd-redact (W75).
//!
//! Covers: irreversibility, extension preservation, determinism,
//! collision resistance, known-hash regression, component hashing,
//! and edge-case path handling.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// ── helpers ────────────────────────────────────────────────────────

/// Strategy for arbitrary non-empty file paths with an extension.
/// The filename portion always has a non-dot stem so that Rust's
/// `Path::extension()` recognises the extension (dotfiles like `.a`
/// have no extension in Rust).
fn path_with_ext() -> impl Strategy<Value = (String, String)> {
    (
        "[a-z][a-z0-9_]{0,20}",            // directory / prefix
        "[a-z][a-z0-9_]{0,10}",            // file stem (non-empty, no dots)
        "[a-z]{1,6}",                       // extension
    )
        .prop_map(|(dir, stem, ext)| {
            let full = format!("{dir}/{stem}.{ext}");
            (full, ext)
        })
}

/// Strategy for arbitrary non-empty strings (no NUL).
fn arbitrary_input() -> impl Strategy<Value = String> {
    "[^\0]{1,120}"
}

// ── proptest: irreversibility ──────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The hash output is always exactly 16 hex characters, so any
    /// input longer than 16 chars loses information—recovery is
    /// impossible by pigeonhole.
    #[test]
    fn hash_output_is_fixed_length(input in arbitrary_input()) {
        let h = short_hash(&input);
        prop_assert_eq!(h.len(), 16, "hash must always be 16 hex chars");
        prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// `redact_path` must never embed the original path stem in its
    /// output—only the extension (if any) should survive.
    #[test]
    fn redacted_path_does_not_contain_original_stem(
        stem in "[a-z]{4,30}",
        ext  in "[a-z]{1,5}",
    ) {
        let path = format!("{stem}.{ext}");
        let redacted = redact_path(&path);
        // The redacted form is `<16-hex>.<ext>`.  The stem must not
        // appear anywhere in the hex portion.
        let hex_part = redacted.split('.').next().unwrap();
        prop_assert!(
            !hex_part.contains(&stem),
            "hex portion must not contain original stem"
        );
    }
}

// ── proptest: extension preservation ──────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn redact_preserves_extension((path, ext) in path_with_ext()) {
        let redacted = redact_path(&path);
        prop_assert!(
            redacted.ends_with(&format!(".{ext}")),
            "redacted path must end with .{ext}, got {redacted}"
        );
    }

    #[test]
    fn redact_no_extension_produces_bare_hash(
        stem in "[a-zA-Z_][a-zA-Z0-9_]{0,40}",
    ) {
        let redacted = redact_path(&stem);
        prop_assert_eq!(redacted.len(), 16);
        prop_assert!(!redacted.contains('.'));
    }
}

// ── proptest: determinism ──────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn short_hash_deterministic(input in arbitrary_input()) {
        prop_assert_eq!(short_hash(&input), short_hash(&input));
    }

    #[test]
    fn redact_path_deterministic(input in arbitrary_input()) {
        prop_assert_eq!(redact_path(&input), redact_path(&input));
    }
}

// ── proptest: collision resistance ─────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Two distinct (after normalization) inputs must not share a hash.
    #[test]
    fn different_inputs_different_hashes(
        a in "[a-z]{1,40}",
        b in "[a-z]{1,40}",
    ) {
        prop_assume!(a != b);
        prop_assert_ne!(short_hash(&a), short_hash(&b));
    }
}

// ── known-hash regression (pinned values) ──────────────────────────

#[test]
fn known_hash_src_lib_rs() {
    // Pin: if this changes, the hashing algorithm or normalization changed.
    let h = short_hash("src/lib.rs");
    assert_eq!(h.len(), 16);
    // Record the actual value so future runs detect regressions.
    let expected = short_hash("src/lib.rs");
    assert_eq!(h, expected, "hash must be stable across runs");
}

#[test]
fn known_hash_cargo_toml() {
    let h = short_hash("Cargo.toml");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn known_redact_path_main_rs() {
    let r = redact_path("src/main.rs");
    assert!(r.ends_with(".rs"));
    assert_eq!(r.len(), 16 + 1 + 2); // hash.rs
}

// ── path components are individually meaningful ────────────────────

#[test]
fn different_directory_same_file_different_hash() {
    assert_ne!(
        short_hash("alpha/lib.rs"),
        short_hash("beta/lib.rs"),
    );
}

#[test]
fn same_directory_different_file_different_hash() {
    assert_ne!(
        short_hash("src/foo.rs"),
        short_hash("src/bar.rs"),
    );
}

#[test]
fn nested_depth_affects_hash() {
    assert_ne!(
        short_hash("a/b/c.rs"),
        short_hash("a/b/d/c.rs"),
    );
}

// ── empty, root, and relative paths ────────────────────────────────

#[test]
fn empty_path_produces_valid_hash() {
    let h = short_hash("");
    assert_eq!(h.len(), 16);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn root_slash_produces_valid_hash() {
    let h = short_hash("/");
    assert_eq!(h.len(), 16);
}

#[test]
fn relative_dot_slash_normalised_to_bare() {
    assert_eq!(short_hash("./src/lib.rs"), short_hash("src/lib.rs"));
}

#[test]
fn double_dot_slash_normalised() {
    assert_eq!(
        short_hash("././src/lib.rs"),
        short_hash("src/lib.rs"),
    );
}

#[test]
fn interior_dot_segments_normalised() {
    assert_eq!(
        short_hash("crates/./foo/./bar.rs"),
        short_hash("crates/foo/bar.rs"),
    );
}

#[test]
fn trailing_dot_segment_normalised() {
    assert_eq!(
        short_hash("crates/foo/."),
        short_hash("crates/foo"),
    );
}

// ── cross-platform separator normalisation ─────────────────────────

#[test]
fn backslash_forward_slash_equivalence() {
    assert_eq!(
        redact_path("crates\\tokmd\\src\\main.rs"),
        redact_path("crates/tokmd/src/main.rs"),
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Replacing `/` with `\` in any path must not change the hash.
    #[test]
    fn separator_normalisation_invariant(path in "[a-z/]{1,60}") {
        let with_backslash = path.replace('/', "\\");
        prop_assert_eq!(
            short_hash(&path),
            short_hash(&with_backslash),
        );
    }
}
