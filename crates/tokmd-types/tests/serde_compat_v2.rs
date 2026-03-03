//! Backward-compatibility tests for serde aliases and rename conventions.
//!
//! Every `#[serde(alias = "...")]` and `#[serde(rename_all = "...")]` in
//! tokmd-types is exercised here to guard against accidental breakage when
//! field names or enum variants are refactored.

use serde_json::{Value, json};
use tokmd_types::cockpit::*;
use tokmd_types::*;

// ═══════════════════════════════════════════════════════════════════════════
// TokenEstimationMeta — field aliases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn token_estimation_meta_alias_tokens_high_deserializes_as_tokens_min() {
    let old_json = json!({
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_high": 800,
        "tokens_est": 1000,
        "tokens_low": 1334,
        "source_bytes": 4000
    });
    let meta: TokenEstimationMeta = serde_json::from_value(old_json).unwrap();
    assert_eq!(meta.tokens_min, 800);
    assert_eq!(meta.tokens_max, 1334);
}

#[test]
fn token_estimation_meta_current_names_still_work() {
    let new_json = json!({
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_min": 800,
        "tokens_est": 1000,
        "tokens_max": 1334,
        "source_bytes": 4000
    });
    let meta: TokenEstimationMeta = serde_json::from_value(new_json).unwrap();
    assert_eq!(meta.tokens_min, 800);
    assert_eq!(meta.tokens_max, 1334);
}

#[test]
fn token_estimation_meta_old_and_new_produce_identical_struct() {
    let old_json = json!({
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_high": 800,
        "tokens_est": 1000,
        "tokens_low": 1334,
        "source_bytes": 4000
    });
    let new_json = json!({
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_min": 800,
        "tokens_est": 1000,
        "tokens_max": 1334,
        "source_bytes": 4000
    });
    let from_old: TokenEstimationMeta = serde_json::from_value(old_json).unwrap();
    let from_new: TokenEstimationMeta = serde_json::from_value(new_json).unwrap();
    assert_eq!(from_old.tokens_min, from_new.tokens_min);
    assert_eq!(from_old.tokens_max, from_new.tokens_max);
    assert_eq!(from_old.tokens_est, from_new.tokens_est);
    assert_eq!(from_old.source_bytes, from_new.source_bytes);
}

#[test]
fn token_estimation_meta_reserializes_with_current_name() {
    let old_json = json!({
        "bytes_per_token_est": 4.0,
        "bytes_per_token_low": 3.0,
        "bytes_per_token_high": 5.0,
        "tokens_high": 800,
        "tokens_est": 1000,
        "tokens_low": 1334,
        "source_bytes": 4000
    });
    let meta: TokenEstimationMeta = serde_json::from_value(old_json).unwrap();
    let reserialized: Value = serde_json::to_value(&meta).unwrap();
    // After round-trip, the canonical names must be used
    assert!(
        reserialized.get("tokens_min").is_some(),
        "must emit tokens_min"
    );
    assert!(
        reserialized.get("tokens_max").is_some(),
        "must emit tokens_max"
    );
    assert!(
        reserialized.get("tokens_high").is_none(),
        "must NOT emit old alias tokens_high"
    );
    assert!(
        reserialized.get("tokens_low").is_none(),
        "must NOT emit old alias tokens_low"
    );
    assert_eq!(reserialized["tokens_min"], 800);
    assert_eq!(reserialized["tokens_max"], 1334);
}

// ═══════════════════════════════════════════════════════════════════════════
// TokenAudit — field aliases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn token_audit_alias_tokens_high_deserializes_as_tokens_min() {
    let old_json = json!({
        "output_bytes": 5000,
        "tokens_high": 1000,
        "tokens_est": 1250,
        "tokens_low": 1667,
        "overhead_bytes": 500,
        "overhead_pct": 0.1
    });
    let audit: TokenAudit = serde_json::from_value(old_json).unwrap();
    assert_eq!(audit.tokens_min, 1000);
    assert_eq!(audit.tokens_max, 1667);
}

