//! Determinism regression tests (wave 70).
//!
//! These tests verify the critical invariant: same input → identical output
//! (byte-stable receipts). They use isolated temp directories with controlled
//! fixture files to exercise Unicode paths, nested directories, mixed-case
//! language names, and every output format.
//!
//! Run with: `cargo test -p tokmd --test determinism_w70`

mod common;

use assert_cmd::Command;
use serde_json::Value;
use std::path::Path;

fn tokmd_cmd_at(dir: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir);
    cmd
}

fn tokmd_cmd() -> Command {
    tokmd_cmd_at(common::fixture_root())
}

/// Normalize non-deterministic envelope fields (timestamps, tool version)
/// so byte-level comparison is meaningful.
fn normalize_envelope(output: &str) -> String {
    let re_ts = regex::Regex::new(r#""generated_at_ms":\s*\d+"#).expect("valid regex");
    let s = re_ts
        .replace_all(output, r#""generated_at_ms":0"#)
        .to_string();
    let re_export_ts =
        regex::Regex::new(r#""export_generated_at_ms":\s*\d+"#).expect("valid regex");
    let s = re_export_ts
        .replace_all(&s, r#""export_generated_at_ms":0"#)
        .to_string();
    let re_ver = regex::Regex::new(r#"("tool":\s*\{\s*"name":\s*"tokmd",\s*"version":\s*")[^"]+"#)
        .expect("valid regex");
    re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string()
}

/// Create a temp dir with `.git` marker and controlled fixture files.
fn make_fixture(files: &[(&str, &str)]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir_all(tmp.path().join(".git")).expect("create .git marker");
    for (rel_path, content) in files {
        let full = tmp.path().join(rel_path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("create parent dirs");
        }
        std::fs::write(&full, content).expect("write fixture file");
    }
    tmp
}

/// Recursively verify all JSON object keys are in alphabetical order.
fn assert_keys_sorted(v: &Value, path: &str) {
    match v {
        Value::Object(map) => {
            let keys: Vec<&String> = map.keys().collect();
            for pair in keys.windows(2) {
                assert!(
                    pair[0] <= pair[1],
                    "JSON keys not sorted at {path}: {:?} > {:?}",
                    pair[0],
                    pair[1]
                );
            }
            for (k, val) in map {
                assert_keys_sorted(val, &format!("{path}.{k}"));
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                assert_keys_sorted(val, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

// ===========================================================================
// 1. lang --format json: 10 identical runs
// ===========================================================================

#[test]
fn w70_lang_json_10_runs_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "lang json failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let results: Vec<String> = (0..10).map(|_| run()).collect();
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(&results[0], r, "lang JSON run 0 vs run {i} differ");
    }
}

// ===========================================================================
// 2. module --format json: twice identical
// ===========================================================================

#[test]
fn w70_module_json_twice_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "module json failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "module JSON must be byte-stable across 2 runs");
}

// ===========================================================================
// 3. export --format jsonl: twice identical line count and content
// ===========================================================================

#[test]
fn w70_export_jsonl_twice_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        assert!(o.status.success(), "export jsonl failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    let a_lines: Vec<&str> = a.lines().filter(|l| !l.trim().is_empty()).collect();
    let b_lines: Vec<&str> = b.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        a_lines.len(),
        b_lines.len(),
        "JSONL line count differs: {} vs {}",
        a_lines.len(),
        b_lines.len()
    );
    assert_eq!(a, b, "export JSONL must be byte-stable across 2 runs");
}

// ===========================================================================
// 4. export --format csv: twice identical
// ===========================================================================

#[test]
fn w70_export_csv_twice_identical() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "csv"])
            .output()
            .expect("run");
        assert!(o.status.success(), "export csv failed");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "export CSV must be byte-stable across 2 runs");
}

// ===========================================================================
// 5. JSON keys always alphabetically ordered (BTreeMap invariant)
// ===========================================================================

#[test]
fn w70_lang_json_keys_alphabetically_ordered() {
    let out = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse JSON");
    assert_keys_sorted(&v, "$");
}

#[test]
fn w70_module_json_keys_alphabetically_ordered() {
    let out = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse JSON");
    assert_keys_sorted(&v, "$");
}

#[test]
fn w70_export_json_keys_alphabetically_ordered() {
    let out = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse JSON");
    assert_keys_sorted(&v, "$");
}

// ===========================================================================
// 6. Language rows sorted desc by code, then by name
// ===========================================================================

#[test]
fn w70_lang_rows_sorted_desc_code_then_name() {
    let out = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows array");
    assert!(rows.len() >= 2, "need at least 2 lang rows for sort check");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_name = pair[0]["lang"].as_str().unwrap();
        let b_name = pair[1]["lang"].as_str().unwrap();
        assert!(
            a_code > b_code || (a_code == b_code && a_name <= b_name),
            "lang sort violated: {a_name}({a_code}) before {b_name}({b_code})"
        );
    }
}

// ===========================================================================
// 7. Module rows sorted desc by code, then by name
// ===========================================================================

#[test]
fn w70_module_rows_sorted_desc_code_then_name() {
    let out = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows array");
    assert!(
        rows.len() >= 2,
        "need at least 2 module rows for sort check"
    );

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_name = pair[0]["module"].as_str().unwrap();
        let b_name = pair[1]["module"].as_str().unwrap();
        assert!(
            a_code > b_code || (a_code == b_code && a_name <= b_name),
            "module sort violated: {a_name}({a_code}) before {b_name}({b_code})"
        );
    }
}

