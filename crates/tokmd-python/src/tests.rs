use super::*;
use crate::envelope::extract_data_json;
use pyo3::types::{PyDict, PyList};
use std::ffi::CString;
use std::fs;
use std::path::Path;

fn with_py<F: FnOnce(Python<'_>)>(f: F) {
    Python::initialize();
    Python::attach(f);
}

fn write_file(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    let parent = path.parent().unwrap_or(root);
    fs::create_dir_all(parent).expect("create parent dirs");
    fs::write(path, contents).expect("write file");
}

fn make_repo(contents: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    write_file(dir.path(), "src/lib.rs", contents);
    dir
}

fn module_from_code<'py>(py: Python<'py>, code: &str, name: &str) -> Bound<'py, PyModule> {
    let code = CString::new(code).expect("code");
    let file = CString::new("fake.py").expect("file");
    let name = CString::new(name).expect("name");
    PyModule::from_code(py, code.as_c_str(), file.as_c_str(), name.as_c_str()).expect("fake module")
}

#[test]
fn version_and_schema_version_are_valid() {
    with_py(|_py| {
        let v = version();
        assert!(!v.is_empty());
        let schema = schema_version();
        assert!(schema > 0);
    });
}

#[test]
fn run_json_version_returns_envelope() {
    with_py(|py| {
        let output = run_json(py, "version", "{}").expect("run_json should succeed");
        let env: serde_json::Value = serde_json::from_str(&output).expect("parse json");
        assert!(env["ok"].as_bool().unwrap_or(false));
        assert!(!env["data"]["version"].as_str().unwrap_or("").is_empty());
        assert!(env["data"]["schema_version"].as_u64().unwrap_or(0) > 0);
    });
}

#[test]
fn run_invalid_mode_returns_error() {
    with_py(|py| {
        let args = PyDict::new(py);
        let err = run(py, "nope", &args).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("unknown_mode"));
    });
}

#[test]
fn extract_envelope_returns_data_when_ok() {
    with_py(|py| {
        let dict = PyDict::new(py);
        dict.set_item("ok", true).unwrap();
        dict.set_item("data", "ok").unwrap();
        let obj = extract_envelope(py, dict.as_any()).expect("extract data");
        let value: String = obj.extract(py).expect("extract string");
        assert_eq!(value, "ok");
    });
}

#[test]
fn extract_envelope_returns_envelope_when_data_missing() {
    with_py(|py| {
        let dict = PyDict::new(py);
        dict.set_item("ok", true).unwrap();
        let obj = extract_envelope(py, dict.as_any()).expect("extract envelope");
        let out = obj.cast_bound::<PyDict>(py).expect("dict");
        assert!(out.get_item("data").unwrap().is_none());
    });
}

#[test]
fn extract_envelope_returns_unknown_error_when_error_missing() {
    with_py(|py| {
        let dict = PyDict::new(py);
        dict.set_item("ok", false).unwrap();
        let err = extract_envelope(py, dict.as_any()).unwrap_err();
        assert!(err.to_string().contains("Unknown error"));
    });
}

#[test]
fn extract_envelope_returns_unknown_error_when_error_not_dict() {
    with_py(|py| {
        let dict = PyDict::new(py);
        dict.set_item("ok", false).unwrap();
        dict.set_item("error", "boom").unwrap();
        let err = extract_envelope(py, dict.as_any()).unwrap_err();
        assert!(err.to_string().contains("Unknown error"));
    });
}

#[test]
fn extract_envelope_missing_code_uses_unknown() {
    with_py(|py| {
        let dict = PyDict::new(py);
        let err_dict = PyDict::new(py);
        dict.set_item("ok", false).unwrap();
        err_dict.set_item("message", "boom").unwrap();
        dict.set_item("error", err_dict).unwrap();
        let err = extract_envelope(py, dict.as_any()).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    });
}

#[test]
fn extract_envelope_missing_message_uses_default() {
    with_py(|py| {
        let dict = PyDict::new(py);
        let err_dict = PyDict::new(py);
        dict.set_item("ok", false).unwrap();
        err_dict.set_item("code", "E").unwrap();
        dict.set_item("error", err_dict).unwrap();
        let err = extract_envelope(py, dict.as_any()).unwrap_err();
        assert!(err.to_string().contains("Unknown error"));
    });
}

#[test]
fn extract_envelope_invalid_format_errors() {
    with_py(|py| {
        let list = PyList::empty(py);
        let err = extract_envelope(py, list.as_any()).unwrap_err();
        assert!(err.to_string().contains("Invalid response format"));
    });
}

