//! Additional deep property-based tests for tokmd-types.
//!
//! Covers: TokenAudit invariants, DiffTotals serde roundtrip,
//! AnalysisFormat/CapabilityState/ScanStatus enum round-trips,
//! Totals arithmetic bounds, and LangRow/ModuleRow field invariants.

use proptest::prelude::*;
use tokmd_types::{
    AnalysisFormat, CapabilityState, DiffRow, DiffTotals, FileKind, FileRow, LangRow, ModuleRow,
    ScanStatus, TokenAudit, TokenEstimationMeta, Totals,
};

// =========================================================================
// TokenEstimationMeta: from_bytes_with_bounds ordering
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_estimation_custom_bounds_ordering(
        bytes in 1usize..5_000_000,
        bpt_est in 2.0f64..10.0,
    ) {
        let bpt_low = bpt_est - 1.0;
        let bpt_high = bpt_est + 1.0;
        let meta = TokenEstimationMeta::from_bytes_with_bounds(bytes, bpt_est, bpt_low, bpt_high);
        prop_assert!(
            meta.tokens_min <= meta.tokens_est,
            "min {} > est {}", meta.tokens_min, meta.tokens_est
        );
        prop_assert!(
            meta.tokens_est <= meta.tokens_max,
            "est {} > max {}", meta.tokens_est, meta.tokens_max
        );
    }

    #[test]
    fn token_estimation_serde_roundtrip(bytes in 0usize..5_000_000) {
        let meta = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: TokenEstimationMeta = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(meta.tokens_min, parsed.tokens_min);
        prop_assert_eq!(meta.tokens_est, parsed.tokens_est);
        prop_assert_eq!(meta.tokens_max, parsed.tokens_max);
        prop_assert_eq!(meta.source_bytes, parsed.source_bytes);
    }

    #[test]
    fn token_estimation_monotonic_with_bytes(
        a in 0usize..1_000_000,
        delta in 1usize..1_000_000,
    ) {
        let b = a + delta;
        let meta_a = TokenEstimationMeta::from_bytes(a, TokenEstimationMeta::DEFAULT_BPT_EST);
        let meta_b = TokenEstimationMeta::from_bytes(b, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert!(
            meta_b.tokens_est >= meta_a.tokens_est,
            "More bytes ({}) should give >= tokens than fewer bytes ({})",
            b, a
        );
    }
}

// =========================================================================
// TokenAudit invariants
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_audit_overhead_bounded(
        output_bytes in 1u64..10_000_000,
        content_bytes in 0u64..10_000_000,
    ) {
        let content_bytes = content_bytes.min(output_bytes);
        let audit = TokenAudit::from_output(output_bytes, content_bytes);
        prop_assert!(audit.overhead_pct >= 0.0 && audit.overhead_pct <= 1.0,
            "overhead_pct {} out of [0,1]", audit.overhead_pct);
        prop_assert_eq!(audit.overhead_bytes, output_bytes - content_bytes);
    }

    #[test]
    fn token_audit_zero_output_zero_overhead(_dummy in 0..1u8) {
        let audit = TokenAudit::from_output(0, 0);
        prop_assert_eq!(audit.overhead_bytes, 0);
        prop_assert!((audit.overhead_pct - 0.0).abs() < 1e-10);
    }

    #[test]
    fn token_audit_ordering_invariant(output_bytes in 1u64..10_000_000) {
        let audit = TokenAudit::from_output(output_bytes, 0);
        prop_assert!(
            audit.tokens_min <= audit.tokens_est,
            "audit min {} > est {}", audit.tokens_min, audit.tokens_est
        );
        prop_assert!(
            audit.tokens_est <= audit.tokens_max,
            "audit est {} > max {}", audit.tokens_est, audit.tokens_max
        );
    }

    #[test]
    fn token_audit_serde_roundtrip(
        output_bytes in 0u64..5_000_000,
        content_bytes in 0u64..5_000_000,
    ) {
        let content_bytes = content_bytes.min(output_bytes);
        let audit = TokenAudit::from_output(output_bytes, content_bytes);
        let json = serde_json::to_string(&audit).unwrap();
        let parsed: TokenAudit = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(audit.output_bytes, parsed.output_bytes);
        prop_assert_eq!(audit.overhead_bytes, parsed.overhead_bytes);
        prop_assert_eq!(audit.tokens_est, parsed.tokens_est);
    }
}