// ===========================================================================
// 8. File paths use forward slashes in all output formats
// ===========================================================================

#[test]
fn w70_export_json_paths_use_forward_slashes() {
    let out = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    for row in v["rows"].as_array().expect("rows") {
        let path = row["path"].as_str().unwrap();
        assert!(!path.contains('\\'), "backslash in export JSON path: {path}");
    }
}

#[test]
fn w70_export_jsonl_paths_use_forward_slashes() {
    let out = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if let Some(path) = v.get("path").and_then(|p| p.as_str()) {
                assert!(
                    !path.contains('\\'),
                    "backslash in JSONL path: {path}"
                );
            }
        }
    }
}

#[test]
fn w70_export_csv_paths_use_forward_slashes() {
    let out = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    for (i, line) in text.lines().enumerate().skip(1) {
        if let Some(path) = line.split(',').next() {
            let path = path.trim_matches('"');
            assert!(
                !path.contains('\\'),
                "backslash in CSV path at line {i}: {path}"
            );
        }
    }
}

#[test]
fn w70_module_json_paths_use_forward_slashes() {
    let out = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    for row in v["rows"].as_array().expect("rows") {
        let module = row["module"].as_str().unwrap();
        assert!(
            !module.contains('\\'),
            "backslash in module path: {module}"
        );
    }
}

// ===========================================================================
// 9. No timestamps or random values leak into deterministic output
// ===========================================================================

#[test]
fn w70_lang_json_no_random_values() {
    let out = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    let v: Value = serde_json::from_str(&text).expect("parse");

    // Only `generated_at_ms` and `tool.version` should vary between runs.
    // Check that no `uuid`, `random`, or `nonce` keys exist.
    fn check_no_random_keys(v: &Value, path: &str) {
        if let Value::Object(map) = v {
            for key in map.keys() {
                let lower = key.to_lowercase();
                assert!(
                    !lower.contains("uuid") && !lower.contains("random") && !lower.contains("nonce"),
                    "suspicious non-deterministic key at {path}: {key}"
                );
            }
            for (k, val) in map {
                check_no_random_keys(val, &format!("{path}.{k}"));
            }
        } else if let Value::Array(arr) = v {
            for (i, val) in arr.iter().enumerate() {
                check_no_random_keys(val, &format!("{path}[{i}]"));
            }
        }
    }
    check_no_random_keys(&v, "$");
}

#[test]
fn w70_module_md_no_timestamp_leakage() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "md"])
            .output()
            .expect("run");
        assert!(o.status.success());
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    // Markdown has no envelope, so it should be perfectly byte-identical
    assert_eq!(a, b, "module Markdown must be byte-stable (no timestamp leakage)");
}

#[test]
fn w70_lang_tsv_no_timestamp_leakage() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "tsv"])
            .output()
            .expect("run");
        assert!(o.status.success());
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "lang TSV must be byte-stable (no timestamp leakage)");
}

// ===========================================================================
// 10. Unicode filenames: stable path normalization
// ===========================================================================