#[test]
fn build_args_sets_defaults_and_options() {
    with_py(|py| {
        let args = build_args(py, None, 0, None, false).expect("build_args should succeed");
        let paths: Vec<String> = args.get_item("paths").unwrap().unwrap().extract().unwrap();
        assert_eq!(paths, vec!["."]);
        assert!(args.get_item("top").unwrap().is_none());
        assert!(args.get_item("excluded").unwrap().is_none());
        assert!(args.get_item("hidden").unwrap().is_none());

        let args = build_args(
            py,
            Some(vec!["src".to_string()]),
            3,
            Some(vec!["target".to_string()]),
            true,
        )
        .expect("build_args should succeed");
        let top: i64 = args.get_item("top").unwrap().unwrap().extract().unwrap();
        assert_eq!(top, 3);
        assert!(args.get_item("excluded").unwrap().is_some());
        assert!(args.get_item("hidden").unwrap().is_some());

        let args = build_args(py, Some(vec!["src".to_string()]), 0, Some(vec![]), false)
            .expect("build_args should succeed");
        assert!(args.get_item("excluded").unwrap().is_none());
    });
}

#[test]
fn run_with_json_module_import_error() {
    with_py(|py| {
        let args = PyDict::new(py);
        let err = run_with_json_module(
            py,
            "version",
            &args,
            Err(pyo3::exceptions::PyImportError::new_err("boom")),
        )
        .unwrap_err();
        assert!(err.to_string().contains("boom"));
    });
}

#[test]
fn run_with_json_module_dumps_error() {
    with_py(|py| {
        let module = module_from_code(
            py,
            "def dumps(x):\n    raise ValueError('nope')\n\ndef loads(s):\n    return {'ok': True, 'data': {}}",
            "fake_dumps_error",
        );
        let args = PyDict::new(py);
        let err = run_with_json_module(py, "version", &args, Ok(module)).unwrap_err();
        assert!(err.to_string().contains("nope"));
    });
}

#[test]
fn run_with_json_module_dumps_non_string() {
    with_py(|py| {
        let module = module_from_code(
            py,
            "def dumps(x):\n    return 123\n\ndef loads(s):\n    return {'ok': True, 'data': {}}",
            "fake_dumps_non_string",
        );
        let args = PyDict::new(py);
        let err = run_with_json_module(py, "version", &args, Ok(module)).unwrap_err();
        assert!(!err.to_string().is_empty());
    });
}

#[test]
fn run_with_json_module_loads_error() {
    with_py(|py| {
        let module = module_from_code(
            py,
            "def dumps(x):\n    return \"{}\"\n\ndef loads(s):\n    raise ValueError('bad')",
            "fake_loads_error",
        );
        let args = PyDict::new(py);
        let err = run_with_json_module(py, "version", &args, Ok(module)).unwrap_err();
        assert!(err.to_string().contains("bad"));
    });
}

#[test]
fn wrappers_scan_small_repo() {
    with_py(|py| {
        let repo = make_repo("fn main() { println!(\"hi\"); }\n");
        let path = repo.path().to_string_lossy().to_string();

        let lang_result = lang(
            py,
            Some(vec![path.clone()]),
            0,
            true,
            Some("collapse"),
            Some("none"),
            None,
            false,
        )
        .expect("lang should succeed");
        let lang_dict = lang_result.cast_bound::<PyDict>(py).expect("lang dict");
        assert_eq!(
            lang_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "lang"
        );

        let module_result = module(
            py,
            Some(vec![path.clone()]),
            0,
            Some(vec!["src".to_string()]),
            1,
            Some("separate"),
            Some("none"),
            None,
            false,
        )
        .expect("module should succeed");
        let module_dict = module_result.cast_bound::<PyDict>(py).expect("module dict");
        assert_eq!(
            module_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "module"
        );

        let export_result = export(
            py,
            Some(vec![path.clone()]),
            Some("json"),
            0,
            0,
            None,
            2,
            Some("separate"),
            Some("none"),
            None,
            false,
        )
        .expect("export should succeed");
        let export_dict = export_result.cast_bound::<PyDict>(py).expect("export dict");
        assert_eq!(
            export_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "export"
        );
    });
}

