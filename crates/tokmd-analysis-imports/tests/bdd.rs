use tokmd_analysis_imports::{normalize_import_target, parse_imports, supports_language};

#[test]
fn given_supported_language_variants_when_checking_then_support_is_case_insensitive() {
    assert!(supports_language("rust"));
    assert!(supports_language("RUST"));
    assert!(supports_language("TypeScript"));
    assert!(supports_language("PYTHON"));
    assert!(!supports_language("markdown"));
}

#[test]
fn given_rust_use_and_mod_lines_when_parsing_then_module_roots_are_extracted() {
    let lines = vec!["use serde_json::Value;", "mod internal;"];
    let imports = parse_imports("rust", &lines);

    assert_eq!(imports, vec!["serde_json", "internal"]);
}

#[test]
fn given_js_import_and_require_when_parsing_then_targets_are_extracted() {
    let lines = vec![
        r#"import React from "react";"#,
        r#"const util = require("./util/helpers");"#,
    ];
    let imports = parse_imports("javascript", &lines);

    assert_eq!(imports, vec!["react", "./util/helpers"]);
}

#[test]
fn given_python_import_forms_when_parsing_then_module_names_are_extracted() {
    let lines = vec!["import os.path", "from collections import defaultdict"];
    let imports = parse_imports("python", &lines);

    assert_eq!(imports, vec!["os.path", "collections"]);
}

#[test]
fn given_go_block_imports_when_parsing_then_each_target_is_emitted() {
    let lines = vec!["import (", r#""fmt""#, r#""github.com/example/pkg""#, ")"];
    let imports = parse_imports("go", &lines);

    assert_eq!(imports, vec!["fmt", "github.com/example/pkg"]);
}

#[test]
fn given_relative_and_qualified_targets_when_normalizing_then_roots_are_deterministic() {
    assert_eq!(normalize_import_target("./internal/foo"), "local");
    assert_eq!(
        normalize_import_target("github.com/example/service"),
        "github"
    );
    assert_eq!(normalize_import_target("serde_json::Value"), "serde_json");
}

#[test]
fn given_unsupported_language_when_parsing_then_no_imports_are_returned() {
    let lines = vec!["include foo", "link bar"];
    let imports = parse_imports("markdown", &lines);
    assert!(imports.is_empty());
}
