//! Static regex patterns for function span detection.

use regex::Regex;
use std::sync::LazyLock;

pub(super) static RUST_FN: LazyLock<Regex> = LazyLock::new(|| {
    // Qualifiers can appear in various orders: pub async unsafe fn, pub unsafe async fn, etc.
    // Identifier aligns with Rust spec: (XID_Start | _) XID_Continue*
    Regex::new(r#"^\s*(pub(\([^)]+\))?\s+)?((async|unsafe|const|extern\s+"[^"]*")\s+)*fn\s+(?:r#)?(?:_|[\p{XID_Start}])\p{XID_Continue}*"#)
        .expect("Static regex must compile")
});

pub(super) static PYTHON_DEF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(async\s+)?def\s+\w+").expect("Static regex must compile"));

pub(super) static JS_FUNCTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(export\s+)?(async\s+)?function\s+\w+").expect("Static regex must compile")
});

pub(super) static JS_ARROW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(export\s+)?(const|let|var)\s+\w+\s*=\s*(async\s+)?\([^)]*\)\s*=>")
        .expect("Static regex must compile")
});

pub(super) static JS_METHOD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(async\s+)?\w+\s*\([^)]*\)\s*\{").expect("Static regex must compile")
});

pub(super) static GO_FUNC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*func\s+\w+").expect("Static regex must compile"));