#[test]
fn w70_unicode_filenames_stable_export() {
    let tmp = make_fixture(&[
        ("src/main.rs", "fn main() {}\n"),
        ("src/héllo.rs", "fn hello() {}\n"),
        ("src/日本語.rs", "fn jp() {}\n"),
        ("docs/café.md", "# Café\n"),
    ]);
    let run = || {
        let o = tokmd_cmd_at(tmp.path())
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "unicode export failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "Unicode filename export must be byte-stable");
}

#[test]
fn w70_unicode_filenames_forward_slashes() {
    let tmp = make_fixture(&[
        ("src/main.rs", "fn main() {}\n"),
        ("data/résumé.py", "print('hi')\n"),
    ]);
    let out = tokmd_cmd_at(tmp.path())
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    for row in v["rows"].as_array().expect("rows") {
        let path = row["path"].as_str().unwrap();
        assert!(!path.contains('\\'), "backslash in Unicode path: {path}");
    }
}

#[test]
fn w70_unicode_filenames_lang_determinism() {
    let tmp = make_fixture(&[
        ("src/main.rs", "fn main() {\n    println!(\"hello\");\n}\n"),
        ("src/über.rs", "fn uber() {}\n"),
        ("lib/motörhead.js", "console.log('ace');\n"),
    ]);
    let run = || {
        let o = tokmd_cmd_at(tmp.path())
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success());
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let results: Vec<String> = (0..3).map(|_| run()).collect();
    assert_eq!(results[0], results[1], "Unicode lang run 0 vs 1");
    assert_eq!(results[1], results[2], "Unicode lang run 1 vs 2");
}

// ===========================================================================
// 11. Nested directories at various depths
// ===========================================================================

#[test]
fn w70_nested_dirs_export_determinism() {
    let tmp = make_fixture(&[
        ("a.rs", "fn a() {}\n"),
        ("d1/b.rs", "fn b() {}\n"),
        ("d1/d2/c.rs", "fn c() {}\n"),
        ("d1/d2/d3/d.rs", "fn d() {}\n"),
        ("d1/d2/d3/d4/e.rs", "fn e() {}\n"),
        ("d1/d2/d3/d4/d5/f.rs", "fn f() {}\n"),
    ]);
    let run = || {
        let o = tokmd_cmd_at(tmp.path())
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "nested dirs export failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "nested directory export must be byte-stable");
}

#[test]
fn w70_nested_dirs_module_determinism() {
    let tmp = make_fixture(&[
        ("top.rs", "fn top() {}\n"),
        ("alpha/one.rs", "fn one() {}\n"),
        ("alpha/beta/two.rs", "fn two() {}\n"),
        ("alpha/beta/gamma/three.rs", "fn three() {}\n"),
    ]);
    let run = || {
        let o = tokmd_cmd_at(tmp.path())
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "nested dirs module failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "nested directory module must be byte-stable");
}

#[test]
fn w70_nested_dirs_forward_slashes_in_module_names() {
    let tmp = make_fixture(&[
        ("alpha/beta/gamma/code.rs", "fn deep() {}\n"),
    ]);
    let out = tokmd_cmd_at(tmp.path())
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    for row in v["rows"].as_array().expect("rows") {
        let module = row["module"].as_str().unwrap();
        assert!(
            !module.contains('\\'),
            "backslash in nested module path: {module}"
        );
    }
}

#[test]
fn w70_nested_dirs_export_sort_stable() {
    let tmp = make_fixture(&[
        ("x/a.rs", "fn a() { let x = 1; let y = 2; }\n"),
        ("y/b.rs", "fn b() {}\n"),
        ("z/c.rs", "fn c() {}\n"),
    ]);
    let out = tokmd_cmd_at(tmp.path())
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_path = pair[0]["path"].as_str().unwrap();
        let b_path = pair[1]["path"].as_str().unwrap();
        assert!(
            a_code > b_code || (a_code == b_code && a_path <= b_path),
            "nested export sort violated: {a_path}({a_code}) before {b_path}({b_code})"
        );
    }
}

// ===========================================================================
// 12. Mixed-case language names
// ===========================================================================

#[test]
fn w70_mixed_languages_determinism() {
    let tmp = make_fixture(&[
        ("main.rs", "fn main() {\n    println!(\"hello\");\n}\n"),
        ("app.js", "console.log('hello');\nconst x = 1;\n"),
        ("style.css", "body { margin: 0; }\n"),
        ("index.html", "<html><body>Hello</body></html>\n"),
        ("script.py", "print('hello')\nx = 1\n"),
        ("data.json", "{\"key\": \"value\"}\n"),
        ("config.toml", "[section]\nkey = \"value\"\n"),
        ("notes.md", "# Notes\n\nSome content here.\n"),
    ]);
    let run = || {
        let o = tokmd_cmd_at(tmp.path())
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "mixed languages lang failed");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let results: Vec<String> = (0..5).map(|_| run()).collect();
    for (i, r) in results.iter().enumerate().skip(1) {
        assert_eq!(&results[0], r, "mixed languages lang JSON run 0 vs {i}");
    }
}

#[test]
fn w70_mixed_languages_sorted_correctly() {
    let tmp = make_fixture(&[
        ("main.rs", "fn main() {\n    println!(\"hello\");\n}\n"),
        ("app.js", "console.log('hello');\nconst x = 1;\n"),
        ("style.css", "body { margin: 0; }\n"),
        ("index.html", "<html><body>Hello</body></html>\n"),
        ("script.py", "print('hello')\nx = 1\n"),
    ]);
    let out = tokmd_cmd_at(tmp.path())
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).expect("parse");
    let rows = v["rows"].as_array().expect("rows");
    assert!(rows.len() >= 2, "need multiple languages for sort check");

    for pair in rows.windows(2) {
        let a_code = pair[0]["code"].as_u64().unwrap();
        let b_code = pair[1]["code"].as_u64().unwrap();
        let a_name = pair[0]["lang"].as_str().unwrap();
        let b_name = pair[1]["lang"].as_str().unwrap();
        assert!(
            a_code > b_code || (a_code == b_code && a_name <= b_name),
            "mixed-case lang sort violated: {a_name}({a_code}) before {b_name}({b_code})"
        );
    }
}

#[test]
fn w70_mixed_languages_module_determinism() {
    let tmp = make_fixture(&[
        ("src/main.rs", "fn main() {}\n"),
        ("src/lib.rs", "pub fn lib() {}\n"),
        ("web/app.js", "console.log('hello');\n"),
        ("web/style.css", "body { margin: 0; }\n"),
        ("docs/guide.md", "# Guide\n\nContent.\n"),
    ]);
    let run = || {
        let o = tokmd_cmd_at(tmp.path())
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success());
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "mixed-language module JSON must be byte-stable");
}