// =========================================================================
// Additional enum serde round-trips
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn scan_status_roundtrip(idx in 0usize..2) {
        let status = [ScanStatus::Complete, ScanStatus::Partial][idx].clone();
        let json = serde_json::to_string(&status).unwrap();
        let parsed: ScanStatus = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&parsed).unwrap();
        prop_assert_eq!(json, json2, "ScanStatus serde roundtrip must be stable");
    }

    #[test]
    fn analysis_format_roundtrip(idx in 0usize..10) {
        let fmt = [
            AnalysisFormat::Md, AnalysisFormat::Json, AnalysisFormat::Jsonld,
            AnalysisFormat::Xml, AnalysisFormat::Svg, AnalysisFormat::Mermaid,
            AnalysisFormat::Obj, AnalysisFormat::Midi, AnalysisFormat::Tree,
            AnalysisFormat::Html,
        ][idx];
        let json = serde_json::to_string(&fmt).unwrap();
        let parsed: AnalysisFormat = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(fmt, parsed);
    }

    #[test]
    fn capability_state_roundtrip(idx in 0usize..3) {
        let state = [
            CapabilityState::Available,
            CapabilityState::Skipped,
            CapabilityState::Unavailable,
        ][idx];
        let json = serde_json::to_string(&state).unwrap();
        let parsed: CapabilityState = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(state, parsed);
    }

    #[test]
    fn file_kind_serde_deterministic(idx in 0usize..2) {
        let kind = [FileKind::Parent, FileKind::Child][idx];
        let json1 = serde_json::to_string(&kind).unwrap();
        let json2 = serde_json::to_string(&kind).unwrap();
        prop_assert_eq!(json1, json2);
    }
}

// =========================================================================
// DiffTotals serde roundtrip with arbitrary values
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn diff_totals_serde_roundtrip(
        old_code in 0usize..100_000,
        new_code in 0usize..100_000,
        old_lines in 0usize..200_000,
        new_lines in 0usize..200_000,
        old_files in 0usize..1_000,
        new_files in 0usize..1_000,
        old_bytes in 0usize..10_000_000,
        new_bytes in 0usize..10_000_000,
    ) {
        let totals = DiffTotals {
            old_code, new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines, new_lines,
            delta_lines: new_lines as i64 - old_lines as i64,
            old_files, new_files,
            delta_files: new_files as i64 - old_files as i64,
            old_bytes, new_bytes,
            delta_bytes: new_bytes as i64 - old_bytes as i64,
            old_tokens: 0, new_tokens: 0, delta_tokens: 0,
        };
        let json = serde_json::to_string(&totals).unwrap();
        let parsed: DiffTotals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(totals, parsed);
    }

    #[test]
    fn diff_totals_delta_consistency(
        old_code in 0usize..100_000,
        new_code in 0usize..100_000,
    ) {
        let delta = new_code as i64 - old_code as i64;
        let totals = DiffTotals {
            old_code, new_code, delta_code: delta,
            old_lines: 0, new_lines: 0, delta_lines: 0,
            old_files: 0, new_files: 0, delta_files: 0,
            old_bytes: 0, new_bytes: 0, delta_bytes: 0,
            old_tokens: 0, new_tokens: 0, delta_tokens: 0,
        };
        prop_assert_eq!(totals.delta_code, totals.new_code as i64 - totals.old_code as i64);
    }
}

