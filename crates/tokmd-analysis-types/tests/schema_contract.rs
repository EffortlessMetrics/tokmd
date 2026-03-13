//! Schema versioning and contract tests for tokmd-analysis-types.

use serde_json::Value;
use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, Archetype,
    BASELINE_VERSION, EntropyReport, FunReport, ImportReport,
};
use tokmd_types::{ScanStatus, ToolInfo};

// ── Helpers ──────────────────────────────────────────────────────────────

fn sample_analysis_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "1.0.0".into(),
        },
        mode: "analyze".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: AnalysisSource {
            inputs: vec![".".into()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: "collapse".into(),
        },
        args: AnalysisArgsMeta {
            preset: "receipt".into(),
            format: "json".into(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_commits: None,
            max_commit_files: None,
            max_file_bytes: None,
            import_granularity: "module".into(),
        },
        archetype: None,
        topics: None,
        entropy: None,
        predictive_churn: None,
        corporate_fingerprint: None,
        license: None,
        derived: None,
        assets: None,
        deps: None,
        git: None,
        imports: None,
        dup: None,
        effort: None,
        complexity: None,
        api_surface: None,
        fun: None,
    }
}

// ── Schema version constants ─────────────────────────────────────────────

#[test]
fn analysis_schema_version_is_positive() {
    let v = ANALYSIS_SCHEMA_VERSION;
    assert!(v > 0, "ANALYSIS_SCHEMA_VERSION must be a positive integer");
}

#[test]
fn analysis_schema_version_pinned() {
    assert_eq!(ANALYSIS_SCHEMA_VERSION, 9);
}

#[test]
fn baseline_version_pinned() {
    assert_eq!(BASELINE_VERSION, 1);
}

// ── JSON round-trip ──────────────────────────────────────────────────────

#[test]
fn analysis_receipt_json_roundtrip() {
    let receipt = sample_analysis_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert_eq!(back.mode, "analyze");
    assert!(back.derived.is_none());
    assert!(back.git.is_none());
}

#[test]
fn schema_version_field_in_json() {
    let receipt = sample_analysis_receipt();
    let json = serde_json::to_string(&receipt).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema_version"], ANALYSIS_SCHEMA_VERSION);
}

// ── Enricher output types are serializable ───────────────────────────────

#[test]
fn archetype_serializable() {
    let a = Archetype {
        kind: "monorepo".into(),
        evidence: vec!["multiple crates".into()],
    };
    let json = serde_json::to_string(&a).unwrap();
    let back: Archetype = serde_json::from_str(&json).unwrap();
    assert_eq!(back.kind, "monorepo");
}

#[test]
fn entropy_report_serializable() {
    let r = EntropyReport { suspects: vec![] };
    let json = serde_json::to_string(&r).unwrap();
    let back: EntropyReport = serde_json::from_str(&json).unwrap();
    assert!(back.suspects.is_empty());
}

#[test]
fn import_report_serializable() {
    let r = ImportReport {
        granularity: "module".into(),
        edges: vec![],
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: ImportReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.granularity, "module");
}

#[test]
fn fun_report_serializable() {
    let r = FunReport { eco_label: None };
    let json = serde_json::to_string(&r).unwrap();
    let back: FunReport = serde_json::from_str(&json).unwrap();
    assert!(back.eco_label.is_none());
}

// ── Derived report defaults are valid ────────────────────────────────────

#[test]
fn analysis_receipt_with_all_none_enrichers_roundtrips() {
    let receipt = sample_analysis_receipt();
    let json = serde_json::to_string_pretty(&receipt).unwrap();
    let v: Value = serde_json::from_str(&json).unwrap();

    // All optional enrichers serialize as null
    assert!(v["archetype"].is_null());
    assert!(v["topics"].is_null());
    assert!(v["entropy"].is_null());
    assert!(v["git"].is_null());
    assert!(v["complexity"].is_null());
    assert!(v["fun"].is_null());

    // Can still deserialize back
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, ANALYSIS_SCHEMA_VERSION);
}

// ── Preset names are stable strings ──────────────────────────────────────

#[test]
fn preset_names_stable() {
    let known_presets = [
        "receipt",
        "health",
        "risk",
        "supply",
        "architecture",
        "topics",
        "security",
        "identity",
        "git",
        "deep",
        "fun",
    ];

    // Verify the default preset is "receipt"
    let receipt = sample_analysis_receipt();
    assert_eq!(receipt.args.preset, "receipt");

    // Verify all known presets are strings (contract: presets are plain strings, not enums)
    for preset in &known_presets {
        let mut r = sample_analysis_receipt();
        r.args.preset = preset.to_string();
        let json = serde_json::to_string(&r).unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["args"]["preset"].as_str().unwrap(), *preset);
    }
}

// ── Backward compatibility: extra fields are ignored ─────────────────────

#[test]
fn unknown_fields_in_json_are_tolerated() {
    let mut receipt = sample_analysis_receipt();
    receipt.args.preset = "receipt".into();
    let json = serde_json::to_string(&receipt).unwrap();

    // Add an unknown top-level field
    let mut v: Value = serde_json::from_str(&json).unwrap();
    v["future_field"] = Value::String("hello".into());
    let extended_json = serde_json::to_string(&v).unwrap();

    // Deserialization should succeed (serde default is to ignore unknown fields)
    let back: AnalysisReceipt = serde_json::from_str(&extended_json).unwrap();
    assert_eq!(back.schema_version, ANALYSIS_SCHEMA_VERSION);
}

// ── Property tests ───────────────────────────────────────────────────────

mod properties {
    use proptest::prelude::*;
    use tokmd_analysis_types::{
        ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource,
    };
    use tokmd_types::{ScanStatus, ToolInfo};

    fn arb_analysis_receipt() -> impl Strategy<Value = AnalysisReceipt> {
        (any::<u128>(), "[a-z]{3,8}").prop_map(|(ts, preset)| AnalysisReceipt {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            generated_at_ms: ts,
            tool: ToolInfo {
                name: "tokmd".into(),
                version: "1.0.0".into(),
            },
            mode: "analyze".into(),
            status: ScanStatus::Complete,
            warnings: vec![],
            source: AnalysisSource {
                inputs: vec![".".into()],
                export_path: None,
                base_receipt_path: None,
                export_schema_version: None,
                export_generated_at_ms: None,
                base_signature: None,
                module_roots: vec![],
                module_depth: 2,
                children: "collapse".into(),
            },
            args: AnalysisArgsMeta {
                preset,
                format: "json".into(),
                window_tokens: None,
                git: None,
                max_files: None,
                max_bytes: None,
                max_commits: None,
                max_commit_files: None,
                max_file_bytes: None,
                import_granularity: "module".into(),
            },
            archetype: None,
            topics: None,
            entropy: None,
            predictive_churn: None,
            corporate_fingerprint: None,
            license: None,
            derived: None,
            assets: None,
            deps: None,
            git: None,
            imports: None,
            dup: None,
            effort: None,
            complexity: None,
            api_surface: None,
            fun: None,
        })
    }

    proptest! {
        #[test]
        fn analysis_receipt_roundtrip(receipt in arb_analysis_receipt()) {
            let json = serde_json::to_string(&receipt).unwrap();
            let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(back.schema_version, ANALYSIS_SCHEMA_VERSION);
            prop_assert_eq!(&back.args.preset, &receipt.args.preset);
        }
    }
}
