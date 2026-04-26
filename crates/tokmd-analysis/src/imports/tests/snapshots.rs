//! Snapshot tests for import parsing output using insta.
//!
//! These tests ensure that parsed import lists remain deterministically
//! stable across refactors.

use crate::imports::{normalize_import_target, parse_imports};

// ── Rust snapshot ──────────────────────────────────────────────────

#[test]
fn snapshot_rust_typical_file() {
    let lines = [
        "use std::collections::BTreeMap;",
        "use std::io::{self, Read};",
        "use serde::{Serialize, Deserialize};",
        "use anyhow::{Context, Result};",
        "mod config;",
        "mod utils;",
        "use crate::models::User;",
    ];
    let imports = parse_imports("rust", &lines);
    insta::assert_debug_snapshot!(imports, @r#"
    [
        "std",
        "std",
        "serde",
        "anyhow",
        "config",
        "utils",
        "crate",
    ]
    "#);
}

#[test]
fn snapshot_rust_normalized() {
    let lines = [
        "use std::collections::BTreeMap;",
        "use crate::config::Settings;",
        "use super::parent;",
        "mod child;",
    ];
    let imports = parse_imports("rust", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    insta::assert_debug_snapshot!(normalized, @r#"
    [
        "std",
        "crate",
        "super",
        "child",
    ]
    "#);
}

// ── JavaScript snapshot ────────────────────────────────────────────

#[test]
fn snapshot_js_typical_file() {
    let lines = [
        r#"import React from "react";"#,
        r#"import { useState, useEffect } from "react";"#,
        r#"import axios from "axios";"#,
        r#"import utils from "./utils";"#,
        r#"const fs = require("fs");"#,
        r#"const path = require("path");"#,
    ];
    let imports = parse_imports("javascript", &lines);
    insta::assert_debug_snapshot!(imports, @r#"
    [
        "react",
        "react",
        "axios",
        "./utils",
        "fs",
        "path",
    ]
    "#);
}

#[test]
fn snapshot_js_normalized() {
    let lines = [
        r#"import React from "react";"#,
        r#"import utils from "./utils";"#,
        r#"const fs = require("fs/promises");"#,
    ];
    let imports = parse_imports("javascript", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    insta::assert_debug_snapshot!(normalized, @r#"
    [
        "react",
        "local",
        "fs",
    ]
    "#);
}

// ── Python snapshot ────────────────────────────────────────────────

#[test]
fn snapshot_python_typical_file() {
    let lines = [
        "import os",
        "import sys",
        "from pathlib import Path",
        "from collections import defaultdict, OrderedDict",
        "import numpy as np",
        "from . import utils",
    ];
    let imports = parse_imports("python", &lines);
    insta::assert_debug_snapshot!(imports, @r#"
    [
        "os",
        "sys",
        "pathlib",
        "collections",
        "numpy",
        ".",
    ]
    "#);
}

#[test]
fn snapshot_python_normalized() {
    let lines = [
        "import os.path",
        "from . import utils",
        "from collections import OrderedDict",
        "import numpy",
    ];
    let imports = parse_imports("python", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    insta::assert_debug_snapshot!(normalized, @r#"
    [
        "os",
        "local",
        "collections",
        "numpy",
    ]
    "#);
}

// ── Go snapshot ────────────────────────────────────────────────────

#[test]
fn snapshot_go_typical_file() {
    let lines = [
        "import (",
        r#"    "context""#,
        r#"    "fmt""#,
        r#"    "net/http""#,
        r#"    "encoding/json""#,
        "",
        r#"    "github.com/gorilla/mux""#,
        r#"    "github.com/sirupsen/logrus""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    insta::assert_debug_snapshot!(imports, @r#"
    [
        "context",
        "fmt",
        "net/http",
        "encoding/json",
        "github.com/gorilla/mux",
        "github.com/sirupsen/logrus",
    ]
    "#);
}

#[test]
fn snapshot_go_normalized() {
    let lines = [
        "import (",
        r#""fmt""#,
        r#""github.com/user/repo/pkg""#,
        r#""net/http""#,
        ")",
    ];
    let imports = parse_imports("go", &lines);
    let normalized: Vec<String> = imports.iter().map(|t| normalize_import_target(t)).collect();
    insta::assert_debug_snapshot!(normalized, @r#"
    [
        "fmt",
        "github",
        "net",
    ]
    "#);
}

// ── TypeScript snapshot ────────────────────────────────────────────

#[test]
fn snapshot_ts_typical_file() {
    let lines = [
        r#"import type { Config } from "./config";"#,
        r#"import { Router } from "express";"#,
        r#"import * as path from "path";"#,
    ];
    let imports = parse_imports("typescript", &lines);
    insta::assert_debug_snapshot!(imports, @r#"
    [
        "./config",
        "express",
        "path",
    ]
    "#);
}

// ── Empty / edge-case snapshots ────────────────────────────────────

#[test]
fn snapshot_empty_file_all_languages() {
    let empty: Vec<&str> = vec![];
    let results: Vec<(&str, Vec<String>)> = ["rust", "python", "javascript", "typescript", "go"]
        .iter()
        .map(|lang| (*lang, parse_imports(lang, &empty)))
        .collect();
    insta::assert_debug_snapshot!(results, @r#"
    [
        (
            "rust",
            [],
        ),
        (
            "python",
            [],
        ),
        (
            "javascript",
            [],
        ),
        (
            "typescript",
            [],
        ),
        (
            "go",
            [],
        ),
    ]
    "#);
}

#[test]
fn snapshot_unsupported_language() {
    let lines = ["#include <stdio.h>", "#include <stdlib.h>"];
    let imports = parse_imports("c", &lines);
    insta::assert_debug_snapshot!(imports, @"[]");
}