#[test]
fn token_audit_current_names_still_work() {
    let new_json = json!({
        "output_bytes": 5000,
        "tokens_min": 1000,
        "tokens_est": 1250,
        "tokens_max": 1667,
        "overhead_bytes": 500,
        "overhead_pct": 0.1
    });
    let audit: TokenAudit = serde_json::from_value(new_json).unwrap();
    assert_eq!(audit.tokens_min, 1000);
    assert_eq!(audit.tokens_max, 1667);
}

#[test]
fn token_audit_old_and_new_produce_identical_struct() {
    let old_json = json!({
        "output_bytes": 5000,
        "tokens_high": 1000,
        "tokens_est": 1250,
        "tokens_low": 1667,
        "overhead_bytes": 500,
        "overhead_pct": 0.1
    });
    let new_json = json!({
        "output_bytes": 5000,
        "tokens_min": 1000,
        "tokens_est": 1250,
        "tokens_max": 1667,
        "overhead_bytes": 500,
        "overhead_pct": 0.1
    });
    let from_old: TokenAudit = serde_json::from_value(old_json).unwrap();
    let from_new: TokenAudit = serde_json::from_value(new_json).unwrap();
    assert_eq!(from_old.tokens_min, from_new.tokens_min);
    assert_eq!(from_old.tokens_max, from_new.tokens_max);
    assert_eq!(from_old.tokens_est, from_new.tokens_est);
    assert_eq!(from_old.output_bytes, from_new.output_bytes);
}

#[test]
fn token_audit_reserializes_with_current_name() {
    let old_json = json!({
        "output_bytes": 5000,
        "tokens_high": 1000,
        "tokens_est": 1250,
        "tokens_low": 1667,
        "overhead_bytes": 500,
        "overhead_pct": 0.1
    });
    let audit: TokenAudit = serde_json::from_value(old_json).unwrap();
    let reserialized: Value = serde_json::to_value(&audit).unwrap();
    assert!(reserialized.get("tokens_min").is_some());
    assert!(reserialized.get("tokens_max").is_some());
    assert!(reserialized.get("tokens_high").is_none());
    assert!(reserialized.get("tokens_low").is_none());
    assert_eq!(reserialized["tokens_min"], 1000);
    assert_eq!(reserialized["tokens_max"], 1667);
}

// ═══════════════════════════════════════════════════════════════════════════
// Enum rename_all = "snake_case" — tokmd-types core enums
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn file_kind_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&FileKind::Parent).unwrap(),
        "\"parent\""
    );
    assert_eq!(
        serde_json::to_string(&FileKind::Child).unwrap(),
        "\"child\""
    );
}

#[test]
fn file_kind_deserializes_snake_case() {
    assert_eq!(
        serde_json::from_str::<FileKind>("\"parent\"").unwrap(),
        FileKind::Parent
    );
    assert_eq!(
        serde_json::from_str::<FileKind>("\"child\"").unwrap(),
        FileKind::Child
    );
}

#[test]
fn scan_status_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&ScanStatus::Complete).unwrap(),
        "\"complete\""
    );
    assert_eq!(
        serde_json::to_string(&ScanStatus::Partial).unwrap(),
        "\"partial\""
    );
}

