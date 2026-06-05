//! # tokmd-types
//!
//! **Tier 0 (Core Types)**
//!
//! This crate defines the core data structures and contracts for `tokmd`.
//! It contains only data types, Serde definitions, and `schema_version`.
//!
//! ## Stability Policy
//!
//! **JSON-first stability**: The primary contract is the JSON schema, not Rust struct literals.
//!
//! - **JSON consumers**: Stable. New fields have sensible defaults; removed/renamed fields
//!   bump `SCHEMA_VERSION`.
//! - **Rust library consumers**: Semi-stable. New fields may be added in minor versions,
//!   which can break struct literal construction. Use `Default` + field mutation or
//!   `..Default::default()` patterns for forward compatibility.
//!
//! If you need strict Rust API stability, pin to an exact version.
//!
//! ## What belongs here
//! * Pure data structs (Receipts, Rows, Reports)
//! * Serialization/Deserialization logic
//! * Stability markers (SCHEMA_VERSION)
//!
//! ## What does NOT belong here
//! * File I/O
//! * CLI argument parsing
//! * Complex business logic
//! * Tokei dependencies

pub mod cockpit;

mod context;
mod diff;
mod evidence_packet;
mod inventory;

pub use context::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    CapabilityState, CapabilityStatus, ContextBundleManifest, ContextExcludedPath, ContextFileRow,
    ContextLogRecord, ContextReceipt, FileClassification, HANDOFF_SCHEMA_VERSION,
    HandoffComplexity, HandoffDerived, HandoffExcludedPath, HandoffHotspot, HandoffIntelligence,
    HandoffManifest, InclusionPolicy, PolicyExcludedFile, SmartExcludedFile, TokenAudit,
    TokenEstimationMeta,
};
pub use diff::{DiffReceipt, DiffRow, DiffTotals};
pub use evidence_packet::{
    EVIDENCE_PACKET_SCHEMA, EvidencePacketArtifacts, EvidencePacketManifest, EvidencePacketStatus,
};
pub use inventory::{
    AnalysisFormat, ChildIncludeMode, ChildrenMode, CommitIntentKind, ConfigMode, ExportArgs,
    ExportArgsMeta, ExportData, ExportFormat, ExportReceipt, FileKind, FileRow, LangArgs,
    LangArgsMeta, LangReceipt, LangReport, LangRow, ModuleArgs, ModuleArgsMeta, ModuleReceipt,
    ModuleReport, ModuleRow, RedactMode, RunReceipt, ScanArgs, ScanStatus, TableFormat, ToolInfo,
    Totals,
};

#[cfg(test)]
pub(crate) use context::is_default_policy;

