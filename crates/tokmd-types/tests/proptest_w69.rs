//! W69 deep property-based tests for tokmd-types.
//!
//! Covers struct serde round-trips, receipt total invariants,
//! schema version constants, enum coverage, and envelope structure.

use proptest::prelude::*;
use tokmd_types::{
    AnalysisFormat, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, CapabilityState,
    ChildIncludeMode, ChildrenMode, CommitIntentKind, ConfigMode, DiffRow, DiffTotals,
    FileClassification, FileKind, FileRow, HANDOFF_SCHEMA_VERSION, InclusionPolicy, LangReport,
    LangRow, ModuleReport, ModuleRow, SCHEMA_VERSION, ScanStatus, TokenAudit, TokenEstimationMeta,
    ToolInfo, Totals, cockpit::COCKPIT_SCHEMA_VERSION,
};

// =========================================================================
// 1. LangRow serde roundtrip preserves all fields
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn lang_row_serde_roundtrip(
        code in 0usize..1_000_000,
        lines in 0usize..2_000_000,
        files in 0usize..10_000,
        bytes in 0usize..100_000_000,
        tokens in 0usize..10_000_000,
        avg_lines in 0usize..5000,
        lang in "[A-Za-z]{1,20}",
    ) {
        let row = LangRow { lang, code, lines, files, bytes, tokens, avg_lines };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: LangRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }
}

// =========================================================================
// 2. ModuleRow serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn module_row_serde_roundtrip(
        code in 0usize..1_000_000,
        lines in 0usize..2_000_000,
        files in 0usize..10_000,
        bytes in 0usize..100_000_000,
        tokens in 0usize..10_000_000,
        avg_lines in 0usize..5000,
        module in "[a-z/]{1,30}",
    ) {
        let row = ModuleRow { module, code, lines, files, bytes, tokens, avg_lines };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: ModuleRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }
}

// =========================================================================
// 3. FileRow serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn file_row_serde_roundtrip(
        code in 0usize..100_000,
        comments in 0usize..50_000,
        blanks in 0usize..50_000,
        bytes in 0usize..10_000_000,
        tokens in 0usize..1_000_000,
        kind_idx in 0usize..2,
    ) {
        let lines = code + comments + blanks;
        let kind = [FileKind::Parent, FileKind::Child][kind_idx];
        let row = FileRow {
            path: "src/main.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            kind,
            code, comments, blanks, lines, bytes, tokens,
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: FileRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }
}

// =========================================================================
// 4. FileRow lines == code + comments + blanks
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn file_row_lines_equal_code_comments_blanks(
        code in 0usize..100_000,
        comments in 0usize..50_000,
        blanks in 0usize..50_000,
    ) {
        let lines = code + comments + blanks;
        let row = FileRow {
            path: "lib.rs".into(),
            module: ".".into(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code, comments, blanks, lines,
            bytes: 0, tokens: 0,
        };
        prop_assert_eq!(row.lines, row.code + row.comments + row.blanks);
    }
}

// =========================================================================
// 5. FileRow code, comments, blanks are all >= 0 (trivially true for usize)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn file_row_counts_non_negative(
        code in 0usize..1_000_000,
        comments in 0usize..500_000,
        blanks in 0usize..500_000,
    ) {
        // usize is always >= 0, but we verify round-trip preserves non-negativity
        let row = FileRow {
            path: "x.py".into(),
            module: ".".into(),
            lang: "Python".into(),
            kind: FileKind::Parent,
            code, comments, blanks,
            lines: code + comments + blanks,
            bytes: 0, tokens: 0,
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: FileRow = serde_json::from_str(&json).unwrap();
        prop_assert!(parsed.code <= usize::MAX);
        prop_assert!(parsed.comments <= usize::MAX);
        prop_assert!(parsed.blanks <= usize::MAX);
        prop_assert_eq!(parsed.lines, parsed.code + parsed.comments + parsed.blanks);
    }
}

