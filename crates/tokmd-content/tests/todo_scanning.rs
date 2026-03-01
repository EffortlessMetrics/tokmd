//! Tests for TODO/FIXME/HACK/XXX tag scanning.
//!
//! Exercises `count_tags` with realistic source files containing
//! various tag patterns, multiline comments, and edge cases.

use std::fs::File;
use std::io::Write;
use tokmd_content::{count_tags, read_text_capped};

/// Helper: write content to a temp file, read it back, then count tags.
fn scan_tags_in_file(content: &str, tags: &[&str]) -> Vec<(String, usize)> {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("code.txt");
    let mut f = File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();

    let text = read_text_capped(&path, 1_000_000).unwrap();
    count_tags(&text, tags)
}

const ALL_TAGS: &[&str] = &["TODO", "FIXME", "HACK", "XXX"];

// ============================================================================
// Basic detection
// ============================================================================

#[test]
fn detects_all_four_standard_tags() {
    let code = "\
// TODO: implement feature
// FIXME: this is broken
// HACK: workaround for issue #123
// XXX: dangerous code
";
    let result = scan_tags_in_file(code, ALL_TAGS);
    assert_eq!(result[0], ("TODO".to_string(), 1));
    assert_eq!(result[1], ("FIXME".to_string(), 1));
    assert_eq!(result[2], ("HACK".to_string(), 1));
    assert_eq!(result[3], ("XXX".to_string(), 1));
}

#[test]
fn detects_multiple_occurrences_of_same_tag() {
    let code = "\
// TODO: first task
fn foo() {}
// TODO: second task
// TODO: third task
";
    let result = scan_tags_in_file(code, &["TODO"]);
    assert_eq!(result[0].1, 3);
}

#[test]
fn no_tags_in_clean_code() {
    let code = "\
fn main() {
    let x = 42;
    println!(\"{}\", x);
}
";
    let result = scan_tags_in_file(code, ALL_TAGS);
    for (_tag, count) in &result {
        assert_eq!(*count, 0);
    }
}

// ============================================================================
// Case sensitivity
// ============================================================================

#[test]
fn case_insensitive_todo_detection() {
    let code = "\
// todo: lowercase
// Todo: titlecase
// TODO: uppercase
// ToDo: mixed
";
    let result = scan_tags_in_file(code, &["TODO"]);
    assert_eq!(result[0].1, 4, "all case variants should be counted");
}

#[test]
fn case_insensitive_fixme_detection() {
    let code = "\
// fixme: lower
// FIXME: upper
// Fixme: title
";
    let result = scan_tags_in_file(code, &["FIXME"]);
    assert_eq!(result[0].1, 3);
}

#[test]
fn case_insensitive_hack_and_xxx() {
    let code = "\
// hack: lower
// HACK: upper
// xxx: lower
// XXX: upper
";
    let result = scan_tags_in_file(code, &["HACK", "XXX"]);
    assert_eq!(result[0].1, 2, "HACK count");
    assert_eq!(result[1].1, 2, "XXX count");
}

// ============================================================================
// Multiline / block comments
// ============================================================================

#[test]
fn tags_in_block_comments() {
    let code = "\
/*
 * TODO: refactor this entire module
 * FIXME: memory leak here
 */
fn leaky() {}
";
    let result = scan_tags_in_file(code, &["TODO", "FIXME"]);
    assert_eq!(result[0].1, 1, "TODO in block comment");
    assert_eq!(result[1].1, 1, "FIXME in block comment");
}

#[test]
fn tags_in_python_docstring() {
    let code = r#"
def complex_function():
    """
    TODO: simplify this algorithm
    HACK: temporary workaround for upstream bug
    """
    pass
"#;
    let result = scan_tags_in_file(code, &["TODO", "HACK"]);
    assert_eq!(result[0].1, 1, "TODO in docstring");
    assert_eq!(result[1].1, 1, "HACK in docstring");
}

#[test]
fn tags_in_html_comments() {
    let code = "\
<!-- TODO: replace placeholder content -->
<div>content</div>
<!-- FIXME: accessibility issue -->
";
    let result = scan_tags_in_file(code, &["TODO", "FIXME"]);
    assert_eq!(result[0].1, 1);
    assert_eq!(result[1].1, 1);
}

// ============================================================================
// False positives / substring matching
// ============================================================================

#[test]
fn todo_in_string_literal_is_counted() {
    // count_tags is substring-based and does NOT distinguish context
    let code = r#"
let msg = "Don't forget: TODO finish this";
"#;
    let result = scan_tags_in_file(code, &["TODO"]);
    assert_eq!(result[0].1, 1, "substring in string literal is counted");
}

#[test]
fn todo_as_variable_name_is_counted() {
    // Substring matching counts `todo_list` and `TODO` alike
    let code = "\
let todo_list = vec![];
";
    let result = scan_tags_in_file(code, &["TODO"]);
    assert_eq!(result[0].1, 1, "substring in identifier is counted");
}

#[test]
fn fixme_embedded_in_longer_word() {
    let code = "\
// unfixmeable problem
";
    let result = scan_tags_in_file(code, &["FIXME"]);
    assert_eq!(result[0].1, 1, "FIXME inside 'unfixmeable' is counted");
}

// ============================================================================
// Adjacent and mixed tags
// ============================================================================

#[test]
fn multiple_tags_on_same_line() {
    let code = "\
// TODO FIXME HACK XXX: everything is broken
";
    let result = scan_tags_in_file(code, ALL_TAGS);
    assert_eq!(result[0].1, 1, "TODO");
    assert_eq!(result[1].1, 1, "FIXME");
    assert_eq!(result[2].1, 1, "HACK");
    assert_eq!(result[3].1, 1, "XXX");
}

#[test]
fn tags_with_colons_and_parens() {
    let code = "\
// TODO: something
// TODO(user): assigned
// FIXME(#123): tracked issue
";
    let result = scan_tags_in_file(code, &["TODO", "FIXME"]);
    assert_eq!(result[0].1, 2, "TODO with different suffixes");
    assert_eq!(result[1].1, 1, "FIXME with issue reference");
}

#[test]
fn empty_file_no_tags() {
    let result = scan_tags_in_file("", ALL_TAGS);
    for (_tag, count) in &result {
        assert_eq!(*count, 0);
    }
}

#[test]
fn whitespace_only_no_tags() {
    let result = scan_tags_in_file("   \n\n\t\t\n", ALL_TAGS);
    for (_tag, count) in &result {
        assert_eq!(*count, 0);
    }
}

// ============================================================================
// Realistic file scanning
// ============================================================================

#[test]
fn realistic_rust_file_with_mixed_tags() {
    let code = "\
use std::collections::HashMap;

/// Module for processing data.
///
/// TODO: add streaming support
pub fn process(data: &[u8]) -> Vec<u8> {
    // FIXME: handle empty input gracefully
    if data.is_empty() {
        return vec![];
    }
    // HACK: pre-allocate oversized buffer to avoid realloc
    let mut result = Vec::with_capacity(data.len() * 2);
    for &byte in data {
        // XXX: this transform is incorrect for non-ASCII
        result.push(byte ^ 0xFF);
    }
    result
}

#[cfg(test)]
mod tests {
    // TODO: add property tests
    // TODO: add benchmark
}
";
    let result = scan_tags_in_file(code, ALL_TAGS);
    assert_eq!(result[0].1, 3, "TODO count");
    assert_eq!(result[1].1, 1, "FIXME count");
    assert_eq!(result[2].1, 1, "HACK count");
    assert_eq!(result[3].1, 1, "XXX count");
}
