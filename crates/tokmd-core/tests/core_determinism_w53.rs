//! Determinism tests for tokmd-core (w53).
//!
//! Verifies that identical inputs produce identical outputs, field ordering
//! is stable, schema versions are present and non-zero, and timestamps
//! are in a consistent format.

use serde_json::Value;
use tokmd_core::ffi::run_json;
use tokmd_core::settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use tokmd_core::{export_workflow, lang_workflow, module_workflow};

// ============================================================================
// Same scan twice produces identical JSON (ignoring timestamps)
// ============================================================================

fn strip_timestamps(v: &mut Value) {
    if let Some(obj) = v.as_object_mut() {
        obj.remove("generated_at_ms");
        for (_, val) in obj.iter_mut() {
            strip_timestamps(val);
        }
    }
    if let Some(arr) = v.as_array_mut() {
        for val in arr.iter_mut() {
            strip_timestamps(val);
        }
    }
}

#[test]
fn lang_workflow_deterministic() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let r1 = lang_workflow(&scan, &lang).unwrap();
    let r2 = lang_workflow(&scan, &lang).unwrap();

    let mut j1: Value = serde_json::to_value(&r1).unwrap();
    let mut j2: Value = serde_json::to_value(&r2).unwrap();
    strip_timestamps(&mut j1);
    strip_timestamps(&mut j2);

    assert_eq!(j1, j2, "two lang scans of the same input must be identical");
}

#[test]
fn module_workflow_deterministic() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();

    let r1 = module_workflow(&scan, &module).unwrap();
    let r2 = module_workflow(&scan, &module).unwrap();

    let mut j1: Value = serde_json::to_value(&r1).unwrap();
    let mut j2: Value = serde_json::to_value(&r2).unwrap();
    strip_timestamps(&mut j1);
    strip_timestamps(&mut j2);

    assert_eq!(j1, j2, "two module scans must be identical");
}

#[test]
fn export_workflow_deterministic() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let r1 = export_workflow(&scan, &export).unwrap();
    let r2 = export_workflow(&scan, &export).unwrap();

    let mut j1: Value = serde_json::to_value(&r1).unwrap();
    let mut j2: Value = serde_json::to_value(&r2).unwrap();
    strip_timestamps(&mut j1);
    strip_timestamps(&mut j2);

    assert_eq!(j1, j2, "two export scans must be identical");
}

#[test]
fn ffi_lang_deterministic() {
    let args = r#"{"paths":["src"]}"#;
    let r1 = run_json("lang", args);
    let r2 = run_json("lang", args);

    let mut j1: Value = serde_json::from_str(&r1).unwrap();
    let mut j2: Value = serde_json::from_str(&r2).unwrap();
    strip_timestamps(&mut j1);
    strip_timestamps(&mut j2);

    assert_eq!(j1, j2, "FFI lang calls must be deterministic");
}

#[test]
fn ffi_version_deterministic() {
    let r1 = run_json("version", "{}");
    let r2 = run_json("version", "{}");
    // Version has no timestamps, should be byte-identical
    assert_eq!(r1, r2, "version mode must be byte-identical");
}

// ============================================================================
// JSON field ordering is stable
// ============================================================================

#[test]
fn lang_json_key_order_stable() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let r1 = lang_workflow(&scan, &lang).unwrap();
    let r2 = lang_workflow(&scan, &lang).unwrap();

    let s1 = serde_json::to_string(&r1).unwrap();
    let s2 = serde_json::to_string(&r2).unwrap();

    // Extract key order by finding all quoted keys
    let keys1: Vec<&str> = s1.match_indices('"').collect::<Vec<_>>()
        .chunks(2)
        .filter_map(|pair| {
            if pair.len() == 2 {
                Some(&s1[pair[0].0 + 1..pair[1].0])
            } else {
                None
            }
        })
        .take(20)
        .collect();
    let keys2: Vec<&str> = s2.match_indices('"').collect::<Vec<_>>()
        .chunks(2)
        .filter_map(|pair| {
            if pair.len() == 2 {
                Some(&s2[pair[0].0 + 1..pair[1].0])
            } else {
                None
            }
        })
        .take(20)
        .collect();

    assert_eq!(keys1, keys2, "JSON key ordering must be stable");
}

#[test]
fn ffi_module_key_order_stable() {
    let args = r#"{"paths":["src"]}"#;
    let r1 = run_json("module", args);
    let r2 = run_json("module", args);

    let mut j1: Value = serde_json::from_str(&r1).unwrap();
    let mut j2: Value = serde_json::from_str(&r2).unwrap();
    strip_timestamps(&mut j1);
    strip_timestamps(&mut j2);

    // Serialize back to check ordering is identical
    let s1 = serde_json::to_string(&j1).unwrap();
    let s2 = serde_json::to_string(&j2).unwrap();
    assert_eq!(s1, s2, "module JSON serialization must be stable");
}

// ============================================================================
// Schema versions are present and non-zero
// ============================================================================

#[test]
fn lang_schema_version_present_and_nonzero() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).unwrap();
    assert!(receipt.schema_version > 0, "schema_version must be > 0");
}

#[test]
fn module_schema_version_present_and_nonzero() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).unwrap();
    assert!(receipt.schema_version > 0, "schema_version must be > 0");
}

#[test]
fn export_schema_version_present_and_nonzero() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(receipt.schema_version > 0, "schema_version must be > 0");
}

#[test]
fn ffi_version_schema_version_nonzero() {
    let r = run_json("version", "{}");
    let v: Value = serde_json::from_str(&r).unwrap();
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert!(sv > 0, "schema_version in version mode must be > 0");
}

#[test]
fn all_receipts_share_same_schema_version() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    assert_eq!(lang.schema_version, module.schema_version);
    assert_eq!(module.schema_version, export.schema_version);
    assert_eq!(export.schema_version, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// Timestamp format consistency
// ============================================================================

#[test]
fn lang_timestamp_is_millis_since_epoch() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).unwrap();

    // Must be after 2020-01-01 and before 2100-01-01
    let min_ms: u128 = 1_577_836_800_000;
    let max_ms: u128 = 4_102_444_800_000;
    assert!(receipt.generated_at_ms > min_ms, "timestamp too old");
    assert!(receipt.generated_at_ms < max_ms, "timestamp too far future");
}

#[test]
fn module_timestamp_is_millis_since_epoch() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).unwrap();

    let min_ms: u128 = 1_577_836_800_000;
    let max_ms: u128 = 4_102_444_800_000;
    assert!(receipt.generated_at_ms > min_ms);
    assert!(receipt.generated_at_ms < max_ms);
}

#[test]
fn ffi_lang_timestamp_in_json() {
    let r = run_json("lang", r#"{"paths":["src"]}"#);
    let v: Value = serde_json::from_str(&r).unwrap();
    let ts = v["data"]["generated_at_ms"].as_u64().expect("must have generated_at_ms");
    assert!(ts > 1_577_836_800_000, "timestamp too old");
}