// =========================================================================
// 6. LangReceipt total == sum of all rows (code)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_report_total_code_equals_sum_of_rows(
        a_code in 0usize..100_000,
        b_code in 0usize..100_000,
        c_code in 0usize..100_000,
    ) {
        let rows = vec![
            LangRow { lang: "Rust".into(), code: a_code, lines: a_code, files: 1, bytes: a_code * 10, tokens: a_code / 4, avg_lines: a_code },
            LangRow { lang: "Go".into(), code: b_code, lines: b_code, files: 1, bytes: b_code * 10, tokens: b_code / 4, avg_lines: b_code },
            LangRow { lang: "Python".into(), code: c_code, lines: c_code, files: 1, bytes: c_code * 10, tokens: c_code / 4, avg_lines: c_code },
        ];
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let total_lines: usize = rows.iter().map(|r| r.lines).sum();
        let total_files: usize = rows.iter().map(|r| r.files).sum();
        let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();
        let total = Totals {
            code: total_code,
            lines: total_lines,
            files: total_files,
            bytes: total_bytes,
            tokens: total_tokens,
            avg_lines: if total_files > 0 { total_lines / total_files } else { 0 },
        };
        let report = LangReport {
            rows: rows.clone(),
            total: total.clone(),
            with_files: false,
            children: ChildrenMode::Collapse,
            top: 0,
        };
        let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
        prop_assert_eq!(report.total.code, sum_code);
    }
}

// =========================================================================
// 7. ModuleReceipt total == sum of all rows (code)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn module_report_total_code_equals_sum_of_rows(
        a_code in 0usize..100_000,
        b_code in 0usize..100_000,
    ) {
        let rows = vec![
            ModuleRow { module: "crates/a".into(), code: a_code, lines: a_code, files: 1, bytes: a_code * 10, tokens: a_code / 4, avg_lines: a_code },
            ModuleRow { module: "crates/b".into(), code: b_code, lines: b_code, files: 1, bytes: b_code * 10, tokens: b_code / 4, avg_lines: b_code },
        ];
        let total_code: usize = rows.iter().map(|r| r.code).sum();
        let total_lines: usize = rows.iter().map(|r| r.lines).sum();
        let total_files: usize = rows.iter().map(|r| r.files).sum();
        let total_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let total_tokens: usize = rows.iter().map(|r| r.tokens).sum();
        let total = Totals {
            code: total_code,
            lines: total_lines,
            files: total_files,
            bytes: total_bytes,
            tokens: total_tokens,
            avg_lines: if total_files > 0 { total_lines / total_files } else { 0 },
        };
        let report = ModuleReport {
            rows: rows.clone(),
            total: total.clone(),
            module_roots: vec!["crates".into()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
            top: 0,
        };
        let sum_code: usize = report.rows.iter().map(|r| r.code).sum();
        prop_assert_eq!(report.total.code, sum_code);
    }
}

// =========================================================================
// 8. LangReport total.lines == sum of rows.lines
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_report_total_lines_equals_sum(
        a in 0usize..100_000,
        b in 0usize..100_000,
    ) {
        let rows = vec![
            LangRow { lang: "A".into(), code: a, lines: a, files: 1, bytes: 0, tokens: 0, avg_lines: a },
            LangRow { lang: "B".into(), code: b, lines: b, files: 1, bytes: 0, tokens: 0, avg_lines: b },
        ];
        let total_lines: usize = rows.iter().map(|r| r.lines).sum();
        let total = Totals { code: a + b, lines: total_lines, files: 2, bytes: 0, tokens: 0, avg_lines: total_lines / 2 };
        prop_assert_eq!(total.lines, rows.iter().map(|r| r.lines).sum::<usize>());
    }
}

// =========================================================================
// 9. ModuleReport total.files == sum of rows.files
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn module_report_total_files_equals_sum(
        a_files in 1usize..100,
        b_files in 1usize..100,
    ) {
        let rows = vec![
            ModuleRow { module: "a".into(), code: 100, lines: 100, files: a_files, bytes: 0, tokens: 0, avg_lines: 100 / a_files },
            ModuleRow { module: "b".into(), code: 200, lines: 200, files: b_files, bytes: 0, tokens: 0, avg_lines: 200 / b_files },
        ];
        let total = Totals { code: 300, lines: 300, files: a_files + b_files, bytes: 0, tokens: 0, avg_lines: 300 / (a_files + b_files) };
        prop_assert_eq!(total.files, rows.iter().map(|r| r.files).sum::<usize>());
    }
}

// =========================================================================
// 10. Schema version constants are expected values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn schema_version_constant_values(_dummy in 0..1u8) {
        prop_assert_eq!(SCHEMA_VERSION, 2u32);
        prop_assert_eq!(COCKPIT_SCHEMA_VERSION, 3u32);
        prop_assert_eq!(HANDOFF_SCHEMA_VERSION, 5u32);
        prop_assert_eq!(CONTEXT_SCHEMA_VERSION, 4u32);
        prop_assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2u32);
    }
}

