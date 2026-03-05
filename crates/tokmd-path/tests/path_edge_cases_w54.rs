//! W54 – Comprehensive path normalization edge-case tests.

use tokmd_path::{normalize_rel_path, normalize_slashes};

// ── Unicode paths ──────────────────────────────────────────────

#[test]
fn unicode_cjk_characters_in_path() {
    assert_eq!(normalize_slashes(r"项目\源码\主.rs"), "项目/源码/主.rs");
    assert_eq!(normalize_rel_path(r".\项目\源码\主.rs"), "项目/源码/主.rs");
}

#[test]
fn unicode_japanese_path() {
    assert_eq!(
        normalize_slashes(r"プロジェクト\src\lib.rs"),
        "プロジェクト/src/lib.rs"
    );
}

#[test]
fn unicode_korean_path() {
    assert_eq!(
        normalize_slashes(r"프로젝트\소스\main.rs"),
        "프로젝트/소스/main.rs"
    );
}

#[test]
fn emoji_in_path() {
    assert_eq!(normalize_slashes(r"🚀\launch\🌍.rs"), "🚀/launch/🌍.rs");
    assert_eq!(normalize_rel_path("./🚀/launch/🌍.rs"), "🚀/launch/🌍.rs");
}

// ── Windows UNC paths ──────────────────────────────────────────

#[test]
fn windows_unc_path_normalizes() {
    let input = r"\\server\share\project\src\main.rs";
    let result = normalize_slashes(input);
    assert_eq!(result, "//server/share/project/src/main.rs");
    assert!(!result.contains('\\'));
}

#[test]
fn windows_unc_with_rel_path() {
    let input = r"\\server\share\.\src\lib.rs";
    let result = normalize_slashes(input);
    assert_eq!(result, "//server/share/./src/lib.rs");
}

// ── Dot segments ───────────────────────────────────────────────

#[test]
fn single_dot_path() {
    assert_eq!(normalize_slashes("."), ".");
    assert_eq!(normalize_rel_path("."), ".");
}

#[test]
fn double_dot_path() {
    assert_eq!(normalize_slashes(".."), "..");
    assert_eq!(normalize_rel_path(".."), "..");
}

#[test]
fn dot_slash_only() {
    // Stripping "./" from "./" leaves empty string
    assert_eq!(normalize_rel_path("./"), "");
}

#[test]
fn multiple_leading_dot_slashes() {
    assert_eq!(normalize_rel_path("./././src/lib.rs"), "src/lib.rs");
    assert_eq!(normalize_rel_path("././././"), "");
}

#[test]
fn parent_dot_dot_preserved() {
    assert_eq!(normalize_rel_path("../src/lib.rs"), "../src/lib.rs");
    assert_eq!(normalize_rel_path("../../a/b.rs"), "../../a/b.rs");
}

#[test]
fn interior_dot_dot_preserved() {
    // normalize_rel_path does NOT resolve ".." segments
    assert_eq!(normalize_rel_path("a/../b/c.rs"), "a/../b/c.rs");
}

// ── Very long paths ────────────────────────────────────────────

#[test]
fn very_long_path_over_260_chars() {
    let segment = "abcdefghij";
    // Build a path > 260 chars: 30 segments × 11 chars each (10 + '/') = 330
    let long_path: String = (0..30).map(|_| segment).collect::<Vec<_>>().join("/");
    assert!(long_path.len() > 260);
    let result = normalize_slashes(&long_path);
    assert_eq!(result, long_path); // already forward slashes
    assert!(!result.contains('\\'));
}

#[test]
fn very_long_path_backslashes() {
    let segment = "abcdefghij";
    let long_path: String = (0..30).map(|_| segment).collect::<Vec<_>>().join("\\");
    assert!(long_path.len() > 260);
    let result = normalize_slashes(&long_path);
    assert!(!result.contains('\\'));
    assert_eq!(result.matches('/').count(), 29);
}

// ── Paths with spaces and special chars ────────────────────────

#[test]
fn path_with_spaces() {
    assert_eq!(
        normalize_slashes(r"my project\src dir\main file.rs"),
        "my project/src dir/main file.rs"
    );
}

#[test]
fn path_with_special_characters() {
    assert_eq!(normalize_slashes(r"a@b#c$d%e\f&g"), "a@b#c$d%e/f&g");
    assert_eq!(
        normalize_rel_path("./[bracket]/file.rs"),
        "[bracket]/file.rs"
    );
}

#[test]
fn path_with_parentheses_and_equals() {
    assert_eq!(normalize_slashes(r"dir(1)\file=2.txt"), "dir(1)/file=2.txt");
}

// ── Empty path components ──────────────────────────────────────

#[test]
fn empty_string_input() {
    assert_eq!(normalize_slashes(""), "");
    assert_eq!(normalize_rel_path(""), "");
}

#[test]
fn consecutive_slashes_preserved() {
    // normalize_slashes only replaces backslashes; does not collapse //
    assert_eq!(normalize_slashes("a//b///c"), "a//b///c");
    assert_eq!(normalize_rel_path("a//b///c"), "a//b///c");
}

// ── Root-only paths ────────────────────────────────────────────

#[test]
fn root_slash_only() {
    assert_eq!(normalize_slashes("/"), "/");
    assert_eq!(normalize_rel_path("/"), "/");
}

#[test]
fn windows_root_drive_letter() {
    assert_eq!(normalize_slashes(r"C:\"), "C:/");
    assert_eq!(normalize_slashes(r"D:\Users\src"), "D:/Users/src");
}

// ── Cross-platform determinism ─────────────────────────────────

#[test]
fn cross_platform_forward_and_backslash_equivalent() {
    let unix = "crates/tokmd/src/lib.rs";
    let win = r"crates\tokmd\src\lib.rs";
    assert_eq!(normalize_slashes(unix), normalize_slashes(win));
}

#[test]
fn cross_platform_rel_path_equivalent() {
    let unix = "./crates/tokmd/src/lib.rs";
    let win = r".\crates\tokmd\src\lib.rs";
    assert_eq!(normalize_rel_path(unix), normalize_rel_path(win));
}

#[test]
fn mixed_separators_normalized() {
    assert_eq!(normalize_slashes(r"a/b\c/d\e"), "a/b/c/d/e");
}