#[test]
fn commit_intent_kind_all_variants_snake_case() {
    let variants = [
        (CommitIntentKind::Feat, "feat"),
        (CommitIntentKind::Fix, "fix"),
        (CommitIntentKind::Refactor, "refactor"),
        (CommitIntentKind::Docs, "docs"),
        (CommitIntentKind::Test, "test"),
        (CommitIntentKind::Chore, "chore"),
        (CommitIntentKind::Ci, "ci"),
        (CommitIntentKind::Build, "build"),
        (CommitIntentKind::Perf, "perf"),
        (CommitIntentKind::Style, "style"),
        (CommitIntentKind::Revert, "revert"),
        (CommitIntentKind::Other, "other"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "CommitIntentKind::{variant:?}"
        );
        let back: CommitIntentKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Enum rename_all = "kebab-case" — shared CLI enums
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn table_format_serializes_kebab_case() {
    assert_eq!(serde_json::to_string(&TableFormat::Md).unwrap(), "\"md\"");
    assert_eq!(serde_json::to_string(&TableFormat::Tsv).unwrap(), "\"tsv\"");
    assert_eq!(
        serde_json::to_string(&TableFormat::Json).unwrap(),
        "\"json\""
    );
}

#[test]
fn export_format_serializes_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ExportFormat::Csv).unwrap(),
        "\"csv\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Jsonl).unwrap(),
        "\"jsonl\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Json).unwrap(),
        "\"json\""
    );
    assert_eq!(
        serde_json::to_string(&ExportFormat::Cyclonedx).unwrap(),
        "\"cyclonedx\""
    );
}

#[test]
fn config_mode_serializes_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ConfigMode::Auto).unwrap(),
        "\"auto\""
    );
    assert_eq!(
        serde_json::to_string(&ConfigMode::None).unwrap(),
        "\"none\""
    );
}

#[test]
fn children_mode_serializes_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ChildrenMode::Collapse).unwrap(),
        "\"collapse\""
    );
    assert_eq!(
        serde_json::to_string(&ChildrenMode::Separate).unwrap(),
        "\"separate\""
    );
}

#[test]
fn child_include_mode_serializes_kebab_case() {
    assert_eq!(
        serde_json::to_string(&ChildIncludeMode::Separate).unwrap(),
        "\"separate\""
    );
    assert_eq!(
        serde_json::to_string(&ChildIncludeMode::ParentsOnly).unwrap(),
        "\"parents-only\""
    );
}

#[test]
fn redact_mode_serializes_kebab_case() {
    assert_eq!(
        serde_json::to_string(&RedactMode::None).unwrap(),
        "\"none\""
    );
    assert_eq!(
        serde_json::to_string(&RedactMode::Paths).unwrap(),
        "\"paths\""
    );
    assert_eq!(serde_json::to_string(&RedactMode::All).unwrap(), "\"all\"");
}