// ===========================================================================
// Additional determinism scenarios
// ===========================================================================

#[test]
fn w70_export_json_determinism() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success());
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "export JSON must be byte-stable across 2 runs");
}

#[test]
fn w70_lang_md_determinism() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "md"])
            .output()
            .expect("run");
        assert!(o.status.success());
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "lang Markdown must be byte-stable across 2 runs");
}

#[test]
fn w70_module_tsv_determinism() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "tsv"])
            .output()
            .expect("run");
        assert!(o.status.success());
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "module TSV must be byte-stable across 2 runs");
}

#[test]
fn w70_export_jsonl_keys_sorted() {
    let out = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    for (i, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let v: Value = serde_json::from_str(line).unwrap_or_else(|e| {
            panic!("JSONL line {i} is not valid JSON: {e}")
        });
        assert_keys_sorted(&v, &format!("$line[{i}]"));
    }
}

#[test]
fn w70_cross_format_row_count_consistency() {
    let json_out = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    let json: Value = serde_json::from_slice(&json_out.stdout).expect("parse JSON");
    let json_rows = json["rows"].as_array().expect("rows array").len();

    let jsonl_out = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    let jsonl_text = String::from_utf8_lossy(&jsonl_out.stdout);
    let jsonl_rows = jsonl_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| !l.contains(r#""type":"meta""#))
        .count();

    let csv_out = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    let csv_text = String::from_utf8_lossy(&csv_out.stdout);
    let csv_rows = csv_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
        .saturating_sub(1); // header row

    assert!(json_rows > 0, "expected at least one export row");
    assert_eq!(
        json_rows, jsonl_rows,
        "JSON ({json_rows}) vs JSONL ({jsonl_rows}) row count mismatch"
    );
    assert_eq!(
        json_rows, csv_rows,
        "JSON ({json_rows}) vs CSV ({csv_rows}) row count mismatch"
    );
}

#[test]
fn w70_single_file_determinism() {
    let tmp = make_fixture(&[("only.rs", "fn only() {\n    42\n}\n")]);
    let run = || {
        let o = tokmd_cmd_at(tmp.path())
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success());
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "single-file lang JSON must be byte-stable");
}

#[test]
fn w70_empty_dir_determinism() {
    let tmp = make_fixture(&[]);
    // Add a single file so tokmd has something to scan
    std::fs::write(tmp.path().join("placeholder.rs"), "// empty\n").unwrap();
    let run = || {
        let o = tokmd_cmd_at(tmp.path())
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        assert!(o.status.success());
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "minimal-dir lang JSON must be byte-stable");
}