// =========================================================================
// DiffRow: all delta fields consistent
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn diff_row_all_deltas_consistent(
        old_code in 0usize..50_000,
        new_code in 0usize..50_000,
        old_lines in 0usize..100_000,
        new_lines in 0usize..100_000,
        old_files in 0usize..500,
        new_files in 0usize..500,
        old_bytes in 0usize..5_000_000,
        new_bytes in 0usize..5_000_000,
        old_tokens in 0usize..500_000,
        new_tokens in 0usize..500_000,
    ) {
        let row = DiffRow {
            lang: "Test".into(),
            old_code, new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines, new_lines,
            delta_lines: new_lines as i64 - old_lines as i64,
            old_files, new_files,
            delta_files: new_files as i64 - old_files as i64,
            old_bytes, new_bytes,
            delta_bytes: new_bytes as i64 - old_bytes as i64,
            old_tokens, new_tokens,
            delta_tokens: new_tokens as i64 - old_tokens as i64,
        };
        prop_assert_eq!(row.delta_code, row.new_code as i64 - row.old_code as i64);
        prop_assert_eq!(row.delta_lines, row.new_lines as i64 - row.old_lines as i64);
        prop_assert_eq!(row.delta_files, row.new_files as i64 - row.old_files as i64);
        prop_assert_eq!(row.delta_bytes, row.new_bytes as i64 - row.old_bytes as i64);
        prop_assert_eq!(row.delta_tokens, row.new_tokens as i64 - row.old_tokens as i64);
    }
}

// =========================================================================
// Totals: serde roundtrip is byte-identical (deterministic JSON)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn totals_json_roundtrip_byte_identical(
        code in 0usize..1_000_000,
        lines in 0usize..2_000_000,
        files in 0usize..10_000,
        bytes in 0usize..100_000_000,
        tokens in 0usize..10_000_000,
        avg_lines in 0usize..5000,
    ) {
        let totals = Totals { code, lines, files, bytes, tokens, avg_lines };
        let json1 = serde_json::to_string(&totals).unwrap();
        let back: Totals = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(&json1, &json2, "Totals JSON roundtrip not byte-identical");
    }
}

// =========================================================================
// LangRow and ModuleRow: JSON serialization is deterministic
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_row_serde_deterministic(
        lang in "[A-Z][a-z]{2,10}",
        code in 0usize..50_000,
        lines in 0usize..100_000,
        files in 1usize..1_000,
        bytes in 0usize..5_000_000,
        tokens in 0usize..500_000,
        avg_lines in 0usize..500,
    ) {
        let row = LangRow { lang, code, lines, files, bytes, tokens, avg_lines };
        let json1 = serde_json::to_string(&row).unwrap();
        let back: LangRow = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(&json1, &json2);
    }

    #[test]
    fn module_row_serde_deterministic(
        module in "[a-z][a-z0-9_/]{0,20}",
        code in 0usize..50_000,
        lines in 0usize..100_000,
        files in 1usize..1_000,
        bytes in 0usize..5_000_000,
        tokens in 0usize..500_000,
        avg_lines in 0usize..500,
    ) {
        let row = ModuleRow { module, code, lines, files, bytes, tokens, avg_lines };
        let json1 = serde_json::to_string(&row).unwrap();
        let back: ModuleRow = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(&json1, &json2);
    }

    #[test]
    fn file_row_serde_deterministic(
        path in "[a-z]{1,5}/[a-z]{1,8}\\.[a-z]{1,3}",
        module in "[a-z]{1,10}",
        lang in "[A-Z][a-z]{2,10}",
        code in 0usize..50_000,
        comments in 0usize..10_000,
        blanks in 0usize..10_000,
    ) {
        let row = FileRow {
            path, module, lang,
            kind: FileKind::Parent,
            code, comments, blanks,
            lines: code + comments + blanks,
            bytes: code * 10,
            tokens: code / 4,
        };
        let json1 = serde_json::to_string(&row).unwrap();
        let back: FileRow = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(&json1, &json2);
    }
}
