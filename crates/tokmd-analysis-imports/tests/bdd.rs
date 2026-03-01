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

// ── Python edge cases ──────────────────────────────────────────────────

#[test]
fn given_python_relative_from_import_when_parsing_then_dot_prefix_is_captured() {
    let lines = vec!["from . import utils", "from ..models import User"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec![".", "..models"]);
}

#[test]
fn given_python_aliased_import_when_parsing_then_module_root_is_extracted() {
    let lines = vec!["import numpy as np", "import pandas as pd"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["numpy", "pandas"]);
}

#[test]
fn given_python_nested_package_import_when_parsing_then_full_dotted_name_is_returned() {
    let lines = vec!["import os.path", "import xml.etree.ElementTree"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["os.path", "xml.etree.ElementTree"]);
}

#[test]
fn given_python_from_import_star_when_parsing_then_module_is_extracted() {
    let lines = vec![
        "from typing import *",
        "from collections import OrderedDict, defaultdict",
    ];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["typing", "collections"]);
}

#[test]
fn given_python_mixed_indentation_when_parsing_then_leading_spaces_are_stripped() {
    let lines = vec!["    import sys", "  from pathlib import Path"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["sys", "pathlib"]);
}

#[test]
fn given_python_comment_line_when_parsing_then_it_is_not_matched() {
    let lines = vec!["# import os", "import json"];
    let imports = parse_imports("python", &lines);
    assert_eq!(imports, vec!["json"]);
}

// ── JavaScript / TypeScript edge cases ─────────────────────────────────

#[test]
fn given_js_side_effect_import_when_parsing_then_target_is_extracted() {
    let lines = vec![r#"import "./polyfills";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["./polyfills"]);
}

#[test]
fn given_js_named_imports_when_parsing_then_source_is_extracted() {
    let lines = vec![
        r#"import { useState, useEffect } from "react";"#,
        r#"import { readFile } from 'fs/promises';"#,
    ];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["react", "fs/promises"]);
}

#[test]
fn given_js_namespace_import_when_parsing_then_source_is_extracted() {
    let lines = vec![r#"import * as path from "path";"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["path"]);
}

#[test]
fn given_js_require_with_single_quotes_when_parsing_then_target_is_extracted() {
    let lines = vec!["const express = require('express');"];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["express"]);
}