#[test]
fn analysis_format_all_variants_kebab_case() {
    let variants = [
        (AnalysisFormat::Md, "md"),
        (AnalysisFormat::Json, "json"),
        (AnalysisFormat::Jsonld, "jsonld"),
        (AnalysisFormat::Xml, "xml"),
        (AnalysisFormat::Svg, "svg"),
        (AnalysisFormat::Mermaid, "mermaid"),
        (AnalysisFormat::Obj, "obj"),
        (AnalysisFormat::Midi, "midi"),
        (AnalysisFormat::Tree, "tree"),
        (AnalysisFormat::Html, "html"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "AnalysisFormat::{variant:?}"
        );
        let back: AnalysisFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Enum rename_all = "snake_case" — bundle hygiene enums
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn file_classification_all_variants_snake_case() {
    let variants = [
        (FileClassification::Generated, "generated"),
        (FileClassification::Fixture, "fixture"),
        (FileClassification::Vendored, "vendored"),
        (FileClassification::Lockfile, "lockfile"),
        (FileClassification::Minified, "minified"),
        (FileClassification::DataBlob, "data_blob"),
        (FileClassification::Sourcemap, "sourcemap"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "FileClassification::{variant:?}"
        );
        let back: FileClassification = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn inclusion_policy_all_variants_snake_case() {
    let variants = [
        (InclusionPolicy::Full, "full"),
        (InclusionPolicy::HeadTail, "head_tail"),
        (InclusionPolicy::Summary, "summary"),
        (InclusionPolicy::Skip, "skip"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "InclusionPolicy::{variant:?}"
        );
        let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn capability_state_all_variants_snake_case() {
    let variants = [
        (CapabilityState::Available, "available"),
        (CapabilityState::Skipped, "skipped"),
        (CapabilityState::Unavailable, "unavailable"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "CapabilityState::{variant:?}"
        );
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Cockpit enums — rename_all = "lowercase" / "snake_case"
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn gate_status_all_variants_lowercase() {
    let variants = [
        (GateStatus::Pass, "pass"),
        (GateStatus::Warn, "warn"),
        (GateStatus::Fail, "fail"),
        (GateStatus::Skipped, "skipped"),
        (GateStatus::Pending, "pending"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, format!("\"{}\"", expected), "GateStatus::{variant:?}");
        let back: GateStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn evidence_source_all_variants_snake_case() {
    let variants = [
        (EvidenceSource::CiArtifact, "ci_artifact"),
        (EvidenceSource::Cached, "cached"),
        (EvidenceSource::RanLocal, "ran_local"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "EvidenceSource::{variant:?}"
        );
        let back: EvidenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn commit_match_all_variants_lowercase() {
    let variants = [
        (CommitMatch::Exact, "exact"),
        (CommitMatch::Partial, "partial"),
        (CommitMatch::Stale, "stale"),
        (CommitMatch::Unknown, "unknown"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "CommitMatch::{variant:?}"
        );
        let back: CommitMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn complexity_indicator_all_variants_lowercase() {
    let variants = [
        (ComplexityIndicator::Low, "low"),
        (ComplexityIndicator::Medium, "medium"),
        (ComplexityIndicator::High, "high"),
        (ComplexityIndicator::Critical, "critical"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "ComplexityIndicator::{variant:?}"
        );
        let back: ComplexityIndicator = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn warning_type_all_variants_snake_case() {
    let variants = [
        (WarningType::LargeFile, "large_file"),
        (WarningType::HighChurn, "high_churn"),
        (WarningType::LowTestCoverage, "low_test_coverage"),
        (WarningType::ComplexChange, "complex_change"),
        (WarningType::BusFactor, "bus_factor"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "WarningType::{variant:?}"
        );
        let back: WarningType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn risk_level_all_variants_lowercase() {
    let variants = [
        (RiskLevel::Low, "low"),
        (RiskLevel::Medium, "medium"),
        (RiskLevel::High, "high"),
        (RiskLevel::Critical, "critical"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, format!("\"{}\"", expected), "RiskLevel::{variant:?}");
        let back: RiskLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn trend_direction_all_variants_lowercase() {
    let variants = [
        (TrendDirection::Improving, "improving"),
        (TrendDirection::Stable, "stable"),
        (TrendDirection::Degrading, "degrading"),
    ];
    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "TrendDirection::{variant:?}"
        );
        let back: TrendDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Struct-level round-trip with aliases embedded in larger structures
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn context_receipt_with_old_token_estimation_aliases() {
    let json = json!({
        "schema_version": 4,
        "generated_at_ms": 1000000,
        "tool": { "name": "tokmd", "version": "1.0.0" },
        "mode": "context",
        "budget_tokens": 10000,
        "used_tokens": 8000,
        "utilization_pct": 0.8,
        "strategy": "greedy",
        "rank_by": "tokens",
        "file_count": 5,
        "files": [],
        "token_estimation": {
            "bytes_per_token_est": 4.0,
            "bytes_per_token_low": 3.0,
            "bytes_per_token_high": 5.0,
            "tokens_high": 800,
            "tokens_est": 1000,
            "tokens_low": 1334,
            "source_bytes": 4000
        },
        "bundle_audit": {
            "output_bytes": 5000,
            "tokens_high": 1000,
            "tokens_est": 1250,
            "tokens_low": 1667,
            "overhead_bytes": 500,
            "overhead_pct": 0.1
        }
    });
    let receipt: ContextReceipt = serde_json::from_value(json).unwrap();
    let est = receipt.token_estimation.unwrap();
    assert_eq!(est.tokens_min, 800);
    assert_eq!(est.tokens_max, 1334);
    let audit = receipt.bundle_audit.unwrap();
    assert_eq!(audit.tokens_min, 1000);
    assert_eq!(audit.tokens_max, 1667);
}

#[test]
fn diff_totals_roundtrip_preserves_all_fields() {
    let totals = DiffTotals {
        old_code: 100,
        new_code: 150,
        delta_code: 50,
        old_lines: 200,
        new_lines: 300,
        delta_lines: 100,
        old_files: 5,
        new_files: 7,
        delta_files: 2,
        old_bytes: 4000,
        new_bytes: 6000,
        delta_bytes: 2000,
        old_tokens: 1000,
        new_tokens: 1500,
        delta_tokens: 500,
    };
    let json = serde_json::to_value(&totals).unwrap();
    let back: DiffTotals = serde_json::from_value(json).unwrap();
    assert_eq!(totals, back);
}

#[test]
fn file_row_roundtrip_with_file_kind_rename() {
    let row = FileRow {
        path: "src/main.rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 100,
        comments: 20,
        blanks: 10,
        lines: 130,
        bytes: 5200,
        tokens: 1300,
    };
    let json = serde_json::to_value(&row).unwrap();
    assert_eq!(json["kind"], "parent");
    let back: FileRow = serde_json::from_value(json).unwrap();
    assert_eq!(row, back);
}

#[test]
fn file_row_child_kind_roundtrip() {
    let json_str = r#"{
        "path": "src/main.rs",
        "module": "src",
        "lang": "JavaScript",
        "kind": "child",
        "code": 50,
        "comments": 5,
        "blanks": 3,
        "lines": 58,
        "bytes": 2000,
        "tokens": 500
    }"#;
    let row: FileRow = serde_json::from_str(json_str).unwrap();
    assert_eq!(row.kind, FileKind::Child);
}

// ═══════════════════════════════════════════════════════════════════════════
// Exhaustive enum round-trip: deserialize from known string, re-serialize
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn child_include_mode_parents_only_kebab_roundtrip() {
    let json_str = "\"parents-only\"";
    let mode: ChildIncludeMode = serde_json::from_str(json_str).unwrap();
    assert_eq!(mode, ChildIncludeMode::ParentsOnly);
    let back = serde_json::to_string(&mode).unwrap();
    assert_eq!(back, json_str);
}

#[test]
fn file_classification_data_blob_snake_case_roundtrip() {
    let json_str = "\"data_blob\"";
    let cls: FileClassification = serde_json::from_str(json_str).unwrap();
    assert_eq!(cls, FileClassification::DataBlob);
    let back = serde_json::to_string(&cls).unwrap();
    assert_eq!(back, json_str);
}

#[test]
fn inclusion_policy_head_tail_snake_case_roundtrip() {
    let json_str = "\"head_tail\"";
    let policy: InclusionPolicy = serde_json::from_str(json_str).unwrap();
    assert_eq!(policy, InclusionPolicy::HeadTail);
    let back = serde_json::to_string(&policy).unwrap();
    assert_eq!(back, json_str);
}

#[test]
fn evidence_source_ci_artifact_snake_case_roundtrip() {
    let json_str = "\"ci_artifact\"";
    let src: EvidenceSource = serde_json::from_str(json_str).unwrap();
    assert_eq!(src, EvidenceSource::CiArtifact);
    let back = serde_json::to_string(&src).unwrap();
    assert_eq!(back, json_str);
}

#[test]
fn warning_type_low_test_coverage_snake_case_roundtrip() {
    let json_str = "\"low_test_coverage\"";
    let wt: WarningType = serde_json::from_str(json_str).unwrap();
    assert_eq!(wt, WarningType::LowTestCoverage);
    let back = serde_json::to_string(&wt).unwrap();
    assert_eq!(back, json_str);
}
