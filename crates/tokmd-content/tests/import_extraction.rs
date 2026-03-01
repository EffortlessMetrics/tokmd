//! Tests for import/dependency pattern detection using `count_tags`.
//!
//! `tokmd-content` uses the generic `count_tags` function to detect
//! import-like patterns in source code. These tests verify that typical
//! import statements from Python, JavaScript, and Rust are correctly
//! counted when scanning real file content.

use std::fs::File;
use std::io::Write;
use tokmd_content::{count_tags, read_text_capped};

/// Helper: write content to a temp file, read it back, then count tags.
fn count_tags_in_file(content: &str, tags: &[&str]) -> Vec<(String, usize)> {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("source.txt");
    let mut f = File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();

    let text = read_text_capped(&path, 1_000_000).unwrap();
    count_tags(&text, tags)
}

// ============================================================================
// Python imports
// ============================================================================

#[test]
fn python_import_statement() {
    let code = "\
import os
import sys
import json
";
    let result = count_tags_in_file(code, &["import"]);
    assert_eq!(result[0].1, 3, "should find 3 `import` occurrences");
}

#[test]
fn python_from_import_statement() {
    let code = "\
from os.path import join
from collections import defaultdict
";
    let result = count_tags_in_file(code, &["import", "from"]);
    // "import" appears once per `from X import Y` line
    assert_eq!(result[0].1, 2, "import count");
    assert_eq!(result[1].1, 2, "from count");
}

#[test]
fn python_mixed_imports() {
    let code = "\
import os
from pathlib import Path
import sys
from typing import Optional, List
";
    let result = count_tags_in_file(code, &["import"]);
    // "import" appears on every line (both `import X` and `from X import Y`)
    assert_eq!(result[0].1, 4);
}

// ============================================================================
// JavaScript / TypeScript imports
// ============================================================================

#[test]
fn javascript_import_statements() {
    let code = "\
import React from 'react';
import { useState } from 'react';
import * as path from 'path';
";
    let result = count_tags_in_file(code, &["import"]);
    assert_eq!(result[0].1, 3);
}

#[test]
fn javascript_require_statements() {
    let code = "\
const fs = require('fs');
const path = require('path');
const http = require('http');
";
    let result = count_tags_in_file(code, &["require"]);
    assert_eq!(result[0].1, 3);
}

#[test]
fn javascript_mixed_import_require() {
    let code = "\
import express from 'express';
const lodash = require('lodash');
import { Router } from 'express';
const util = require('util');
";
    let result = count_tags_in_file(code, &["import", "require"]);
    assert_eq!(result[0].1, 2, "import count");
    assert_eq!(result[1].1, 2, "require count");
}

#[test]
fn javascript_dynamic_import() {
    let code = "\
const mod = await import('./module.js');
";
    let result = count_tags_in_file(code, &["import"]);
    assert_eq!(result[0].1, 1);
}

// ============================================================================
// Rust imports
// ============================================================================

#[test]
fn rust_use_statements() {
    let code = "\
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
";
    let result = count_tags_in_file(code, &["use"]);
    assert_eq!(result[0].1, 3);
}

#[test]
fn rust_extern_crate() {
    let code = "\
extern crate serde;
extern crate anyhow;
use serde::Serialize;
";
    let result = count_tags_in_file(code, &["extern crate", "use"]);
    assert_eq!(result[0].1, 2, "extern crate count");
    assert_eq!(result[1].1, 1, "use count");
}

#[test]
fn rust_nested_use() {
    let code = "\
use std::{
    collections::BTreeMap,
    io::{self, Read},
    path::PathBuf,
};
";
    // Only one `use` keyword at the top
    let result = count_tags_in_file(code, &["use"]);
    assert_eq!(result[0].1, 1);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn empty_file_produces_no_imports() {
    let result = count_tags_in_file("", &["import", "require", "use", "from"]);
    for (_tag, count) in &result {
        assert_eq!(*count, 0);
    }
}

#[test]
fn file_with_no_imports() {
    let code = "\
fn main() {
    let x = 42;
    println!(\"hello world\");
}
";
    let result = count_tags_in_file(code, &["import", "require", "from"]);
    for (_tag, count) in &result {
        assert_eq!(*count, 0, "expected 0 import patterns in plain code");
    }
}

#[test]
fn comment_containing_import_is_still_counted() {
    // count_tags is a simple substring matcher; it counts imports in comments too
    let code = "\
// import os  # this is just a comment
fn main() {}
";
    let result = count_tags_in_file(code, &["import"]);
    assert_eq!(
        result[0].1, 1,
        "count_tags counts substrings regardless of context"
    );
}

#[test]
fn string_literal_containing_import_is_counted() {
    // count_tags does not distinguish string literals
    let code = r#"
let msg = "please import this module";
"#;
    let result = count_tags_in_file(code, &["import"]);
    assert_eq!(result[0].1, 1, "count_tags sees substrings in strings too");
}

#[test]
fn multiple_languages_same_file() {
    // Polyglot file with imports from several languages
    let code = "\
import os
const fs = require('fs');
use std::path::Path;
from typing import Dict
";
    let result = count_tags_in_file(code, &["import", "require", "use", "from"]);
    // "import" appears in: `import os`, `from typing import Dict` = 2
    assert_eq!(result[0].1, 2, "import count");
    assert_eq!(result[1].1, 1, "require count");
    assert_eq!(result[2].1, 1, "use count");
    assert_eq!(result[3].1, 1, "from count");
}