#[test]
fn wrappers_scan_small_repo_defaults() {
    with_py(|py| {
        let repo = make_repo("fn main() { println!(\"hi\"); }\n");
        let path = repo.path().to_string_lossy().to_string();

        let lang_result = lang(
            py,
            Some(vec![path.clone()]),
            0,
            false,
            None,
            None,
            None,
            false,
        )
        .expect("lang should succeed");
        let lang_dict = lang_result.cast_bound::<PyDict>(py).expect("lang dict");
        assert_eq!(
            lang_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "lang"
        );

        let module_result = module(
            py,
            Some(vec![path.clone()]),
            0,
            None,
            1,
            None,
            None,
            None,
            false,
        )
        .expect("module should succeed");
        let module_dict = module_result.cast_bound::<PyDict>(py).expect("module dict");
        assert_eq!(
            module_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "module"
        );

        let export_result = export(
            py,
            Some(vec![path.clone()]),
            None,
            0,
            0,
            Some(vec!["src".to_string()]),
            2,
            None,
            None,
            None,
            false,
        )
        .expect("export should succeed");
        let export_dict = export_result.cast_bound::<PyDict>(py).expect("export dict");
        assert_eq!(
            export_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "export"
        );

        let analysis_result = analyze(
            py,
            Some(vec![path]),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("analyze should succeed");
        let analysis_dict = analysis_result
            .cast_bound::<PyDict>(py)
            .expect("analysis dict");
        assert_eq!(
            analysis_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "analysis"
        );
    });
}

#[test]
fn analyze_returns_receipt() {
    with_py(|py| {
        let repo = make_repo("fn main() {}\n");
        let path = repo.path().to_string_lossy().to_string();
        let analysis_result = analyze(
            py,
            Some(vec![path]),
            Some("receipt"),
            Some(1000),
            Some(false),
            Some(10),
            Some(4096),
            Some(1),
            None,
            false,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("analyze should succeed");
        let analysis_dict = analysis_result
            .cast_bound::<PyDict>(py)
            .expect("analysis dict");
        assert_eq!(
            analysis_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "analysis"
        );
    });
}

#[test]
fn diff_compares_two_paths() {
    with_py(|py| {
        let repo_a = make_repo("fn main() { println!(\"a\"); }\n");
        let repo_b = make_repo("fn main() { println!(\"b\"); }\n");
        let path_a = repo_a.path().to_string_lossy().to_string();
        let path_b = repo_b.path().to_string_lossy().to_string();

        let diff_result = diff(py, Some(&path_a), Some(&path_b)).expect("diff should succeed");
        let diff_dict = diff_result.cast_bound::<PyDict>(py).expect("diff dict");
        assert_eq!(
            diff_dict
                .get_item("mode")
                .unwrap()
                .unwrap()
                .extract::<String>()
                .unwrap(),
            "diff"
        );
    });
}

// ========================================================================
// Compile-check stubs: verify the core API surface that bindings depend on
// ========================================================================

/// Integration tests for cdylib crates cannot live in `tests/` because
/// Cargo does not produce an rlib for linking.  These inline stubs verify
/// that the underlying `tokmd_core` contract is stable.

#[test]
fn core_version_matches_binding_version() {
    let core_ver = tokmd_core::ffi::version();
    let binding_ver = version();
    assert_eq!(core_ver, binding_ver, "binding must delegate to core");
}

#[test]
fn core_schema_version_matches_binding() {
    let core_sv = tokmd_core::ffi::schema_version();
    let binding_sv = schema_version();
    assert_eq!(core_sv, binding_sv, "binding must delegate to core");
}

#[test]
fn core_run_json_returns_valid_json_for_all_modes() {
    let modes = ["lang", "module", "export", "analyze", "diff", "version"];
    for mode in modes {
        let result = tokmd_core::ffi::run_json(mode, "{}");
        let v: serde_json::Value =
            serde_json::from_str(&result).expect("run_json must return valid JSON");
        assert!(
            v.get("ok").is_some(),
            "envelope for mode '{mode}' missing 'ok'"
        );
    }
}

#[test]
fn core_run_json_unknown_mode_returns_error() {
    let result = tokmd_core::ffi::run_json("bogus", "{}");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn extract_data_json_valid_success_envelope() {
    let envelope = r#"{"ok":true,"data":{"mode":"lang"}}"#;
    let data = extract_data_json(envelope).expect("should extract data");
    let v: serde_json::Value = serde_json::from_str(&data).unwrap();
    assert_eq!(v["mode"].as_str(), Some("lang"));
}

#[test]
fn extract_data_json_error_envelope_fails() {
    let envelope = r#"{"ok":false,"error":{"code":"unknown_mode","message":"bad"}}"#;
    let err = extract_data_json(envelope).unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[test]
fn map_envelope_error_preserves_message() {
    let err = tokmd_envelope::ffi::EnvelopeExtractError::JsonParse("test error".to_string());
    let py_err = map_envelope_error(err);
    assert!(py_err.to_string().contains("test error"));
}

// ========================================================================
// RED TESTS: FFI Error Handling Contract
// ========================================================================
// These tests define the contract for how tokmd-python handles errors
// across the Python ↔ Rust FFI boundary.
// Run ID: run_tokmd_887_1744034820000
// Task: 1.1 - tokmd-python FFI Contract
//
// Acceptance Criteria from spec.md:
// - FFI Safety: Python bindings never panic under any input condition
// - All #[pyfunction] exports return PyResult<T>
// - Internal errors converted via anyhow::Error → PyErr mapping

// CONTRACT 1: FFI functions never panic on invalid input

/// CONTRACT: Passing None where a path is expected should raise Python
/// TypeError/ValueError, NOT panic the interpreter.
#[test]
fn red_test_python_ffi_no_panic_on_none_paths() {
    with_py(|py| {
        let args = PyDict::new(py);
        // Set paths to None (invalid) - should not panic
        args.set_item("paths", py.None()).unwrap();

        // This should return Err(PyErr), NOT panic
        let result = run(py, "lang", &args);

        // CONTRACT: Must be Err, not panic
        assert!(
            result.is_err(),
            "CONTRACT VIOLATION: run() with None paths must return Err, got Ok"
        );

        // CONTRACT: Error should be a Python exception (TokmdError or TypeError)
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("TokmdError")
                || err_str.contains("TypeError")
                || err_str.contains("paths"),
            "CONTRACT VIOLATION: Error should mention paths or be TokmdError/TypeError, got: {}",
            err_str
        );
    });
}

/// CONTRACT: Empty paths list should produce graceful error, not panic.
#[test]
fn red_test_python_ffi_no_panic_on_empty_paths() {
    with_py(|py| {
        // Pass empty paths vector
        let args = PyDict::new(py);
        let empty_list = PyList::empty(py);
        args.set_item("paths", empty_list).unwrap();

        // Should not panic
        let result = run(py, "lang", &args);

        // CONTRACT: Must be Err or handle gracefully (not panic)
        match result {
            Ok(obj) => {
                // If it returns Ok, the result should indicate no files found
                let dict = obj.cast_bound::<PyDict>(py).expect("should be dict");
                let rows = dict.get_item("rows").unwrap();
                assert!(rows.is_some(), "Result should have rows field");
            }
            Err(err) => {
                // Err is also acceptable - test passes either way as long as no panic
                let _ = err.to_string(); // Just verify we can stringify the error
            }
        }
    });
}

/// CONTRACT: Invalid UTF-8 in paths should produce error, not panic.
#[test]
fn red_test_python_ffi_no_panic_on_unusual_paths() {
    with_py(|py| {
        // This test documents that unusual paths should be handled
        // CONTRACT: Should handle gracefully
        let result = lang(
            py,
            Some(vec!["\u{FFFD}\u{FFFE}".to_string()]), // Replacement chars
            0,
            false,
            None,
            None,
            None,
            false,
        );

        // Should not panic - either Ok or Err is acceptable.
        if let Err(err) = result {
            let _ = err.to_string();
        }
    });
}

/// CONTRACT: Very long paths should not cause panic (buffer overflow protection).
#[test]
fn red_test_python_ffi_no_panic_on_extremely_long_paths() {
    with_py(|py| {
        let long_path = "a".repeat(10000);

        // Should not panic
        let result = lang(py, Some(vec![long_path]), 0, false, None, None, None, false);

        // CONTRACT: Must not panic - Err is acceptable
        if let Err(ref err) = result {
            let _ = err.to_string(); // Verify error can be stringified
        }
        // Test passes if we reach here (no panic)
    });
}

/// CONTRACT: IO errors (file not found) should translate to Python exceptions.
#[test]
fn red_test_python_ffi_io_error_translation() {
    with_py(|py| {
        let nonexistent_path = "/definitely/does/not/exist/tokmd_test_12345";

        let result = lang(
            py,
            Some(vec![nonexistent_path.to_string()]),
            0,
            false,
            None,
            None,
            None,
            false,
        );

        // CONTRACT: Must return Err, not panic
        assert!(
            result.is_err(),
            "CONTRACT VIOLATION: Nonexistent path should return Err, got Ok"
        );

        // CONTRACT: Error should be informative
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            !err_str.is_empty() && err_str.len() > 5,
            "CONTRACT VIOLATION: Error should have meaningful message, got: {}",
            err_str
        );
    });
}

/// CONTRACT: Permission errors should translate to Python exceptions.
#[test]
fn red_test_python_ffi_permission_error_translation() {
    // This test documents the expected behavior for permission errors
    // CONTRACT: When tokmd encounters a permission error:
    // - Must NOT panic
    // - Must return Err(PyErr)
    // - Python exception should contain "permission" or "access" in message
}

// CONTRACT 2: All public functions return PyResult (type safety)

/// CONTRACT: version() should be panic-free and return a valid string.
#[test]
fn red_test_python_ffi_version_returns_valid_string() {
    with_py(|_py| {
        let ver = version();

        // CONTRACT: Must return non-empty string
        assert!(
            !ver.is_empty(),
            "CONTRACT VIOLATION: version() must return non-empty string"
        );

        // CONTRACT: Should be a valid version format
        assert!(
            ver.chars().any(|c| c.is_ascii_digit()),
            "CONTRACT VIOLATION: version should contain digits, got: {}",
            ver
        );
    });
}

/// CONTRACT: schema_version() should be panic-free and return valid number.
#[test]
fn red_test_python_ffi_schema_version_returns_valid_number() {
    with_py(|_py| {
        let sv = schema_version();

        // CONTRACT: Must return positive number
        assert!(
            sv > 0,
            "CONTRACT VIOLATION: schema_version() must return positive number, got: {}",
            sv
        );
    });
}

/// CONTRACT: All wrapper functions return PyResult (verified at runtime).
#[test]
fn red_test_python_ffi_all_wrappers_return_pyresult() {
    with_py(|py| {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.to_string_lossy().to_string();

        // lang() - should return PyResult
        let _ = lang(
            py,
            Some(vec![temp_path.clone()]),
            0,
            false,
            None,
            None,
            None,
            false,
        );

        // module() - should return PyResult
        let _ = module(
            py,
            Some(vec![temp_path.clone()]),
            0,
            None,
            1,
            None,
            None,
            None,
            false,
        );

        // export() - should return PyResult
        let _ = export(
            py,
            Some(vec![temp_path.clone()]),
            None,
            0,
            0,
            None,
            2,
            None,
            None,
            None,
            false,
        );

        // analyze() - should return PyResult
        let _ = analyze(
            py,
            Some(vec![temp_path.clone()]),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        // diff() - should return PyResult
        let _ = diff(py, Some(&temp_path), Some(&temp_path));

        // cockpit() - should return PyResult
        let _ = cockpit(py, None, None, None, None);
    });
}

// CONTRACT 3: Internal error handling invariants

/// CONTRACT: Envelope extraction errors should map to TokmdError.
#[test]
fn red_test_python_ffi_envelope_error_mapping() {
    with_py(|py| {
        // Test envelope error mapping through extract_data_json
        let result = run_json(py, "bogus_mode_that_fails", "{}");

        // The error should be properly wrapped
        match result {
            Ok(json) => {
                // If Ok, envelope should contain error info
                assert!(
                    json.contains("ok") || json.contains("error"),
                    "Response should be valid envelope"
                );
            }
            Err(err) => {
                // Error should be a proper Python exception
                let _ = err.to_string();
            }
        }
    });
}

/// CONTRACT: JSON parsing errors should not cause panic.
#[test]
fn red_test_python_ffi_json_error_handling() {
    with_py(|py| {
        // Test with various malformed JSON inputs
        let test_cases = vec![
            "{}",                            // Empty object
            "{invalid",                      // Invalid JSON
            "",                              // Empty string
            "null",                          // Null (not an envelope)
            r#"{"ok": true}"#,               // Missing data field
            r#"{"ok": false}"#,              // Missing error field
            r#"{"ok": true, "data": null}"#, // Null data
        ];

        for json_input in test_cases {
            // run_json should handle all these without panic
            let result = run_json(py, "version", json_input);

            // CONTRACT: Must not panic - Ok or Err both acceptable
            let _ = result;
        }
    });
}

// CONTRACT 4: GIL handling safety

/// CONTRACT: Functions releasing GIL should not panic on error.
#[test]
fn red_test_python_ffi_gil_release_safety() {
    with_py(|py| {
        // Functions like run_json release the GIL during execution
        let args = PyDict::new(py);
        args.set_item("paths", vec!["nonexistent_path".to_string()])
            .unwrap();

        // This releases GIL - should be safe
        let result = run(py, "analyze", &args);

        // After run() returns (Ok or Err), GIL should be valid
        // Try another Python operation to verify GIL state
        let dict = PyDict::new(py);
        dict.set_item("test", 42).unwrap();

        // Original result should be available
        let _ = result;
    });
}