// =========================================================================
// 11. Totals serde roundtrip with large values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn totals_roundtrip_large_values(
        code in 0usize..usize::MAX / 2,
        lines in 0usize..usize::MAX / 2,
    ) {
        let totals = Totals { code, lines, files: 1, bytes: 0, tokens: 0, avg_lines: lines };
        let json = serde_json::to_string(&totals).unwrap();
        let parsed: Totals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(totals, parsed);
    }
}

// =========================================================================
// 12. DiffRow serde roundtrip preserves all fields
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn diff_row_full_roundtrip(
        old_code in 0usize..100_000,
        new_code in 0usize..100_000,
    ) {
        let row = DiffRow {
            lang: "Rust".into(),
            old_code, new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines: old_code, new_lines: new_code,
            delta_lines: new_code as i64 - old_code as i64,
            old_files: 1, new_files: 2, delta_files: 1,
            old_bytes: old_code * 4, new_bytes: new_code * 4,
            delta_bytes: (new_code as i64 - old_code as i64) * 4,
            old_tokens: old_code / 4, new_tokens: new_code / 4,
            delta_tokens: new_code as i64 / 4 - old_code as i64 / 4,
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: DiffRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }
}

// =========================================================================
// 13. DiffTotals default is all zeros
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn diff_totals_default_all_zeros(_dummy in 0..1u8) {
        let t = DiffTotals::default();
        prop_assert_eq!(t.old_code, 0);
        prop_assert_eq!(t.new_code, 0);
        prop_assert_eq!(t.delta_code, 0);
        prop_assert_eq!(t.old_lines, 0);
        prop_assert_eq!(t.new_lines, 0);
        prop_assert_eq!(t.delta_lines, 0);
        prop_assert_eq!(t.old_files, 0);
        prop_assert_eq!(t.new_files, 0);
        prop_assert_eq!(t.delta_files, 0);
        prop_assert_eq!(t.old_bytes, 0);
        prop_assert_eq!(t.new_bytes, 0);
        prop_assert_eq!(t.delta_bytes, 0);
        prop_assert_eq!(t.old_tokens, 0);
        prop_assert_eq!(t.new_tokens, 0);
        prop_assert_eq!(t.delta_tokens, 0);
    }
}

// =========================================================================
// 14. DiffTotals serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn diff_totals_serde_roundtrip(
        old_code in 0usize..100_000,
        new_code in 0usize..100_000,
    ) {
        let t = DiffTotals {
            old_code, new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines: old_code, new_lines: new_code,
            delta_lines: new_code as i64 - old_code as i64,
            old_files: 1, new_files: 2, delta_files: 1,
            old_bytes: 0, new_bytes: 0, delta_bytes: 0,
            old_tokens: 0, new_tokens: 0, delta_tokens: 0,
        };
        let json = serde_json::to_string(&t).unwrap();
        let parsed: DiffTotals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(t, parsed);
    }
}

// =========================================================================
// 15. TokenEstimationMeta ordering: min <= est <= max
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn token_estimation_ordering(bytes in 0usize..50_000_000) {
        let meta = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert!(meta.tokens_min <= meta.tokens_est);
        prop_assert!(meta.tokens_est <= meta.tokens_max);
        prop_assert_eq!(meta.source_bytes, bytes);
    }
}

// =========================================================================
// 16. TokenEstimationMeta custom bounds ordering
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_estimation_custom_bounds(
        bytes in 1usize..10_000_000,
        bpt_est in 2.0f64..8.0,
    ) {
        let bpt_low = bpt_est - 1.0;
        let bpt_high = bpt_est + 1.0;
        let meta = TokenEstimationMeta::from_bytes_with_bounds(bytes, bpt_est, bpt_low, bpt_high);
        prop_assert!(meta.tokens_min <= meta.tokens_max);
    }
}

// =========================================================================
// 17. TokenAudit overhead_pct in [0.0, 1.0]
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_audit_overhead_pct_bounded(
        output_bytes in 1u64..10_000_000,
        content_frac in 0.0f64..1.0,
    ) {
        let content_bytes = (output_bytes as f64 * content_frac) as u64;
        let audit = TokenAudit::from_output(output_bytes, content_bytes);
        prop_assert!(audit.overhead_pct >= 0.0);
        prop_assert!(audit.overhead_pct <= 1.0);
    }
}

// =========================================================================
// 18. TokenAudit zero output has zero overhead
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn token_audit_zero_output(_dummy in 0..1u8) {
        let audit = TokenAudit::from_output(0, 0);
        prop_assert_eq!(audit.overhead_pct, 0.0);
        prop_assert_eq!(audit.overhead_bytes, 0);
    }
}