#[test]
fn given_js_dynamic_require_mid_line_when_parsing_then_target_is_extracted() {
    let lines = vec![r#"const cfg = JSON.parse(require("./config.json"));"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["./config.json"]);
}

#[test]
fn given_ts_type_import_when_parsing_then_source_is_extracted() {
    let lines = vec![r#"import type { Config } from "./types";"#];
    let imports = parse_imports("typescript", &lines);
    assert_eq!(imports, vec!["./types"]);
}

#[test]
fn given_ts_default_and_named_import_when_parsing_then_source_is_extracted() {
    let lines = vec![r#"import React, { Component } from "react";"#];
    let imports = parse_imports("typescript", &lines);
    assert_eq!(imports, vec!["react"]);
}

#[test]
fn given_js_import_with_comment_on_same_line_when_parsing_then_source_is_extracted() {
    let lines = vec![r#"import axios from "axios"; // HTTP client"#];
    let imports = parse_imports("javascript", &lines);
    assert_eq!(imports, vec!["axios"]);
}

// ── Rust edge cases ────────────────────────────────────────────────────

#[test]
fn given_rust_crate_and_self_use_when_parsing_then_roots_are_extracted() {
    let lines = vec![
        "use crate::config::Settings;",
        "use self::helpers::parse;",
        "use super::parent_mod;",
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["crate", "self", "super"]);
}

#[test]
fn given_rust_nested_use_block_when_parsing_then_root_crate_is_extracted() {
    let lines = vec!["use std::{io, fs, path::PathBuf};"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn given_rust_extern_crate_when_parsing_then_it_is_not_matched_by_use() {
    // extern crate is not handled — only `use` and `mod`
    let lines = vec!["extern crate serde;", "use anyhow::Result;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["anyhow"]);
}

#[test]
fn given_rust_pub_use_when_parsing_then_not_captured() {
    // `pub use` does not start with "use " so it's excluded
    let lines = vec!["pub use tokmd_types::Receipt;", "use serde::Serialize;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["serde"]);
}

#[test]
fn given_rust_mod_declaration_when_parsing_then_module_name_is_extracted() {
    let lines = vec!["mod analysis;", "mod format;", "mod tests;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["analysis", "format", "tests"]);
}

#[test]
fn given_rust_indented_use_when_parsing_then_it_is_captured() {
    let lines = vec!["    use std::collections::BTreeMap;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

// ── Go edge cases ──────────────────────────────────────────────────────

#[test]
fn given_go_single_line_import_when_parsing_then_target_is_extracted() {
    let lines = vec![r#"import "fmt""#];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt"]);
}

#[test]
fn given_go_aliased_imports_in_block_when_parsing_then_targets_are_extracted() {
    let lines = vec![
        "import (",
        r#"    log "github.com/sirupsen/logrus""#,
        r#"    _ "github.com/lib/pq""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(
        imports,
        vec!["github.com/sirupsen/logrus", "github.com/lib/pq"]
    );
}

#[test]
fn given_go_stdlib_block_when_parsing_then_all_targets_are_extracted() {
    let lines = vec![
        "import (",
        r#""context""#,
        r#""net/http""#,
        r#""encoding/json""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["context", "net/http", "encoding/json"]);
}

#[test]
fn given_go_empty_block_when_parsing_then_no_targets_are_returned() {
    let lines = vec!["import (", ")"];
    let imports = parse_imports("go", &lines);
    assert!(imports.is_empty());
}

#[test]
fn given_go_mixed_single_and_block_when_parsing_then_all_targets_are_extracted() {
    let lines = vec![
        r#"import "os""#,
        "import (",
        r#""strings""#,
        r#""bytes""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["os", "strings", "bytes"]);
}

// ── Normalization edge cases ───────────────────────────────────────────

#[test]
fn given_double_dot_relative_when_normalizing_then_local_is_returned() {
    assert_eq!(normalize_import_target("../sibling/mod"), "local");
    assert_eq!(normalize_import_target("./index"), "local");
}

#[test]
fn given_go_module_path_when_normalizing_then_first_segment_is_root() {
    assert_eq!(
        normalize_import_target("github.com/user/repo/pkg"),
        "github"
    );
}

#[test]
fn given_rust_crate_path_when_normalizing_then_first_segment_before_colon_is_root() {
    assert_eq!(normalize_import_target("serde_json::Value"), "serde_json");
}

#[test]
fn given_slash_separated_path_when_normalizing_then_first_component_is_root() {
    assert_eq!(normalize_import_target("fs/promises"), "fs");
    assert_eq!(normalize_import_target("@scope/package"), "@scope");
}

#[test]
fn given_simple_name_when_normalizing_then_it_is_returned_unchanged() {
    assert_eq!(normalize_import_target("react"), "react");
    assert_eq!(normalize_import_target("os"), "os");
}

#[test]
fn given_quoted_target_when_normalizing_then_quotes_are_stripped() {
    assert_eq!(normalize_import_target(r#""fmt""#), "fmt");
    assert_eq!(normalize_import_target("'lodash'"), "lodash");
}

#[test]
fn given_whitespace_padded_target_when_normalizing_then_it_is_trimmed() {
    assert_eq!(normalize_import_target("  react  "), "react");
    assert_eq!(normalize_import_target("  ./local  "), "local");
}

// ── Empty / degenerate input ───────────────────────────────────────────

#[test]
fn given_empty_lines_when_parsing_any_language_then_no_imports_are_returned() {
    let empty: Vec<&str> = vec![];
    for lang in &["rust", "python", "javascript", "typescript", "go"] {
        assert!(parse_imports(lang, &empty).is_empty(), "failed for {lang}");
    }
}

#[test]
fn given_only_blank_lines_when_parsing_then_no_imports_are_returned() {
    let blanks = vec!["", "   ", "\t"];
    for lang in &["rust", "python", "javascript", "typescript", "go"] {
        assert!(parse_imports(lang, &blanks).is_empty(), "failed for {lang}");
    }
}

#[test]
fn given_all_languages_share_case_insensitive_dispatch() {
    let lines = vec!["import os"];
    assert_eq!(
        parse_imports("Python", &lines),
        parse_imports("python", &lines)
    );
    assert_eq!(
        parse_imports("PYTHON", &lines),
        parse_imports("python", &lines)
    );
}

// ── Rust additional edge cases ─────────────────────────────────────────

#[test]
fn given_rust_glob_use_when_parsing_then_root_is_extracted() {
    let lines = vec!["use std::collections::*;"];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std"]);
}

#[test]
fn given_rust_non_import_lines_when_parsing_then_they_are_skipped() {
    let lines = vec![
        "fn main() {}",
        "let x = 5;",
        "pub struct Foo;",
        "// use fake_import;",
    ];
    let imports = parse_imports("rust", &lines);
    assert!(imports.is_empty());
}

#[test]
fn given_rust_multiple_use_statements_when_parsing_then_all_are_extracted() {
    let lines = vec![
        "use std::io;",
        "use serde::Serialize;",
        "use anyhow::Result;",
    ];
    let imports = parse_imports("rust", &lines);
    assert_eq!(imports, vec!["std", "serde", "anyhow"]);
}

// ── Go additional edge cases ───────────────────────────────────────────

#[test]
fn given_go_block_with_blank_lines_when_parsing_then_blanks_are_skipped() {
    let lines = vec!["import (", "", r#""fmt""#, "", r#""os""#, ")"];
    let imports = parse_imports("go", &lines);
    assert_eq!(imports, vec!["fmt", "os"]);
}

// ── Python additional edge cases ───────────────────────────────────────

#[test]
fn given_python_only_comments_when_parsing_then_no_imports() {
    let lines = vec!["# import os", "# from sys import argv"];
    let imports = parse_imports("python", &lines);
    assert!(imports.is_empty());
}

// ── Determinism explicit test ──────────────────────────────────────────

#[test]
fn given_any_language_parsing_is_deterministic() {
    let rust_lines = vec!["use std::io;", "use serde::Serialize;", "mod internal;"];
    let py_lines = vec![
        "import os",
        "from collections import OrderedDict",
        "import json",
    ];
    let js_lines: Vec<&str> = vec![
        r#"import React from "react";"#,
        r#"const fs = require("fs");"#,
    ];
    let go_lines = vec!["import (", r#""fmt""#, r#""os""#, ")"];

    for _ in 0..3 {
        assert_eq!(
            parse_imports("rust", &rust_lines),
            vec!["std", "serde", "internal"]
        );
        assert_eq!(
            parse_imports("python", &py_lines),
            vec!["os", "collections", "json"]
        );
        assert_eq!(parse_imports("javascript", &js_lines), vec!["react", "fs"]);
        assert_eq!(parse_imports("go", &go_lines), vec!["fmt", "os"]);
    }
}

// ── Normalization additional cases ─────────────────────────────────────

#[test]
fn given_empty_string_when_normalizing_then_result_is_empty() {
    assert_eq!(normalize_import_target(""), "");
}

#[test]
fn given_only_dots_when_normalizing_then_local_is_returned() {
    assert_eq!(normalize_import_target("."), "local");
    assert_eq!(normalize_import_target(".."), "local");
    assert_eq!(normalize_import_target("..."), "local");
}