/// The current schema version for core receipt types (`lang`, `module`, `export`, `diff`, `run`).
///
/// # Examples
///
/// ```
/// assert_eq!(tokmd_types::SCHEMA_VERSION, 2);
/// ```
pub const SCHEMA_VERSION: u32 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Schema version constants ──────────────────────────────────────
    #[test]
    fn schema_version_constants() {
        assert_eq!(SCHEMA_VERSION, 2);
        assert_eq!(HANDOFF_SCHEMA_VERSION, 5);
        assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2);
        assert_eq!(CONTEXT_SCHEMA_VERSION, 4);
    }

    // ── Default impls ─────────────────────────────────────────────────
    #[test]
    fn config_mode_default_is_auto() {
        assert_eq!(ConfigMode::default(), ConfigMode::Auto);
    }

    #[test]
    fn inclusion_policy_default_is_full() {
        assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
    }

    #[test]
    fn diff_totals_default_is_zeroed() {
        let dt = DiffTotals::default();
        assert_eq!(dt.old_code, 0);
        assert_eq!(dt.new_code, 0);
        assert_eq!(dt.delta_code, 0);
        assert_eq!(dt.delta_tokens, 0);
    }

    #[test]
    fn tool_info_default_is_empty() {
        let ti = ToolInfo::default();
        assert!(ti.name.is_empty());
        assert!(ti.version.is_empty());
    }

    #[test]
    fn tool_info_current() {
        let ti = ToolInfo::current();
        assert_eq!(ti.name, "tokmd");
        assert!(!ti.version.is_empty());
    }

    // ── Serde roundtrips for enums ────────────────────────────────────
    #[test]
    fn table_format_serde_roundtrip() {
        for variant in [TableFormat::Md, TableFormat::Tsv, TableFormat::Json] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: TableFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn export_format_serde_roundtrip() {
        for variant in [
            ExportFormat::Csv,
            ExportFormat::Jsonl,
            ExportFormat::Json,
            ExportFormat::Cyclonedx,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ExportFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn config_mode_serde_roundtrip() {
        for variant in [ConfigMode::Auto, ConfigMode::None] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ConfigMode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn children_mode_serde_roundtrip() {
        for variant in [ChildrenMode::Collapse, ChildrenMode::Separate] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: ChildrenMode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn redact_mode_serde_roundtrip() {
        for variant in [RedactMode::None, RedactMode::Paths, RedactMode::All] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: RedactMode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn file_kind_serde_roundtrip() {
        for variant in [FileKind::Parent, FileKind::Child] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: FileKind = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn scan_status_serde_roundtrip() {
        let json = serde_json::to_string(&ScanStatus::Complete).unwrap();
        assert_eq!(json, "\"complete\"");
        let back: ScanStatus = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, ScanStatus::Complete));
    }

    #[test]
    fn file_classification_serde_roundtrip() {
        for variant in [
            FileClassification::Generated,
            FileClassification::Fixture,
            FileClassification::Vendored,
            FileClassification::Lockfile,
            FileClassification::Minified,
            FileClassification::DataBlob,
            FileClassification::Sourcemap,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: FileClassification = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn inclusion_policy_serde_roundtrip() {
        for variant in [
            InclusionPolicy::Full,
            InclusionPolicy::HeadTail,
            InclusionPolicy::Summary,
            InclusionPolicy::Skip,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: InclusionPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn capability_state_serde_roundtrip() {
        for variant in [
            CapabilityState::Available,
            CapabilityState::Skipped,
            CapabilityState::Unavailable,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: CapabilityState = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn capability_status_omits_reason_when_none() {
        let status = CapabilityStatus {
            name: "git".into(),
            status: CapabilityState::Available,
            reason: None,
        };

        let value = serde_json::to_value(&status).unwrap();
        assert_eq!(value["name"], "git");
        assert_eq!(value["status"], "available");
        assert!(value.get("reason").is_none());
    }

    #[test]
    fn artifact_entry_omits_hash_when_none() {
        let artifact = ArtifactEntry {
            name: "summary.md".into(),
            path: "out/summary.md".into(),
            description: "Markdown summary".into(),
            bytes: 128,
            hash: None,
        };

        let value = serde_json::to_value(&artifact).unwrap();
        assert_eq!(value["name"], "summary.md");
        assert_eq!(value["bytes"], 128);
        assert!(value.get("hash").is_none());
    }

    #[test]
    fn analysis_format_serde_roundtrip() {
        for variant in [
            AnalysisFormat::Md,
            AnalysisFormat::Json,
            AnalysisFormat::Jsonld,
            AnalysisFormat::Xml,
            AnalysisFormat::Svg,
            AnalysisFormat::Mermaid,
            AnalysisFormat::Obj,
            AnalysisFormat::Midi,
            AnalysisFormat::Tree,
            AnalysisFormat::Html,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: AnalysisFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn commit_intent_kind_serde_roundtrip() {
        for variant in [
            CommitIntentKind::Feat,
            CommitIntentKind::Fix,
            CommitIntentKind::Refactor,
            CommitIntentKind::Docs,
            CommitIntentKind::Test,
            CommitIntentKind::Chore,
            CommitIntentKind::Ci,
            CommitIntentKind::Other,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: CommitIntentKind = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    // ── is_default_policy helper ──────────────────────────────────────
    #[test]
    fn is_default_policy_works() {
        assert!(is_default_policy(&InclusionPolicy::Full));
        assert!(!is_default_policy(&InclusionPolicy::Skip));
        assert!(!is_default_policy(&InclusionPolicy::Summary));
        assert!(!is_default_policy(&InclusionPolicy::HeadTail));
    }

    // ── Struct serde roundtrips ───────────────────────────────────────
    #[test]
    fn totals_serde_roundtrip() {
        let t = Totals {
            code: 100,
            lines: 200,
            files: 10,
            bytes: 5000,
            tokens: 250,
            avg_lines: 20,
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: Totals = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn lang_row_serde_roundtrip() {
        let r = LangRow {
            lang: "Rust".into(),
            code: 100,
            lines: 150,
            files: 5,
            bytes: 3000,
            tokens: 200,
            avg_lines: 30,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: LangRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn diff_row_serde_roundtrip() {
        let r = DiffRow {
            lang: "Rust".into(),
            old_code: 100,
            new_code: 120,
            delta_code: 20,
            old_lines: 200,
            new_lines: 220,
            delta_lines: 20,
            old_files: 10,
            new_files: 11,
            delta_files: 1,
            old_bytes: 5000,
            new_bytes: 6000,
            delta_bytes: 1000,
            old_tokens: 250,
            new_tokens: 300,
            delta_tokens: 50,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: DiffRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn diff_totals_serde_roundtrip() {
        let t = DiffTotals {
            old_code: 100,
            new_code: 120,
            delta_code: 20,
            ..DiffTotals::default()
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: DiffTotals = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }

    // ── TokenEstimationMeta ───────────────────────────────────────────
    #[test]
    fn token_estimation_from_bytes_defaults() {
        let est = TokenEstimationMeta::from_bytes(4000, TokenEstimationMeta::DEFAULT_BPT_EST);
        assert_eq!(est.source_bytes, 4000);
        assert_eq!(est.tokens_est, 1000); // 4000 / 4.0
        // tokens_min uses bpt_high=5.0 → 4000/5.0 = 800
        assert_eq!(est.tokens_min, 800);
        // tokens_max uses bpt_low=3.0 → ceil(4000/3.0) = 1334
        assert_eq!(est.tokens_max, 1334);
    }

    #[test]
    fn token_estimation_invariant_min_le_est_le_max() {
        let est = TokenEstimationMeta::from_bytes(12345, 4.0);
        assert!(est.tokens_min <= est.tokens_est);
        assert!(est.tokens_est <= est.tokens_max);
    }

    #[test]
    fn token_estimation_zero_bytes() {
        let est = TokenEstimationMeta::from_bytes(0, 4.0);
        assert_eq!(est.tokens_min, 0);
        assert_eq!(est.tokens_est, 0);
        assert_eq!(est.tokens_max, 0);
    }

    #[test]
    fn token_estimation_with_custom_bounds() {
        let est = TokenEstimationMeta::from_bytes_with_bounds(1000, 4.0, 2.0, 8.0);
        assert_eq!(est.bytes_per_token_est, 4.0);
        assert_eq!(est.bytes_per_token_low, 2.0);
        assert_eq!(est.bytes_per_token_high, 8.0);
        assert_eq!(est.tokens_est, 250); // 1000 / 4.0
        assert_eq!(est.tokens_min, 125); // 1000 / 8.0
        assert_eq!(est.tokens_max, 500); // 1000 / 2.0
    }

    // ── TokenAudit ────────────────────────────────────────────────────
    #[test]
    fn token_audit_from_output_basic() {
        let audit = TokenAudit::from_output(1000, 800);
        assert_eq!(audit.output_bytes, 1000);
        assert_eq!(audit.overhead_bytes, 200);
        assert!((audit.overhead_pct - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn token_audit_from_output_with_divisors() {
        let audit = TokenAudit::from_output_with_divisors(1000, 800, 4.0, 2.0, 8.0);

        assert_eq!(audit.output_bytes, 1000);
        assert_eq!(audit.overhead_bytes, 200);
        assert_eq!(audit.tokens_est, 250);
        assert_eq!(audit.tokens_min, 125);
        assert_eq!(audit.tokens_max, 500);
    }

    #[test]
    fn token_audit_zero_output() {
        let audit = TokenAudit::from_output(0, 0);
        assert_eq!(audit.output_bytes, 0);
        assert_eq!(audit.overhead_bytes, 0);
        assert_eq!(audit.overhead_pct, 0.0);
    }

    #[test]
    fn token_audit_content_exceeds_output() {
        // content_bytes > output_bytes should saturate to 0 overhead
        let audit = TokenAudit::from_output(100, 200);
        assert_eq!(audit.overhead_bytes, 0);
        assert_eq!(audit.overhead_pct, 0.0);
    }

    #[test]
    fn token_audit_serde_roundtrip() {
        let audit = TokenAudit::from_output(5000, 4500);
        let json = serde_json::to_string(&audit).unwrap();
        let back: TokenAudit = serde_json::from_str(&json).unwrap();
        assert_eq!(back.output_bytes, 5000);
        assert_eq!(back.overhead_bytes, 500);
    }

    // ── Kebab-case serde naming ───────────────────────────────────────
    #[test]
    fn table_format_uses_kebab_case() {
        assert_eq!(serde_json::to_string(&TableFormat::Md).unwrap(), "\"md\"");
        assert_eq!(serde_json::to_string(&TableFormat::Tsv).unwrap(), "\"tsv\"");
    }

    #[test]
    fn export_format_uses_kebab_case() {
        assert_eq!(
            serde_json::to_string(&ExportFormat::Cyclonedx).unwrap(),
            "\"cyclonedx\""
        );
    }

    #[test]
    fn redact_mode_uses_kebab_case() {
        assert_eq!(
            serde_json::to_string(&RedactMode::Paths).unwrap(),
            "\"paths\""
        );
    }

    // ── FileRow serde roundtrip ───────────────────────────────────────
    #[test]
    fn file_row_serde_roundtrip() {
        let r = FileRow {
            path: "src/main.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code: 50,
            comments: 10,
            blanks: 5,
            lines: 65,
            bytes: 2000,
            tokens: 100,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: FileRow = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub mod readme_doctests {}