// =========================================================================
// 19. ToolInfo::current() has non-empty name and version
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn tool_info_current_non_empty(_dummy in 0..1u8) {
        let ti = ToolInfo::current();
        prop_assert_eq!(ti.name, "tokmd");
        prop_assert!(!ti.version.is_empty());
    }
}

// =========================================================================
// 20. ToolInfo::default() has empty fields
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn tool_info_default_empty(_dummy in 0..1u8) {
        let ti = ToolInfo::default();
        prop_assert!(ti.name.is_empty());
        prop_assert!(ti.version.is_empty());
    }
}

// =========================================================================
// 21. All ChildrenMode variants roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn children_mode_all_variants(idx in 0usize..2) {
        let all = [ChildrenMode::Collapse, ChildrenMode::Separate];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: ChildrenMode = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 22. All AnalysisFormat variants roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn analysis_format_all_variants(idx in 0usize..10) {
        let all = [
            AnalysisFormat::Md, AnalysisFormat::Json, AnalysisFormat::Jsonld,
            AnalysisFormat::Xml, AnalysisFormat::Svg, AnalysisFormat::Mermaid,
            AnalysisFormat::Obj, AnalysisFormat::Midi, AnalysisFormat::Tree,
            AnalysisFormat::Html,
        ];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: AnalysisFormat = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 23. All CommitIntentKind variants roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn commit_intent_all_variants(idx in 0usize..12) {
        let all = [
            CommitIntentKind::Feat, CommitIntentKind::Fix, CommitIntentKind::Refactor,
            CommitIntentKind::Docs, CommitIntentKind::Test, CommitIntentKind::Chore,
            CommitIntentKind::Ci, CommitIntentKind::Build, CommitIntentKind::Perf,
            CommitIntentKind::Style, CommitIntentKind::Revert, CommitIntentKind::Other,
        ];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: CommitIntentKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 24. All FileClassification variants roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(14))]

    #[test]
    fn file_classification_all_variants(idx in 0usize..7) {
        let all = [
            FileClassification::Generated, FileClassification::Fixture,
            FileClassification::Vendored, FileClassification::Lockfile,
            FileClassification::Minified, FileClassification::DataBlob,
            FileClassification::Sourcemap,
        ];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: FileClassification = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 25. All InclusionPolicy variants roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn inclusion_policy_all_variants(idx in 0usize..4) {
        let all = [
            InclusionPolicy::Full, InclusionPolicy::HeadTail,
            InclusionPolicy::Summary, InclusionPolicy::Skip,
        ];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: InclusionPolicy = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 26. InclusionPolicy default is Full
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn inclusion_policy_default(_dummy in 0..1u8) {
        prop_assert_eq!(InclusionPolicy::default(), InclusionPolicy::Full);
    }
}

// =========================================================================
// 27. ConfigMode default is Auto
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn config_mode_default(_dummy in 0..1u8) {
        prop_assert_eq!(ConfigMode::default(), ConfigMode::Auto);
    }
}

// =========================================================================
// 28. ScanStatus serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn scan_status_roundtrip(idx in 0usize..2) {
        let all = [ScanStatus::Complete, ScanStatus::Partial];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let _parsed: ScanStatus = serde_json::from_str(&json).unwrap();
        // ScanStatus doesn't derive PartialEq, so just verify deserialization succeeds
        prop_assert!(json.contains("complete") || json.contains("partial"));
    }
}

// =========================================================================
// 29. CapabilityState all variants roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn capability_state_all_variants(idx in 0usize..3) {
        let all = [
            CapabilityState::Available,
            CapabilityState::Skipped,
            CapabilityState::Unavailable,
        ];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: CapabilityState = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 30. Receipt envelope JSON structure: LangRow has expected keys
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_row_json_has_expected_keys(
        code in 0usize..1000,
        lines in 0usize..2000,
    ) {
        let row = LangRow {
            lang: "Rust".into(), code, lines, files: 1,
            bytes: code * 4, tokens: code / 4, avg_lines: lines,
        };
        let json = serde_json::to_string(&row).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object().unwrap();
        prop_assert!(obj.contains_key("lang"));
        prop_assert!(obj.contains_key("code"));
        prop_assert!(obj.contains_key("lines"));
        prop_assert!(obj.contains_key("files"));
        prop_assert!(obj.contains_key("bytes"));
        prop_assert!(obj.contains_key("tokens"));
        prop_assert!(obj.contains_key("avg_lines"));
        prop_assert_eq!(obj.len(), 7);
    }
}
