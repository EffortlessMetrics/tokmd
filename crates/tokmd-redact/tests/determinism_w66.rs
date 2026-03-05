//! Determinism hardening tests for tokmd-redact.
//!
//! Verifies that BLAKE3-based redaction is deterministic:
//! same path -> same hash, every time, across platforms.

use proptest::prelude::*;
use tokmd_redact::{redact_path, short_hash};

// -- 1. Same path -> same hash (repeated) --

#[test]
fn short_hash_is_deterministic_100_times() {
    let input = "crates/tokmd-redact/src/lib.rs";
    let hashes: Vec<String> = (0..100).map(|_| short_hash(input)).collect();
    assert!(hashes.windows(2).all(|w| w[0] == w[1]));
}

// -- 2. Different paths -> different hashes --

#[test]
fn different_paths_produce_different_hashes() {
    let paths = [
        "src/main.rs",
        "src/lib.rs",
        "Cargo.toml",
        "README.md",
        "tests/test.rs",
        "crates/tokmd/src/main.rs",
        "crates/tokmd-types/src/lib.rs",
    ];
    let hashes: Vec<String> = paths.iter().map(|p| short_hash(p)).collect();
    let mut sorted = hashes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        hashes.len(),
        sorted.len(),
        "collision detected among {} paths",
        paths.len()
    );
}

// -- 3. redact_path preserves extension deterministically --

#[test]
fn redact_path_extension_is_deterministic() {
    let extensions = ["rs", "py", "js", "toml", "json", "md", "yml"];
    for ext in &extensions {
        let path = format!("src/file.{ext}");
        let r1 = redact_path(&path);
        let r2 = redact_path(&path);
        assert_eq!(r1, r2, "redact_path not deterministic for .{ext}");
        assert!(r1.ends_with(&format!(".{ext}")));
    }
}

// -- 4. redact_path no extension -> bare hash --

#[test]
fn redact_path_no_extension_is_deterministic() {
    let r1 = redact_path("Makefile");
    let r2 = redact_path("Makefile");
    assert_eq!(r1, r2);
    assert_eq!(r1.len(), 16);
    assert!(!r1.contains('.'));
}

// -- 5. Separator normalization: forward and back slash --

#[test]
fn hash_normalizes_forward_and_back_slash() {
    assert_eq!(
        short_hash("crates/tokmd/src/lib.rs"),
        short_hash("crates\\tokmd\\src\\lib.rs"),
    );
}

// -- 6. Mixed separator normalization --

#[test]
fn hash_normalizes_mixed_separators() {
    let h1 = short_hash("a/b\\c/d\\e");
    let h2 = short_hash("a/b/c/d/e");
    let h3 = short_hash("a\\b\\c\\d\\e");
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
}

// -- 7. Leading ./ normalization --

#[test]
fn hash_strips_leading_dot_slash() {
    assert_eq!(short_hash("./src/lib.rs"), short_hash("src/lib.rs"));
    assert_eq!(short_hash("././src/lib.rs"), short_hash("src/lib.rs"));
}

// -- 8. Interior /./ normalization --

#[test]
fn hash_resolves_interior_dot_segments() {
    assert_eq!(
        short_hash("crates/./tokmd/./src/lib.rs"),
        short_hash("crates/tokmd/src/lib.rs"),
    );
}

// -- 9. Trailing /. normalization --

#[test]
fn hash_resolves_trailing_dot() {
    assert_eq!(short_hash("src/."), short_hash("src"));
}

// -- 10. redact_path separator normalization --

#[test]
fn redact_path_normalizes_separators() {
    assert_eq!(redact_path("src/main.rs"), redact_path("src\\main.rs"));
}

// -- 11. redact_path with deep nested paths --

#[test]
fn redact_path_deep_path_determinism() {
    let deep = "a/b/c/d/e/f/g/h/i/j/k.rs";
    let r1 = redact_path(deep);
    let r2 = redact_path(deep);
    assert_eq!(r1, r2);
    assert!(r1.ends_with(".rs"));
}

// -- 12. Hash length is always 16 --

#[test]
fn short_hash_length_is_always_16() {
    let inputs = [
        "",
        "a",
        "short",
        "a/very/long/path/to/some/deeply/nested/file.rs",
        "path with spaces/file name.txt",
    ];
    for input in &inputs {
        assert_eq!(
            short_hash(input).len(),
            16,
            "hash length wrong for {input:?}"
        );
    }
}

// -- 13. Hash is pure hex --

#[test]
fn short_hash_is_pure_hex() {
    let inputs = ["src/lib.rs", "Cargo.toml", "README.md"];
    for input in &inputs {
        let h = short_hash(input);
        assert!(
            h.chars().all(|c| c.is_ascii_hexdigit()),
            "hash for {input:?} contains non-hex chars: {h}"
        );
    }
}

// -- 14. Different redact_path outputs for different paths --

#[test]
fn redact_path_different_paths_produce_different_hashes() {
    let r1 = redact_path("src/main.rs");
    let r2 = redact_path("src/lib.rs");
    assert_ne!(r1, r2);
    assert!(r1.ends_with(".rs"));
    assert!(r2.ends_with(".rs"));
}

// -- 15. Double extension only preserves last --

#[test]
fn redact_path_double_extension_preserves_last() {
    let r1 = redact_path("archive.tar.gz");
    let r2 = redact_path("archive.tar.gz");
    assert_eq!(r1, r2);
    assert!(r1.ends_with(".gz"));
}

// -- Property tests --

proptest! {
    #[test]
    fn prop_short_hash_deterministic(path in "\\PC{1,100}") {
        let h1 = short_hash(&path);
        let h2 = short_hash(&path);
        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn prop_short_hash_length(path in "\\PC{0,100}") {
        prop_assert_eq!(short_hash(&path).len(), 16);
    }

    #[test]
    fn prop_short_hash_is_hex(path in "\\PC{0,100}") {
        let h = short_hash(&path);
        prop_assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn prop_redact_path_deterministic(path in "\\PC{1,100}") {
        let r1 = redact_path(&path);
        let r2 = redact_path(&path);
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn prop_separator_normalization(segments in prop::collection::vec("[a-z]{1,8}", 1..5)) {
        let fwd = segments.join("/");
        let back = segments.join("\\");
        prop_assert_eq!(short_hash(&fwd), short_hash(&back));
    }
}
