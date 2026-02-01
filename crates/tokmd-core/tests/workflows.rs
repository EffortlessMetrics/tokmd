//! Integration tests for tokmd-core workflows.

use tokmd_core::{
    lang_workflow, module_workflow,
    settings::{LangSettings, ModuleSettings, ScanSettings},
};

#[test]
fn lang_workflow_scans_current_crate() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");

    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(
        !receipt.report.rows.is_empty(),
        "should find some languages"
    );
    // This crate is Rust, so we should find Rust
    assert!(
        receipt.report.rows.iter().any(|r| r.lang == "Rust"),
        "should find Rust in this crate"
    );
}

#[test]
fn module_workflow_scans_current_crate() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow should succeed");

    assert_eq!(receipt.mode, "module");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    // Should have at least one module
    assert!(!receipt.report.rows.is_empty(), "should find some modules");
}

#[test]
fn lang_workflow_respects_top_setting() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings {
        top: 1,
        ..Default::default()
    };

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");

    // Should have at most 2 rows (1 real + 1 "Other" if needed)
    assert!(receipt.report.rows.len() <= 2, "should respect top setting");
}

#[test]
fn scan_settings_excluded_patterns() {
    let scan = ScanSettings {
        paths: vec!["src".to_string()],
        excluded: vec!["**/tests/**".to_string()],
        ..Default::default()
    };
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");

    // Should still work with exclusions
    assert_eq!(receipt.mode, "lang");
}
